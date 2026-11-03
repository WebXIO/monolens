import { describe, expect, it } from "vitest"
import { parseMongoDbQuery } from ".."

const ID = '507f1f77bcf86cd799439011'

describe('parseMongoQuery — real-world queries', () => {
  it('ObjectId and an operator', () => {
    expect(parseMongoDbQuery(`{ user: ObjectId("${ID}"), age: { $gt: 15 } }`)).toEqual({
      user: { $oid: ID },
      age: { $gt: 15 },
    })
  })

  it('$and with multiple conditions', () => {
    expect(
      parseMongoDbQuery('{ $and: [{ age: { $gte: 18 } }, { active: true }] }')
    ).toEqual({
      $and: [{ age: { $gte: 18 } }, { active: true }],
    })
  })

  it('$or query', () => {
    expect(
      parseMongoDbQuery('{ $or: [{ status: "active" }, { status: "pending" }] }')
    ).toEqual({
      $or: [{ status: 'active' }, { status: 'pending' }],
    })
  })

  it('date range query with date-only ISODates', () => {
    expect(
      parseMongoDbQuery(
        '{ createdAt: { $gte: ISODate("2024-01-01"), $lt: ISODate("2025-01-01") } }'
      )
    ).toEqual({
      createdAt: {
        $gte: { $date: '2024-01-01T00:00:00.000Z' },
        $lt:  { $date: '2025-01-01T00:00:00.000Z' },
      },
    })
  })

  it('quoted dotted keys', () => {
    expect(
      parseMongoDbQuery(`{ "user._id": ObjectId("${ID}"), "user.role": "admin" }`)
    ).toEqual({
      'user._id': { $oid: ID },
      'user.role': 'admin',
    })
  })

  it('dotted keys without quotes', () => {
    expect(parseMongoDbQuery('{ user.address.city: "Graz" }')).toEqual({ 'user.address.city': 'Graz' })
  })

  it('$elemMatch query', () => {
    expect(
      parseMongoDbQuery('{ scores: { $elemMatch: { $gt: 80, $lt: 100 } } }')
    ).toEqual({
      scores: { $elemMatch: { $gt: 80, $lt: 100 } },
    })
  })

  it('complex query with multiple types', () => {
    expect(
      parseMongoDbQuery(`{
        _id: ObjectId("${ID}"),
        age: { $gte: 18, $lte: 65 },
        status: { $in: ["active", "trial"] },
        deleted: false,
        note: null
      }`)
    ).toEqual({
      _id: { $oid: ID },
      age: { $gte: 18, $lte: 65 },
      status: { $in: ['active', 'trial'] },
      deleted: false,
      note: null,
    })
  })

  it('accepts a trailing comma', () => {
    expect(parseMongoDbQuery('{ a: 1, }')).toEqual({ a: 1 })
    expect(parseMongoDbQuery('{ a: [1, 2,] }')).toEqual({ a: [1, 2] })
  })

  it('an empty object', () => {
    expect(parseMongoDbQuery('{ }')).toEqual({})
  })

  it('numbers with exponents and decimals', () => {
    expect(parseMongoDbQuery('{ a: 1e5, b: -2.5E-3, c: 0.5 }')).toEqual({ a: 100000, b: -0.0025, c: 0.5 })
  })
})

describe('parseMongoQuery — shell functions', () => {
  it('lowercases ObjectId hex', () => {
    expect(parseMongoDbQuery(`{ _id: ObjectId("${ID.toUpperCase()}") }`)).toEqual({ _id: { $oid: ID } })
  })

  it('new before a function is ignored', () => {
    expect(parseMongoDbQuery('{ at: new Date("2024-03-01") }')).toEqual({ at: { $date: '2024-03-01T00:00:00.000Z' } })
    expect(parseMongoDbQuery(`{ _id: new ObjectId("${ID}") }`)).toEqual({ _id: { $oid: ID } })
  })

  it('reads a date-time without zone as UTC', () => {
    expect(parseMongoDbQuery('{ at: ISODate("2024-03-01T10:15") }')).toEqual({ at: { $date: '2024-03-01T10:15:00.000Z' } })
    expect(parseMongoDbQuery('{ at: ISODate("2024-03-01T10:15:30.5") }')).toEqual({ at: { $date: '2024-03-01T10:15:30.500Z' } })
  })

  it('reads a space between date and time like a T', () => {
    expect(parseMongoDbQuery('{ at: ISODate("2024-03-01 10:15") }')).toEqual({ at: { $date: '2024-03-01T10:15:00.000Z' } })
  })

  it('converts a date with an offset to UTC', () => {
    expect(parseMongoDbQuery('{ at: ISODate("2024-03-01T10:00:00+02:00") }')).toEqual({ at: { $date: '2024-03-01T08:00:00.000Z' } })
  })

  it('takes milliseconds since the epoch', () => {
    expect(parseMongoDbQuery('{ at: Date(0) }')).toEqual({ at: { $date: '1970-01-01T00:00:00.000Z' } })
  })

  it('Date() without argument is now', () => {
    const before = Date.now()
    const { at } = parseMongoDbQuery('{ at: new Date() }') as { at: { $date: string } }
    const ms = Date.parse(at.$date)
    expect(ms).toBeGreaterThanOrEqual(before - 1000)
    expect(ms).toBeLessThanOrEqual(Date.now() + 1000)
  })

  it('NumberLong, NumberInt and NumberDecimal keep their digits as strings', () => {
    expect(parseMongoDbQuery('{ a: NumberLong("9007199254740993"), b: NumberLong(5), c: NumberInt(-7), d: NumberDecimal("1.10") }')).toEqual({
      a: { $numberLong: '9007199254740993' },
      b: { $numberLong: '5' },
      c: { $numberInt: '-7' },
      d: { $numberDecimal: '1.10' },
    })
  })

  it('Timestamp', () => {
    expect(parseMongoDbQuery('{ ts: Timestamp(1700000000, 3) }')).toEqual({ ts: { $timestamp: { t: 1700000000, i: 3 } } })
  })

  it('BinData with the subtype as hex', () => {
    expect(parseMongoDbQuery('{ b: BinData(4, "AAEC") }')).toEqual({ b: { $binary: { base64: 'AAEC', subType: '04' } } })
    expect(parseMongoDbQuery('{ b: BinData(128, "") }')).toEqual({ b: { $binary: { base64: '', subType: '80' } } })
  })

  it('UUID as binary subtype 4', () => {
    expect(parseMongoDbQuery('{ u: UUID("00112233-4455-6677-8899-aabbccddeeff") }')).toEqual({
      u: { $binary: { base64: 'ABEiM0RVZneImaq7zN3u/w==', subType: '04' } },
    })
  })

  it('RegExp and regex literals become $regularExpression with sorted options', () => {
    expect(parseMongoDbQuery('{ name: RegExp("^al", "mi") }')).toEqual({ name: { $regularExpression: { pattern: '^al', options: 'im' } } })
    expect(parseMongoDbQuery('{ name: /^al/i }')).toEqual({ name: { $regularExpression: { pattern: '^al', options: 'i' } } })
    expect(parseMongoDbQuery('{ name: { $in: [/a\\/b/, /c/] } }')).toEqual({
      name: { $in: [
        { $regularExpression: { pattern: 'a\\/b', options: '' } },
        { $regularExpression: { pattern: 'c', options: '' } },
      ] },
    })
  })
})

describe('parseMongoQuery — error handling', () => {
  it('throws on unexpected character', () => {
    expect(() => parseMongoDbQuery('{ a: @invalid }')).toThrow()
  })

  it('throws on missing closing brace', () => {
    expect(() => parseMongoDbQuery('{ a: 1')).toThrow()
  })

  it('throws on bare unknown identifier', () => {
    expect(() => parseMongoDbQuery('{ x: unknownIdent }')).toThrow(/Unknown identifier/)
  })

  it('throws on missing colon', () => {
    expect(() => parseMongoDbQuery('{ a 1 }')).toThrow()
  })

  it('throws on a missing comma', () => {
    expect(() => parseMongoDbQuery('{ a: 1 b: 2 }')).toThrow(/Expected , or }/)
    expect(() => parseMongoDbQuery('{ a: [1 2] }')).toThrow(/Expected , or ]/)
  })

  it('throws on anything after the query', () => {
    expect(() => parseMongoDbQuery('{ a: 1 } }')).toThrow(/after the end of the query/)
    expect(() => parseMongoDbQuery('{ a: 1 } { b: 2 }')).toThrow(/after the end of the query/)
  })

  it('throws on an unknown function', () => {
    expect(() => parseMongoDbQuery('{ a: Foo(1) }')).toThrow(/Unknown function: Foo/)
  })

  it('throws on an ObjectId that is not 24 hex characters', () => {
    expect(() => parseMongoDbQuery('{ _id: ObjectId("abc123") }')).toThrow(/ObjectId/)
    expect(() => parseMongoDbQuery('{ _id: ObjectId("507f1f77bcf86cd79943901z") }')).toThrow(/ObjectId/)
    expect(() => parseMongoDbQuery('{ _id: ObjectId() }')).toThrow(/ObjectId/)
  })

  it('throws on an invalid date', () => {
    expect(() => parseMongoDbQuery('{ at: ISODate("yesterday") }')).toThrow(/Invalid date/)
    expect(() => parseMongoDbQuery('{ at: ISODate("2024-13-45") }')).toThrow(/Invalid date/)
    expect(() => parseMongoDbQuery('{ at: Date(2024, 1, 1) }')).toThrow(/one argument/)
  })

  it('throws on numbers that are not integers where integers are needed', () => {
    expect(() => parseMongoDbQuery('{ a: NumberLong("12x") }')).toThrow(/NumberLong/)
    expect(() => parseMongoDbQuery('{ a: NumberInt(1.5) }')).toThrow(/NumberInt/)
    expect(() => parseMongoDbQuery('{ a: NumberInt(2147483648) }')).toThrow(/out of range/)
    expect(() => parseMongoDbQuery('{ a: NumberDecimal("abc") }')).toThrow(/NumberDecimal/)
    expect(() => parseMongoDbQuery('{ a: Timestamp(1) }')).toThrow(/Timestamp/)
    expect(() => parseMongoDbQuery('{ a: Timestamp(-1, 0) }')).toThrow(/Timestamp/)
  })

  it('throws on invalid binary data', () => {
    expect(() => parseMongoDbQuery('{ b: BinData(256, "AA==") }')).toThrow(/subtype/)
    expect(() => parseMongoDbQuery('{ b: BinData(0, "not base64!") }')).toThrow(/base64/)
    expect(() => parseMongoDbQuery('{ u: UUID("1234") }')).toThrow(/UUID/)
  })

  it('throws on unsupported regex flags', () => {
    expect(() => parseMongoDbQuery('{ a: /x/g }')).toThrow(/flags/)
    expect(() => parseMongoDbQuery('{ a: RegExp("x", "y") }')).toThrow(/flags/)
  })

  it('throws on an unterminated string or regex', () => {
    expect(() => parseMongoDbQuery('{ a: "open }')).toThrow(/Unterminated string/)
    expect(() => parseMongoDbQuery('{ a: /open }')).toThrow(/Unterminated regular expression/)
  })
})
