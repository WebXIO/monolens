// TOKENS
export const T = {
  LBRACE: 'LBRACE',       // {
  RBRACE: 'RBRACE',       // }
  LBRACKET: 'LBRACKET',   // [
  RBRACKET: 'RBRACKET',   // ]
  LPAREN: 'LPAREN',       // (
  RPAREN: 'RPAREN',       // )
  COLON: 'COLON',         // :
  COMMA: 'COMMA',         // ,
  STRING: 'STRING',       // "hello" or 'hello'
  NUMBER: 'NUMBER',       // 42, 3.14, -1, 1e5
  IDENT: 'IDENT',         // user, user.name, $gt, ObjectId, true, null
  REGEX: 'REGEX',         // /pattern/flags
  EOF: 'EOF',
}

export type Token = {
   type: string;
   value?: string | number;
   // only on REGEX tokens
   flags?: string;
}
