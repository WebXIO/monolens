import { describe, expect, it } from "vitest"
import { parseMongoDbQuery } from ".."

describe('parseMongoQuery — real-world queries', () => {
  it('query from the original question', () => {
    expect(parseMongoDbQuery('{ user: ObjectId("abc123"), age: { $gt: 15 } }')).toEqual({
      user: { $oid: 'abc123' },
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

  it('date range query', () => {
    expect(
      parseMongoDbQuery(
        '{ createdAt: { $gte: ISODate("2024-01-01"), $lt: ISODate("2025-01-01") } }'
      )
    ).toEqual({
      createdAt: {
        $gte: { $date: '2024-01-01' },
        $lt:  { $date: '2025-01-01' },
      },
    })
  })

  it('nested document with ObjectId reference', () => {
    expect(
      parseMongoDbQuery('{ "user._id": ObjectId("abc"), "user.role": "admin" }')
    ).toEqual({
      'user._id': { $oid: 'abc' },
      'user.role': 'admin',
    })
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
        _id: ObjectId("507f1f77bcf86cd799439011"),
        age: { $gte: 18, $lte: 65 },
        status: { $in: ["active", "trial"] },
        deleted: false
      }`)
    ).toEqual({
      _id: { $oid: '507f1f77bcf86cd799439011' },
      age: { $gte: 18, $lte: 65 },
      status: { $in: ['active', 'trial'] },
      deleted: false,
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
    expect(() => parseMongoDbQuery('{ x: unknownIdent }')).toThrow()
  })

  it('throws on missing colon', () => {
    expect(() => parseMongoDbQuery('{ a 1 }')).toThrow()
  })

  it('throws', () => {
    parseMongoDbQuery('{ a: 1, }')
  })
})