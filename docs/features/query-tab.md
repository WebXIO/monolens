# Query tab

## Purpose

In a collection's tab the user writes a filter, a sort and a projection in MongoDB shell syntax, e.g. `{ age: { $gt: 18 }, _id: ObjectId("…") }`, runs it and pages through the matching documents. This is how Monolens reads data today.

## How it works

**The view** (`TabViewQuery`)

- Inputs for filter (default `{ }`), sort and projection, an Execute button that turns into Cancel while a query runs, a limit dropdown (10, 25, 50, 100, 200; default 50), Refresh (this page again), the switch between table and JSON view, and the page navigation: "51–100 of 1,234" with previous and next. A total with "~" is an estimate.
- When the view mounts it runs the query once. Execute, `Mod+Enter` (in the active tab only) and a new limit run what is typed, from the first page; a new limit only takes effect when the inputs parse. Previous, next and Refresh repeat the query that was run last, with another skip; what is typed since doesn't count until it is executed.
- `parseQueryTexts` parses the three inputs (`queryState.ts`). An empty input or `{ }` means none. A parse error names the input ("Sort: Unexpected token: EOF") and nothing is sent.
- The query goes through `ConnectionServiceIPC.findDocuments(connectionId, query)` as a cancellable task; a new query cancels one still running. A failed query is a toast with the server's message; the old results stay.
- The autocomplete knows the fields of all documents of the last result.
- The documents go to the table or the JSON view, see [document-views.md](document-views.md).

**The query input** (`QueryInput`)

- A CodeMirror 6 editor, kept to one line, with the One Dark theme and an optional placeholder. A clear button (X) appears when the input is not empty and resets it to `{ }`.
- Autocomplete (`useMongoAutoComplete` in `src/domains/mongo`): after a colon the query operators and the shell functions; after `{` or `,` the field names, the top-level operators `$and`, `$or`, `$nor`, `$text`, and the operators.

**The parser** (`src/utils/queryParser`)

- `generateTokens`: braces, brackets, parentheses, colon, comma, strings in `"` or `'` with backslash escapes, numbers with decimals and exponents, regex literals `/pattern/flags`, identifiers including dots (`user.address.city` as an unquoted key). An unterminated string or regex is an error.
- `createParser().parseQuery()` reads objects, arrays, strings, numbers, regexes, `true`, `false`, `null` and function calls, `new` in front of a function is ignored. Keys may be identifiers or strings, a trailing comma is allowed, a missing comma is an error, and nothing may follow the value.
- Shell functions become extended JSON that bson in Rust reads as BSON types; every function checks its arguments, so nothing silently becomes a plain object (`transformFunction`):

  | Shell | Sent as | Checked |
  |---|---|---|
  | `ObjectId("x")` | `{ $oid }`, lowercase | 24 hex characters |
  | `ISODate("x")`, `Date("x")`, `new Date(…)` | `{ $date: "<ISO>" }` | a date or date-time without zone is UTC, with `T` or a space between; milliseconds; no argument is now; invalid is an error |
  | `NumberLong(n)`, `NumberInt(n)` | `{ $numberLong }`, `{ $numberInt }` | integers, NumberInt within 32 bits |
  | `NumberDecimal(n)` | `{ $numberDecimal }` | a number |
  | `Timestamp(t, i)` | `{ $timestamp: { t, i } }` | non-negative integers |
  | `BinData(type, "b64")`, `UUID("…")` | `{ $binary: { base64, subType } }` (hex; UUID subtype `04`) | subtype 0–255, base64, 32 hex characters |
  | `RegExp("p", "o")`, `/p/o` | `{ $regularExpression: { pattern, options } }` | options from `i m s x u`, sorted |

**In Rust**: `start_find_documents(connectionId, query)` runs find (filter, sort, projection, skip, limit 1–10000, `allowDiskUse`) and the count at the same time: `estimatedDocumentCount` for an empty filter (marked as estimate), `countDocuments` otherwise. Documents return as relaxed extended JSON, see [document-views.md](document-views.md).

## Decisions

- **A parser of our own** (`0c738f1`) for the shell syntax: unquoted keys, single quotes and shell functions, which JSON doesn't allow. The history gives no reason for not using a library.
- **Extended JSON between UI and Rust.** The query crosses IPC as JSON, and bson turns the extended-JSON objects into types. Since `094ed26` the parser checks every function, because what bson can't read silently matched nothing (e.g. `ISODate("2024-01-01")`, which bson only reads as a full RFC 3339 date).
- **A regex is a BSON regex** (`094ed26`), not the `$regex` operator, so it also works inside `$in`.
- **CodeMirror with autocomplete** (`9f677e1`).
- **Run on open**: a new tab shows the first documents at once.
- **Paging repeats the last executed query** (`16b6cd7`), so typing doesn't change what the next page means.
- **An empty filter is counted from metadata** (`5a30f74`): counting a big collection with `countDocuments` scans it. The UI marks the number as estimate.

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
| 2026-11-03 | Parser: checked shell functions, UTC dates, regex literals, `NumberInt`, `UUID`, `new`, dotted keys, exponents, nothing after the query | `094ed26` |
| 2026-11-03 | `Mod+Enter` only in the active tab; each tab queries its own connection | `95a3e7b`, `afd84c7` |
| 2026-11-03 | Rust: one `FindQuery` with sort and projection, the count alongside, the server's message on errors | `5a30f74` |
| 2026-11-03 | Sort and projection inputs, paging with the total, cancel, error toasts, autocomplete from all documents | `16b6cd7` |

## Limits and open points

- Find only: no aggregation, no `explain`, no `$where` with JavaScript functions, no comments in the inputs, no `NumberDouble`, `MinKey` or `MaxKey`.
- The estimate of an unfiltered collection may be off, e.g. after an unclean shutdown or on a sharded cluster.
- Open issues: #11 a query builder, #41 building queries by drag and drop, #13 export a query as code.

## Where the code is

| What | Path |
|---|---|
| View | `src/components/tabs/views/TabViewQuery.vue` |
| Parsing the inputs, paging | `src/components/query/queryState.ts` (tests in `src/components/query/test/`) |
| Input and autocomplete | `src/components/query/QueryInput.vue`, `src/domains/mongo/index.ts` |
| Parser and tests | `src/utils/queryParser/`, `src/utils/queryParser/test/` |
| Command | `src-tauri/src/database/commands/start_find_documents.rs`, `FindQuery` in `src-tauri/src/database/models/find_document.rs` |
| Find, count, conversion | `find_documents` in `src-tauri/src/drivers/mongo_official_driver.rs`, `src-tauri/src/drivers/documents.rs` |
