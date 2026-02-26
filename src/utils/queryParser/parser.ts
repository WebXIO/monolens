import { transformFunction } from "./transformFunction"
import { T, Token } from "./types"

export function createParser(tokens: Token[]) {
  let pos = 0

  
  const peek = () => tokens[pos]

  const consume = (type?: string) => {
    const token = tokens[pos]
    if (type && token.type !== type) {
      throw new Error(`Expected ${type} but got ${token.type}`)
    }
    pos++
    return token
  }

  const check = (type: string) => peek().type === type

  function parseValue() {
    const token = peek()

    if (token.type === T.LBRACE)   return parseObject()
    if (token.type === T.LBRACKET) return parseArray()
    if (token.type === T.STRING)   { consume(); return token.value }
    if (token.type === T.NUMBER)   { consume(); return token.value }

    if (token.type === T.IDENT) {
      if (token.value === 'true')  { consume(); return true }
      if (token.value === 'false') { consume(); return false }
      if (token.value === 'null')  { consume(); return null }

      return parseFunctionCall()
    }

    throw new Error(`Unexpected token: ${token.type}`)
  }

  function parseObject() {
    consume(T.LBRACE)
    const obj: any = {}

    while (!check(T.RBRACE)) {
      
      const keyToken = consume()
      if (keyToken.type !== T.IDENT && keyToken.type !== T.STRING) {
        throw new Error(`Expected object key, got ${keyToken.type}`)
      }
      const key = keyToken.value

      if(!key) throw new Error(`Expected value, got undefined`)

      consume(T.COLON)
      const value = parseValue()
      obj[key] = value

      if (check(T.COMMA)) consume()
    }

    consume(T.RBRACE)
    return obj
  }

  function parseArray() {
    consume(T.LBRACKET)
    const arr: any[] = []

    while (!check(T.RBRACKET)) {
      arr.push(parseValue())
      if (check(T.COMMA)) consume()
    }

    consume(T.RBRACKET)
    return arr
  }

  function parseFunctionCall(): any {
    const name = consume(T.IDENT).value

    if(!name) throw new Error(`Expected value, got undefined`)

    if (!check(T.LPAREN)) {
      throw new Error(`Unknown identifier: ${name}`)
    }

    consume(T.LPAREN)
    const args = []

    while (!check(T.RPAREN)) {
      args.push(parseValue())
      if (check(T.COMMA)) consume()
    }

    consume(T.RPAREN)
    return transformFunction(String(name), args)
  }

  return { parseValue }
}