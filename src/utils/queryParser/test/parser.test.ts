import { test, expect } from "vitest";
import { createParser } from "../parser";

test("Test simple parser", () => {
  const parser = createParser([
    { type: "LBRACE" },
    { type: "IDENT", value: "name" },
    { type: "COLON" },
    { type: "STRING", value: "test" },
    { type: "COMMA" },
    { type: "IDENT", value: "age" },
    { type: "COLON" },
    { type: "NUMBER", value: 16 },
    { type: "RBRACE" },
    { type: "EOF" },
  ]);

  expect(parser.parseValue()).toMatchObject({ name: "test", age: 16 });
});

test("Test function calll parser", () => {
  const parser = createParser([
    { type: "LBRACE" },
    { type: "IDENT", value: "_id" },
    { type: "COLON" },
    { type: "IDENT", value: "ObjectId" },
    { type: "LPAREN" },
    { type: "STRING", value: "507f1f77bcf86cd799439011" },
    { type: "RPAREN" },
    { type: "RBRACE" },
    { type: "EOF" },
  ]);

  expect(parser.parseValue()).toMatchObject({
    _id: {
      $oid: "507f1f77bcf86cd799439011",
    },
  });
});
