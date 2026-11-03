# Plans

A plan is researched against the code and written before the work starts, and gets a row under "Open" here. Its status column follows the decisions. Once it is built, it gets a status note at the top (when, which commits, what deviated) and moves to [implemented/](implemented/) with `git mv`. After that it is not updated any more: the feature files in [../features/](../features/) describe the present, the plan keeps the reasons.

A plan names the GitHub issues it covers.

## Open

| Plan | What | Status |
|---|---|---|

## Implemented

| Plan | Built | Now described in |
|---|---|---|
| [working-state.md](implemented/working-state.md) | 2026-11-03: the bugs of the documentation fixed, several connections at once, connection strings with options, paging, count, sort and projection; plus a full package update | [connection-manager.md](../features/connection-manager.md), [connect-and-explorer.md](../features/connect-and-explorer.md), [query-tab.md](../features/query-tab.md) and the other feature files |
