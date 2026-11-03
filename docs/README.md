# Monolens documentation

How to install and run Monolens is in the root [README.md](../README.md). This folder explains how it is built and why it is built that way.

The documentation was written on 2026-11-03 from `develop` (`859c252`), the commit history, the pull requests and the issues on GitHub, when the project was picked up again after a break since March 2026.

| Where | What you find there |
|---|---|
| [architecture/overview.md](architecture/overview.md) | The parts of the app, every IPC command, how a query travels, what the app stores where, the state of the build |
| [architecture/decisions.md](architecture/decisions.md) | Decision log: what was decided, when, why, and what was rejected |
| [features/](features/) | One file per feature, see below |
| [plans/](plans/) | Plans as they were written before the work started. Open ones in `plans/`, built ones in `plans/implemented/` with a status note at the top (implemented when, what deviated). [plans/README.md](plans/README.md) lists all of them with their status. The feature file is what stays current |
| [../CLAUDE.md](../CLAUDE.md) | Working guide: commands, conventions, gotchas |

## Features

| File | Feature |
|---|---|
| [connection-manager.md](features/connection-manager.md) | Saved connections: create, edit, delete, the start screen, the header dropdown, `connections.json` |
| [credential-storage.md](features/credential-storage.md) | Passwords in the OS keyring, loaded in Rust only when connecting |
| [connection-test.md](features/connection-test.md) | The staged connection test with live progress |
| [connect-and-explorer.md](features/connect-and-explorer.md) | Connecting, the driver pool, the sidebar with databases and collections |
| [background-tasks.md](features/background-tasks.md) | Cancellable tasks in Rust and `abortableInvoke` in the UI |
| [error-handling.md](features/error-handling.md) | `kind`, `message`, `user_message`, and the toasts |
| [tabs.md](features/tabs.md) | One tab per collection, tab history, kept-alive views |
| [query-tab.md](features/query-tab.md) | Shell-style filter with autocomplete, the query parser, find and count in Rust |
| [document-views.md](features/document-views.md) | The virtualized table and the JSON view of the results |
| [shortcuts.md](features/shortcuts.md) | The command registry and the key bindings |
| [logging.md](features/logging.md) | One log for Rust and the web view |
| [app-shell.md](features/app-shell.md) | Window, layout, dark theme, font, icons, packaging |

## What Monolens does not do yet

Monolens reads. Writing data, indexes, aggregation, import, export and statistics are not built. The open GitHub issues describe what was planned:

| Area | Issues |
|---|---|
| Writing data | #5 CRUD on documents, #6 add, #7 update, #9 delete, #43 command pattern with undo |
| Querying | #11 query builder, #41 queries by drag and drop, #13 export a query as code |
| Connections | #40 quick connections, #45 stage types with a guard for production, #28 in-memory MongoDB |
| Display | #39 table enhancements (nested objects, text size) |
| Bugs | #42 `Ctrl+Enter` runs in every tab |
| Other | #14 database statistics, #18 log file and rotation, #31 i18n |

The tab kinds `INDEXES`, `AGGREGATION`, `EXPORTER`, `IMPORTER` and `STATISTICS` already exist in the code as placeholders.

## Writing a feature file

Every file in `features/` has the same six sections, in this order:

1. **Purpose**: what the feature is for and who uses it, in a few sentences.
2. **How it works**: the flow, the data, the commands and the UI, as it is now.
3. **Decisions**: what was decided and why, including what was rejected. A decision that matters beyond the feature also gets a line in [architecture/decisions.md](architecture/decisions.md).
4. **Building steps**: the steps in the order they were built, each with date and commit hash.
5. **Limits and open points**: what the feature does not do, known bugs and risks, agreed follow-ups, the GitHub issues.
6. **Where the code is**: the files to start reading.

When a feature changes:

- Add a building step with the commit.
- Bring "How it works" up to date. It describes the present, not the history.
- Record a new decision in the file, and in the decision log if it reaches further.

Building steps are taken from the commit history. Where the history says little, the file says so and does not fill the gap. Points marked "read from the code" were not reproduced in the running app.
