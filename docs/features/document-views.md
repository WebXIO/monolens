# Document views

## Purpose

The result of a query is shown as a table, one row per document and one column per field, or as formatted JSON, one block per document. The table is the default. Both are read-only.

## How it works

**Input.** Rust turns each document into relaxed extended JSON (`drivers/documents.rs`): `ObjectId` as `{ "$oid": "…" }`, dates as `{ "$date": "2024-01-01T00:00:00Z" }`, decimals as `{ "$numberDecimal": "…" }`, binaries as `{ "$binary": { base64, subType } }`. `Int32`, doubles and `Int64` within ±2^53 arrive as plain JSON numbers; a larger `Int64` stays `{ "$numberLong": "…" }`, at any depth, because a JavaScript number would change it.

**Table view** (`TableDocumentsView`, `composables/table/`)

- TanStack Table 9 with the features `documentTableFeatures` registers: sorting with the sorted row model and the sort functions `auto` picks from, column ordering, visibility, sizing and resizing.
- Columns are the union of the top-level fields of all documents: `_id` first, the rest alphabetically. A missing field is an empty cell.
- `formatCellValue` turns a value into text:
  - `$oid` gives the hex string, `$date` the ISO date (also the canonical `{ "$numberLong": ms }` form)
  - `$numberLong`, `$numberInt`, `$numberDecimal` and `$numberDouble` give the number
  - `$binary` gives `[Binary]`, `$regularExpression` gives `/pattern/options`, `$timestamp` gives its JSON
  - `null` gives `null`, other objects and arrays give JSON text
  
  `useFormattedValueCache` caches the text per document and field until the next result.
- Cells longer than 120 characters are cut with `…` and show the full text in a tooltip.
- Rows are virtualized with TanStack Virtual: fixed row height 33 px, 20 rows of overscan.
- A click on a header sorts by that column, ascending, descending, off (`sortFn: 'auto'`). Sorting runs in the web view over the loaded page only; the query tab's sort input sorts on the server.
- Columns can be dragged by their grip to a new position. The order is saved in `localStorage` under `monolens:columnOrder:<connection id>:<database>:<collection>` and merged with the current fields: saved ones keep their place, new ones are appended.
- Columns can be resized by dragging their right edge, between 60 and 500 px. The start width comes from the 90th percentile of the text lengths in the first 500 documents, or the header if wider. Resized widths are not saved.
- No documents: "No documents found for this query."

**JSON view** (`JSONDocumentsEditor`)

- One block per document, titled `Document <n> · _id: <_id as JSON>`, with a chevron to collapse it.
- Each open block is a read-only CodeMirror editor with JSON highlighting, line wrapping, the One Dark theme and the basic setup (line numbers, folding), at most 384 px high.
- A new result opens all blocks. Editors are created when a block opens and destroyed when it closes.

## Decisions

- **Table as the default view** (`ead8967`). The commit gives no reason.
- **Column order saved per connection, database and collection** (`ead8967`), so a reordered collection looks the same next time.
- **Virtual rows, cached cell text, sampled column widths** (`0aca486`, PR #44, issue #39), so bigger results render fast.
- **Relaxed extended JSON with exact large integers** (`5a30f74`): readable dates, and no silent change of an `Int64` beyond 2^53.
- **TanStack Table 9 with explicit features** (`3493da8`): v9 only has the APIs of registered features.

## Building steps

| Date | Step | Commit |
|---|---|---|
| 2026-02-25 | First display of the documents | `8722010` |
| 2026-02-26 | JSON view with one CodeMirror editor per document | `0c738f1` |
| 2026-03-02 | Table view: sortable and movable columns, formatting of `_id` and other extended JSON | `ead8967` |
| 2026-03-11 | Composables for drag and drop, sizing and the table; virtual rows; cached cell text; resizable columns | `0aca486` |
| 2026-11-03 | TanStack Table 9, tests for the table | `3493da8` |
| 2026-11-03 | Relaxed extended JSON from Rust, large `Int64` kept exact | `5a30f74` |
| 2026-11-03 | The JSON view's watcher guard that never returned removed | `d5ff9e3` |

## Limits and open points

- **Read-only.** Documents can't be added, edited or deleted. Open issues: #5 CRUD on data, #6 adding, #7 updating, #9 deleting, #43 a command pattern so changes can be undone.
- **Nested values are flat text.** An embedded document or array is one JSON string in its cell. Issue #39 asks to open nested objects on double click and for bigger text.
- Column widths are not saved, only the order.
- The JSON view shows the extended-JSON objects as they arrive (`{ "$oid": … }`), not shell notation.
- No tests for `formatCellValue` yet.

## Where the code is

| What | Path |
|---|---|
| Table | `src/components/documents/table/TableDocumentsView.vue` |
| Table logic | `src/composables/table/useDocumentTable.ts`, `useColumnSizing.ts`, `useColumnDragDrop.ts`, `useFormattedValueCache.ts` (tests in `test/`) |
| Cell text | `src/utils/formatCellValues.ts` |
| JSON view | `src/components/documents/json/JSONDocumentsEditor.vue` |
| Conversion in Rust | `src-tauri/src/drivers/documents.rs` |
