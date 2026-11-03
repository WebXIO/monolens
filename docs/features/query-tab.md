# Query tab

## Purpose

In a collection's tab the user writes a filter in MongoDB shell syntax, e.g. `{ age: { $gt: 18 }, _id: ObjectId("…") }`, runs it and sees the matching documents. This is how Monolens reads data today.

## How it works

**The view** (`TabViewQuery`)

- A query input, an Execute button, a limit dropdown (10, 25, 50, 100, 200; default 50), a Refresh button and the switch between table and JSON view.
- When the view mounts it runs the query once. Execute, Refresh and `Ctrl+Enter` run it again.
- `execute()`:
  1. An empty input becomes `{}`.
  2. `parseMongoDbQuery` parses the input. A parse error is a toast, and nothing is sent.
  3. It invokes `start_find_documents` with the active connection, the tab's database and collection, the filter, `skip: 0` and the limit, then `await_task_result`.
  4. It stores `{ documents, totalCount }`.
  5. If documents came back, it takes the field names of the first one for the autocomplete.
- The documents go to the table or the JSON view, see [document-views.md](document-views.md).

**The query input** (`QueryInput`)

- A CodeMirror 6 editor, kept to one line, with the One Dark theme. A clear button (X) appears when the query is not empty and resets it to `{ }`.
- Autocomplete (`useMongoAutoComplete` in `src/domains/mongo`):
  - After a colon it offers the query operators (`$eq` … `$text`) and the shell functions (`ObjectId`, `ISODate`, `NumberLong`, `NumberDecimal`, `Timestamp`, `RegExp`), which insert their parentheses.
  - After `{` or `,` it offers the field names of the last result, the top-level operators `$and`, `$or`, `$nor`, `$text`, and the operators.

**The parser** (`src/utils/queryParser`)

- `generateTokens` turns the text into tokens: braces, brackets, parentheses, colon, comma, strings in `"` or `'` with backslash escapes, numbers (digits, `.`, a leading `-`), and identifiers (`[a-zA-Z_$][a-zA-Z0-9_$]*`).
- `createParser().parseValue()` reads objects, arrays, strings, numbers, `true`, `false`, `null` and function calls. Keys may be identifiers or strings. A trailing comma is allowed.
- A function call becomes extended JSON (`transformFunction`):

  | Shell | Sent as |
  |---|---|
  | `ObjectId("x")` | `{ $oid: "x" }` |
  | `ISODate("x")`, `Date("x")` | `{ $date: "x" }` |
  | `NumberLong(n)` | `{ $numberLong: "n" }` |
  | `NumberDecimal(n)` | `{ $numberDecimal: "n" }` |
  | `Timestamp(t, i)` | `{ $timestamp: { t, i } }` |
  | `BinData(type, "b64")` | `{ $binary: { base64, subType: type } }` |
  | `RegExp("p", "o")` | `{ $regex: "p", $options: "o" }` |

  Any other function is an error.

**In Rust** (`find_documents` of the MongoDB driver)

- `bson::to_document` turns the JSON filter into BSON. bson's serializer reads objects that look like extended JSON as BSON types, so `{ $oid }`, `{ $numberLong }`, `{ $numberDecimal }`, `{ $timestamp }` and `{ $date }` with an RFC 3339 string become real types.
- `find` with `skip`, `limit` and `allowDiskUse(true)`. The cursor is read into a list of JSON values.
- `count_documents` with the same filter gives `totalCount`.

## Decisions

- **A parser of our own** (`0c738f1`) for the shell syntax: unquoted keys, single quotes and shell functions, which JSON doesn't allow. The history gives no reason for not using a library.
- **Extended JSON as the format between UI and Rust.** The filter crosses IPC as JSON, and bson turns the extended-JSON objects into types without code of our own.
- **CodeMirror with autocomplete** (`9f677e1`).
- **Run on open**: a new tab shows the first documents at once.

## Building steps

| Date | Step | Commit |
|---|---|---|
| 2026-02-23 | `start_find_documents` with filter, skip and limit, as a task | `cbc9b83` |
| 2026-02-24 | `TabViewQuery`, proof of concept | `e0256f3` |
| 2026-02-25 | First display of the documents | `8722010` |
| 2026-02-26 | Query parser with tests, JSON view | `0c738f1` |
| 2026-03-01 | CodeMirror query input with autocomplete | `9f677e1` |
| 2026-03-02 | Header and query controls stay in place while the data scrolls | `0a88369` |
| 2026-03-02 | Clear button | `17ac29b` |
| 2026-03-02 | Table view, limit dropdown, view switch | `ead8967` |

## Limits and open points

Read from the code on 2026-11-03, not reproduced in the running app:

- **Only filter and limit.** `skip` is always 0, so there is no paging. Projection and sort are missing too, although issue #12 (closed) listed them. The Rust command already takes `skip`.
- **`totalCount` is computed but never shown.** Every query also counts all matches, a second full query on the server.
- **Server errors are swallowed.** `execute()` has `try`/`finally` without `catch`: a failed query shows no toast, the old results stay, and the rejection is unhandled. It also bypasses `ConnectionServiceIPC.findDocuments` and `abortableInvoke`, so it can't be cancelled and its error isn't unwrapped.
- **The autocomplete knows only the first document's fields.** `new Set(...arrays)` uses only its first argument.
- **Shell functions that don't become BSON types** stay plain objects and match nothing, without an error:
  - `ISODate("2024-01-01")`: bson only reads a full RFC 3339 string like `2024-01-01T00:00:00Z` as a date
  - `ObjectId` with a string that is not 24 hex characters
  - `BinData(0, "…")`: bson expects the subtype as a hex string
- `RegExp` becomes the `$regex` operator, which works as the condition of a field but not inside `$in`.
- **Not supported**: dotted keys without quotes (`user.name: 1` fails; `"user.name": 1` works), `/regex/` literals, `new Date()`, `NumberInt`, `UUID`, exponents (`1e5`), comments.
- Text after the first complete value is ignored: `{ a: 1 } }` runs as `{ a: 1 }`.
- `ConnectionServiceIPC.findDocuments` exists and is unused.
- The test named "throws" in `query-parser.test.ts` checks nothing: it parses `{ a: 1, }`, which succeeds.
- Open issues: #11 a query builder, #41 building queries by drag and drop, #13 export a query as code, #42 `Ctrl+Enter` runs in every tab.

## Where the code is

| What | Path |
|---|---|
| View | `src/components/tabs/views/TabViewQuery.vue` |
| Input and autocomplete | `src/components/query/QueryInput.vue`, `src/domains/mongo/index.ts` |
| Parser and tests | `src/utils/queryParser/`, `src/utils/queryParser/test/` |
| Command | `src-tauri/src/database/commands/start_find_documents.rs` |
| Find and count | `find_documents` in `src-tauri/src/drivers/mongo_official_driver.rs`, `src-tauri/src/database/models/find_document.rs` |
