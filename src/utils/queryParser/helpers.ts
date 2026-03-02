export function isNumber(ch: string) {
  return /\d/.test(ch);
}

export function isWhitespace(ch: string) {
   return /\s/.test(ch)
}