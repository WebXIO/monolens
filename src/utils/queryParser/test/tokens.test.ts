import { test, expect, it } from "vitest";
import { generateTokens } from "../generateTokens";
import { T } from "../types";

test("Test Tokens generation", () => {
  const tokens = generateTokens(`{name: "test", age: 16}`);

  expect(tokens).toMatchObject([
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
});

test("Test Tokens generation function calls", () => {
  const tokens = generateTokens(`{_id: ObjectId("...")}`);

  expect(tokens).toMatchObject([
    { type: "LBRACE" },
    { type: "IDENT", value: "_id" },
    { type: "COLON" },
    { type: "IDENT", value: "ObjectId" },
    { type: "LPAREN" },
    { type: "STRING", value: "..." },
    { type: "RPAREN" },
    { type: "RBRACE" },
    { type: "EOF" },
  ]);
});

test("tokenizes all single-character symbols", () => {
  const types = generateTokens("{}[]():,").map((t) => t.type);
  expect(types).toEqual([
    "LBRACE",
    "RBRACE",
    "LBRACKET",
    "RBRACKET",
    "LPAREN",
    "RPAREN",
    "COLON",
    "COMMA",
    "EOF",
  ]);
});

it("tokenizes a double-quoted string", () => {
  const [tok] = generateTokens('"hello"');
  expect(tok).toEqual({ type: "STRING", value: "hello" });
});

it("tokenizes a single-quoted string", () => {
  const [tok] = generateTokens("'world'");
  expect(tok).toEqual({ type: "STRING", value: "world" });
});

it("handles escaped quotes inside strings", () => {
  const [tok] = generateTokens('"he said \\"hi\\""');
  expect(tok.value).toBe('he said "hi"');
});

it("tokenizes a positive integer", () => {
  const [tok] = generateTokens("42");
  expect(tok).toEqual({ type: "NUMBER", value: 42 });
});

it("tokenizes a negative integer", () => {
  const [tok] = generateTokens("-7");
  expect(tok).toEqual({ type: "NUMBER", value: -7 });
});

it("tokenizes a float", () => {
  const [tok] = generateTokens("3.14");
  expect(tok).toEqual({ type: "NUMBER", value: 3.14 });
});

it("tokenizes an identifier", () => {
  const [tok] = generateTokens("username");
  expect(tok).toEqual({ type: "IDENT", value: "username" });
});

it("tokenizes a $operator identifier", () => {
  const [tok] = generateTokens("$gt");
  expect(tok).toEqual({ type: "IDENT", value: "$gt" });
});

it("skips whitespace and newlines", () => {
  const types = generateTokens("  {  }  ").map((t) => t.type);
  expect(types).toEqual(["LBRACE", "RBRACE", "EOF"]);
});

it("throws on unexpected character", () => {
  expect(() => generateTokens("{")).not.toThrow(); // just brace is fine
  expect(() => generateTokens("@")).toThrow(/Unexpected character/);
});
