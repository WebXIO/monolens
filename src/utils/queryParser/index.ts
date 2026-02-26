import { generateTokens } from "./generateTokens";
import { createParser } from "./parser";

export function parseMongoDbQuery(query: string) {
   const tokens = generateTokens(query);
   const parser = createParser(tokens);
   return parser.parseValue();
}