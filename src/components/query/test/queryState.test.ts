import { describe, expect, it } from "vitest";
import { fieldNames, pageInfo, parseQueryTexts } from "../queryState";

describe("parseQueryTexts", () => {
  it("parses all three inputs", () => {
    expect(parseQueryTexts({ filter: "{ age: { $gt: 18 } }", sort: "{ age: -1 }", projection: "{ name: 1, _id: 0 }" })).toEqual({
      filter: { age: { $gt: 18 } },
      sort: { age: -1 },
      projection: { name: 1, _id: 0 },
    });
  });

  it("treats empty inputs and { } as none, the filter as {}", () => {
    expect(parseQueryTexts({ filter: "", sort: "{ }", projection: "  " })).toEqual({ filter: {}, sort: null, projection: null });
  });

  it("names the input of a parse error", () => {
    expect(() => parseQueryTexts({ filter: "{}", sort: "{ age: }", projection: "" })).toThrow(/^Sort: /);
    expect(() => parseQueryTexts({ filter: "{ a: @ }", sort: "", projection: "" })).toThrow(/^Filter: /);
  });

  it("refuses values that aren't objects", () => {
    expect(() => parseQueryTexts({ filter: "5", sort: "", projection: "" })).toThrow("Filter: must be an object like { field: value }");
    expect(() => parseQueryTexts({ filter: "{}", sort: "[1]", projection: "" })).toThrow(/^Sort: must be an object/);
    expect(() => parseQueryTexts({ filter: "{}", sort: "", projection: "null" })).toThrow(/^Projection: must be an object/);
  });
});

describe("fieldNames", () => {
  it("collects the fields of every document, _id first", () => {
    expect(fieldNames([{ name: "a", _id: 1 }, { age: 3 }, { city: "x", name: "b" }])).toEqual(["_id", "age", "city", "name"]);
  });

  it("is empty for no documents", () => {
    expect(fieldNames([])).toEqual([]);
  });
});

describe("pageInfo", () => {
  it("shows the range of the page and the total", () => {
    expect(pageInfo(50, 50, 137, false, 50)).toEqual({ label: "51–100 of 137", hasPrevious: true, hasNext: true });
  });

  it("has no next page on the last one", () => {
    expect(pageInfo(100, 37, 137, false, 50)).toEqual({ label: "101–137 of 137", hasPrevious: true, hasNext: false });
  });

  it("marks an estimated total and allows next after a full page", () => {
    expect(pageInfo(0, 50, 50, true, 50)).toEqual({ label: "1–50 of ~50", hasPrevious: false, hasNext: true });
    expect(pageInfo(0, 20, 50, true, 50).hasNext).toBe(true);
    expect(pageInfo(0, 20, 20, true, 50).hasNext).toBe(false);
  });

  it("shows an empty result", () => {
    expect(pageInfo(0, 0, 0, false, 50)).toEqual({ label: "0 of 0", hasPrevious: false, hasNext: false });
  });

  it("formats large numbers", () => {
    expect(pageInfo(1000, 50, 1234567, false, 50).label).toBe("1,001–1,050 of 1,234,567");
  });
});
