import { describe, expect, it } from "vitest";
import { fieldNames } from "../documentFields";

describe("fieldNames", () => {
  it("collects the fields of every document, _id first", () => {
    expect(fieldNames([{ name: "a", _id: 1 }, { age: 3 }, { city: "x", name: "b" }])).toEqual(["_id", "age", "city", "name"]);
  });

  it("is empty for no documents", () => {
    expect(fieldNames([])).toEqual([]);
  });
});
