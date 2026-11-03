# Keyboard shortcuts

## Purpose

Common actions have a key: close the tab, disconnect, run the query.

## How it works

- `CommandRegistry` (in `src/domains/shortcuts`, created once by the domain registry) maps command ids to `{ id, name, execute }`. `executeCommand(id)` runs the command, then notifies every subscriber of that id. `subscribe` and `unsubscribe` manage the subscribers. `disableExecution` and `enableExecution` switch all commands off and on; nothing calls them yet.
- The `shortcuts` plugin (`src/plugins/shortcuts.ts`) registers the commands with the stores and adds one global `keydown` listener. `eventToKeyString` builds a key string like `ctrl+shift+d`: modifiers in the order ctrl, shift, alt, meta, then the lowercase key. On a match with a default binding the listener calls `preventDefault()` and runs the command.
- The bindings (`DefaultBindings`):

  | Key | Command | What it does |
  |---|---|---|
  | `Ctrl+W` | `tab.close` | `tabsStore.closeActiveTab()` |
  | `Ctrl+Shift+D` | `connection.disconnect` | `connectionStore.disconnect()` |
  | `Ctrl+Enter` | `event.execute` | Nothing itself. `TabViewQuery` subscribes to it and runs its query |

## Decisions

- **Commands decoupled from keys** (`748c02a`, issue #22). A command has an id and a name, and the bindings are a separate list, so they can later be configured or listed in a command palette.
- **Events through the same registry.** `event.execute` is a command without its own action; components subscribe to it.

## Building steps

| Date | Step | Commit |
|---|---|---|
| 2026-02-17 | Registry, plugin, the three bindings | `748c02a` |
| 2026-02-24 | The query view subscribes to `Ctrl+Enter` | `e0256f3` |

## Limits and open points

- **`Ctrl+Enter` runs the query in every mounted query view**, not only in the active tab (issue #42). Every view instance subscribes when it mounts, and views in hidden tabs stay mounted, see [tabs.md](tabs.md#limits-and-open-points).
- **`unsubscribe` is broken.** It calls `subs.findIndex(subscriber)`, which uses the subscriber as the predicate. For the query view that calls `execute()`, whose promise is truthy, so the first subscriber is removed, whichever it is, and a query runs. It should be `indexOf`.
- The bindings are fixed. On macOS they need `Ctrl`, not `Cmd`, because `Cmd` reports as `meta`.
- The listener is global, so the keys also act while typing in an input.
- Nothing lists the shortcuts in the UI.

## Where the code is

| What | Path |
|---|---|
| Registry | `src/domains/shortcuts/registry/CommandRegistry.ts` |
| Commands and bindings | `src/domains/shortcuts/config/commands.ts`, `default-binding.ts`, `src/domains/shortcuts/models/` |
| Key string | `src/domains/shortcuts/utils/eventToKeyString.ts` |
| Listener | `src/plugins/shortcuts.ts` |
| Subscriber | `src/components/tabs/views/TabViewQuery.vue` |
