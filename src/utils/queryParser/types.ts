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
  NUMBER: 'NUMBER',       // 42, 3.14, -1
  IDENT: 'IDENT',         // user, $gt, ObjectId, true, null
  EOF: 'EOF',
}

export type Token = {
   type: string;
   value?: string | number;
}