# Plans

A plan is researched against the code and written before the work starts, and gets a row under "Open" here. Its status column follows the decisions. Once it is built, it gets a status note at the top (when, which commits, what deviated) and moves to [implemented/](implemented/) with `git mv`. After that it is not updated any more: the feature files in [../features/](../features/) describe the present, the plan keeps the reasons.

A plan names the GitHub issues it covers.

## Open

| Plan | What | Status |
|---|---|---|
| [working-state.md](working-state.md) | Fix the bugs found on 2026-11-03, complete querying (paging, count, sort, projection), several connections at once, connection strings with options | Agreed 2026-11-03, being built on `fix/working_state` |

## Implemented

| Plan | Built | Now described in |
|---|---|---|
