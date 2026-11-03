import { beforeEach, describe, expect, it } from "vitest";
import { effectScope, nextTick, ref } from "vue";
import { type JsonDocument } from "@/components/documents/models/types";
import { mergeColumnOrder, useDocumentTable } from "../useDocumentTable";

const STORAGE_KEY = "conn:db:coll";

function setup(docs: JsonDocument[]) {
  const documents = ref<JsonDocument[]>(docs);
  const storageKey = ref(STORAGE_KEY);
  const scope = effectScope();
  const result = scope.run(() => useDocumentTable(documents, storageKey))!;
  return { ...result, documents, scope };
}

function headerIds(table: ReturnType<typeof setup>["table"]) {
  return table.getFlatHeaders().map((h) => h.column.id);
}

const docs: JsonDocument[] = [
  { _id: { $oid: "a1" }, name: "Bea", age: 31 },
  { _id: { $oid: "a2" }, name: "Al", age: 7, city: "Graz" },
  { _id: { $oid: "a3" }, name: "Cid", age: 19 },
];

beforeEach(() => {
  localStorage.clear();
});

describe("mergeColumnOrder", () => {
  it("keeps the saved order and appends new fields", () => {
    expect(mergeColumnOrder(["b", "a"], ["a", "b", "c"])).toEqual(["b", "a", "c"]);
  });

  it("drops saved fields that no longer exist", () => {
    expect(mergeColumnOrder(["x", "b", "a"], ["a", "b"])).toEqual(["b", "a"]);
  });

  it("returns the current fields when nothing is saved", () => {
    expect(mergeColumnOrder([], ["a", "b"])).toEqual(["a", "b"]);
  });

  it("returns nothing for no current fields", () => {
    expect(mergeColumnOrder(["a"], [])).toEqual([]);
  });
});

describe("useDocumentTable", () => {
  it("has one column per top-level field, _id first, then alphabetical", () => {
    const { table } = setup(docs);
    expect(headerIds(table)).toEqual(["_id", "age", "city", "name"]);
  });

  it("has no columns and rows without documents", () => {
    const { table } = setup([]);
    expect(headerIds(table)).toEqual([]);
    expect(table.getRowModel().rows).toHaveLength(0);
  });

  it("exposes the raw values in column order", () => {
    const { table } = setup(docs);
    const cells = table.getRowModel().rows[0].getVisibleCells();
    expect(cells.map((c) => c.getValue())).toEqual([{ $oid: "a1" }, 31, undefined, "Bea"]);
  });

  it("sorts rows ascending and descending by a column", async () => {
    const { table } = setup(docs);
    table.setSorting([{ id: "age", desc: false }]);
    await nextTick();
    expect(table.getRowModel().rows.map((r) => r.original.age)).toEqual([7, 19, 31]);

    table.setSorting([{ id: "name", desc: true }]);
    await nextTick();
    expect(table.getRowModel().rows.map((r) => r.original.name)).toEqual(["Cid", "Bea", "Al"]);
  });

  it("applies a new column order to headers and cells and saves it", async () => {
    const { table } = setup(docs);
    table.setColumnOrder(["name", "_id", "age", "city"]);
    await nextTick();
    expect(headerIds(table)).toEqual(["name", "_id", "age", "city"]);
    expect(table.getRowModel().rows[0].getVisibleCells().map((c) => c.column.id)).toEqual(["name", "_id", "age", "city"]);
    expect(JSON.parse(localStorage.getItem(`monolens:columnOrder:${STORAGE_KEY}`)!)).toEqual(["name", "_id", "age", "city"]);
  });

  it("restores a saved column order and appends new fields", () => {
    localStorage.setItem(`monolens:columnOrder:${STORAGE_KEY}`, JSON.stringify(["name", "age"]));
    const { table } = setup(docs);
    expect(headerIds(table)).toEqual(["name", "age", "_id", "city"]);
  });

  it("follows replaced documents", async () => {
    const { table, documents } = setup(docs);
    documents.value = [{ _id: 1, title: "x" }];
    await nextTick();
    expect(headerIds(table)).toEqual(["_id", "title"]);
    expect(table.getRowModel().rows).toHaveLength(1);
  });

  it("sizes columns between the minimum and the maximum width", () => {
    const { table } = setup([{ _id: 1, long: "x".repeat(400) }]);
    const sizes = Object.fromEntries(table.getFlatHeaders().map((h) => [h.column.id, h.getSize()]));
    expect(sizes._id).toBeGreaterThanOrEqual(60);
    expect(sizes.long).toBe(500);
  });
});
