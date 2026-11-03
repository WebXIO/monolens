# Keyboard shortcuts

## Purpose

Common actions have a key: close the tab, disconnect, run the query.

## How it works

- `CommandRegistry` (in `src/domains/shortcuts`, created once by the domain registry) maps command ids to `{ id, name, execute }`. `executeCommand(id)` runs the command, then notifies every subscriber of that id. `subscribe` and `unsubscribe` manage the subscribers. `disableExecution` and `enableExecution` switch all commands off and on; nothing calls them yet.
- The `shortcuts` plugin (`src/plugins/shortcuts.ts`) registers the commands with the stores and adds one global `keydown` listener. `eventToKeyString` builds a key string like `mod+shift+d`: `mod` is the platform's command key (Cmd on macOS, Ctrl elsewhere), then `ctrl` (macOS) or `meta` (elsewhere), shift, alt, and the lowercase key. On a match with a default binding the listener calls `preventDefault()` and runs the command.
- The bindings (`DefaultBindings`):

  | Key | Command | What it does |
  |---|---|---|
  | `Mod+W` | `tab.close` | `tabsStore.closeActiveTab()` |
  | `Mod+Shift+D` | `connection.disconnect` | disconnects the connection of the active tab, closing its tabs |
  | `Mod+Enter` | `event.execute` | Nothing itself. Every `TabViewQuery` subscribes and runs its query only if its tab is active |

- On macOS, Tauri's default menu binds `Cmd+W` to "Close Window". `src-tauri/src/menu.rs` builds that menu without it (app menu, Edit for copy and paste, View, Window with minimize and maximize), and `lib.rs` uses it on macOS only.

## Decisions

- **Commands decoupled from keys** (`748c02a`, issue #22). A command has an id and a name, and the bindings are a separate list, so they can later be configured or listed in a command palette.
- **Events through the same registry.** `event.execute` is a command without its own action; components subscribe to it.
- **`mod` instead of `ctrl`** (`95a3e7b`): Mac users expect Cmd. Without the menu change `Cmd+W` would close the app.

## Building steps

| Date | Step | Commit |
|---|---|---|
| 2026-02-17 | Registry, plugin, the three bindings | `748c02a` |
| 2026-02-24 | The query view subscribes to `Ctrl+Enter` | `e0256f3` |
| 2026-11-03 | `unsubscribe` with `indexOf` (it used `findIndex`, which called the subscriber and removed the first one); `mod` key; macOS menu without "Close Window"; tests | `95a3e7b` |
| 2026-11-03 | `Mod+Enter` only in the active tab (issue #42); `Mod+Shift+D` disconnects the active tab's connection | `afd84c7` |

## Limits and open points

- The bindings are fixed. Nothing lists them in the UI.
- The listener is global, so the keys also act while typing in an input.
- The `Cmd` keys and the macOS menu were not checked on a Mac.

## Where the code is

| What | Path |
|---|---|
| Registry | `src/domains/shortcuts/registry/CommandRegistry.ts` |
| Commands and bindings | `src/domains/shortcuts/config/commands.ts`, `default-binding.ts`, `src/domains/shortcuts/models/` |
| Key string | `src/domains/shortcuts/utils/eventToKeyString.ts` |
| Listener | `src/plugins/shortcuts.ts` |
| macOS menu | `src-tauri/src/menu.rs`, `src-tauri/src/lib.rs` |
| Tests | `src/domains/shortcuts/test/` |
