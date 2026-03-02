import { isNumber, isWhitespace } from "./helpers";
import { T, Token } from "./types";

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

      while (i < input.length) {
        i++;
        const current = input[i];

        if (escaped) {
          str += current;
          escaped = false;
        } else if (current === "\\") {
          escaped = true;
        } else if (current === quote) {
          break;
        } else {
          str += current;
        }
      }
      i++;
      tokens.push({ type: T.STRING, value: str });
      continue;
    }

    if (isNumber(ch) || (ch === "-" && isNumber(input[i + 1]))) {
      let num = "";
      if (ch === "-") {
        num += "-";
        i++;
      }

      while (i < input.length && /[0-9.]/.test(input[i])) {
        num += input[i++];
      }
      tokens.push({ type: T.NUMBER, value: Number(num) });
      continue;
    }

    // Identifiers: field names, $operators, function names, true/false/null
    if (/[a-zA-Z_$]/.test(ch)) {
      let ident = "";
      while (i < input.length && /[a-zA-Z0-9_$]/.test(input[i])) {
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
