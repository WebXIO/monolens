import { isWhitespace } from "./helpers";
import { T, Token } from "./types";

const NUMBER = /^-?\d+(\.\d+)?([eE][+-]?\d+)?/;
// dots are allowed after the first character, so `user.name` can be a key without quotes
const IDENT_START = /[a-zA-Z_$]/;
const IDENT_PART = /[a-zA-Z0-9_$.]/;

export function generateTokens(input: string) {
  const tokens: Token[] = [];
  let i = 0;

  while (i < input.length) {
    const ch = input[i];

    if (isWhitespace(ch)) {
      i++;
      continue;
    }

    switch (ch) {
      case "{":
        tokens.push({ type: T.LBRACE });
        i++;
        continue;
      case "}":
        tokens.push({ type: T.RBRACE });
        i++;
        continue;
      case "[":
        tokens.push({ type: T.LBRACKET });
        i++;
        continue;
      case "]":
        tokens.push({ type: T.RBRACKET });
        i++;
        continue;
      case "(":
        tokens.push({ type: T.LPAREN });
        i++;
        continue;
      case ")":
        tokens.push({ type: T.RPAREN });
        i++;
        continue;
      case ":":
        tokens.push({ type: T.COLON });
        i++;
        continue;
      case ",":
        tokens.push({ type: T.COMMA });
        i++;
        continue;
    }

    if (ch === '"' || ch === "'") {
      const quote = ch;
      let str = "";
      let escaped = false;
      let closed = false;

      while (i < input.length) {
        i++;
        const current = input[i];

        if (current === undefined) break;

        if (escaped) {
          str += current;
          escaped = false;
        } else if (current === "\\") {
          escaped = true;
        } else if (current === quote) {
          closed = true;
          break;
        } else {
          str += current;
        }
      }

      if (!closed) throw new Error(`Unterminated string starting with ${quote}${str}`);

      i++;
      tokens.push({ type: T.STRING, value: str });
      continue;
    }

    // there is no division in queries, so a slash always starts a regex literal
    if (ch === "/") {
      let pattern = "";
      let closed = false;
      i++;

      while (i < input.length) {
        const current = input[i];
        if (current === "\\" && i + 1 < input.length) {
          // keep the escape, it belongs to the regex
          pattern += current + input[i + 1];
          i += 2;
          continue;
        }
        if (current === "/") {
          closed = true;
          i++;
          break;
        }
        pattern += current;
        i++;
      }

      if (!closed) throw new Error(`Unterminated regular expression /${pattern}`);

      let flags = "";
      while (i < input.length && /[a-z]/.test(input[i])) {
        flags += input[i++];
      }

      tokens.push({ type: T.REGEX, value: pattern, flags });
      continue;
    }

    const number = NUMBER.exec(input.slice(i));
    if (number) {
      tokens.push({ type: T.NUMBER, value: Number(number[0]) });
      i += number[0].length;
      continue;
    }

    // Identifiers: field names, $operators, function names, true/false/null, new
    if (IDENT_START.test(ch)) {
      let ident = "";
      while (i < input.length && IDENT_PART.test(input[i])) {
        ident += input[i++];
      }
      tokens.push({ type: T.IDENT, value: ident });
      continue;
    }

    throw new Error(`Unexpected character: ${ch} at position ${i}`);
  }

  tokens.push({ type: T.EOF });
  return tokens;
}
