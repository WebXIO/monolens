# App shell

## Purpose

The frame around the features: the window, the layout, the theme, the font, the icons and the packages it is built from.

## How it works

- **Window**: one window titled "Monolens", 1200 × 900, app identifier `org.monolens`, version 0.1.0. `tauri.conf.json` sets no CSP (`"csp": null`). Bundling covers all targets of the platform. On macOS the app builds its own menu, see [shortcuts.md](shortcuts.md).
- **Layout** (`App.vue`):
  - A header with the logo and, while a connection is open, the "Connections" dropdown. It stays at the top while the data scrolls.
  - Below it either `ConnectionGrid` (no connection open) or `AppSidebar` next to `TabsContainer`, inside shadcn-vue's `SidebarProvider` and `SidebarInset`.
  - The toasts and one `ConfirmDialog` for the whole app.
- **Theme**: dark only. The palette in `src/styles/default.css` is set on `:root` in OKLCH, with a green primary and dark greys, plus `success`, `warning` and `info`. A `.dark` block exists but no element gets the class. Tailwind 4 through `@tailwindcss/vite`, animations from `tw-animate-css`.
- **Components**: shadcn-vue (`components.json`: style `new-york`, base colour `neutral`, CSS variables, lucide icons through `@lucide/vue`) in `src/components/ui/`, built on reka-ui; `field` comes from the `new-york-v4` registry. They are generated with the shadcn-vue CLI and then belong to the repo.
- **Font**: Roboto 400, 500 and 700, bundled as TTF in `src/assets/fonts/roboto` (OFL licence next to it).
- **Icons**: generated with `pnpm tauri icon` from the Monolens logo (`monolens.png` in the root) for desktop, iOS and Android.
- **Toasts**: `vue-sonner`, dark, top centre, rich colours, see [error-handling.md](error-handling.md).
- **Packages** (2026-11-03): Tauri 2.12, Vue 3.5, Pinia 4, Vite 8, TypeScript 6 (vue-tsc can't run on 7), vitest 5 in happy-dom, TanStack Table 9 and Form 1, zod 4, pnpm 12 pinned in `package.json`.

## Decisions

- **Bundled Roboto** (`226eceb`, issue #30): the built app looked wrong with the system font.
- **1200 × 900** (`b35246a`): the history gives no reason.
- **shadcn-vue on Tailwind 4**: the history gives no reason. The README names Vue 3 and Tailwind.
- **All packages to their latest release** (`3493da8`, `955c08f`), except TypeScript 7 (vue-tsc can't run on it) and Tauri 3 (an alpha). `lucide-vue-next` is deprecated and was replaced by `@lucide/vue`.
- **pnpm pinned** (`e19ec32`) in `packageManager` and `devEngines`, as in smtp-relay. pnpm 12 refuses build scripts that `pnpm-workspace.yaml` doesn't allow.

## Building steps

| Date | Step | Commit |
|---|---|---|
| 2026-02-02 | Tauri, Vue and Vite template | `3f2376f` |
| 2026-02-15 | Build fixed for deb, rpm and AppImage | `7ee4b70` |
| 2026-02-17 | App icons from the logo | `85894cb` |
| 2026-02-18 | Window 1200 × 900 | `b35246a` |
| 2026-02-20 | Icons with a background | `e00aa25` |
| 2026-02-21 | New styling of the connection screens, stepper | `c780905`, `9229c45` |
| 2026-02-23 | Toasts, Roboto | `37819e1`, `226eceb` |
| 2026-03-02 | Sticky header | `0a88369` |
| 2026-11-03 | `pnpm build` works again (an unused import in a test broke it) | `c25ace4` |
| 2026-11-03 | pnpm 12 pinned; npm packages and Rust crates updated | `e19ec32`, `3493da8`, `955c08f` |
| 2026-11-03 | macOS menu without "Close Window" | `95a3e7b` |
| 2026-11-03 | Confirmation dialog | `29aceb2` |
| 2026-11-03 | Template leftovers removed: Vite favicon and SVGs, title "Monolens" | `d5ff9e3` |
| 2026-11-03 | Vite config on `import.meta.dirname` | `3c38991` |
| 2026-11-03 | CI: tests and build on Linux, macOS and Windows | `b69ba0b` |

## Limits and open points

- CI tests and builds on Linux, macOS and Windows, but the app has only been run on Linux. On macOS the Keychain may ask for access again after every rebuild of an unsigned app, and on Windows the web view may keep browser keys such as Ctrl+R (reload, which closes all tabs); both are unchecked.
- No release pipeline, no code signing, no auto-update.
- No CSP. The web view only loads the bundled UI, but a CSP would limit the damage of injected content.
- Dark theme only. No settings page.
- `tauri-plugin-opener` is registered but the UI doesn't use it yet.
- Open issues: #31 i18n.

## Where the code is

| What | Path |
|---|---|
| Window, bundle, menu | `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`, `src-tauri/src/menu.rs` |
| CI | `.github/workflows/ci.yml` |
| Layout | `src/App.vue`, `src/components/Header.vue`, `src/components/Logo.vue`, `src/components/ConfirmDialog.vue` |
| Theme and font | `src/styles/default.css`, `src/assets/fonts/roboto/` |
| UI kit | `src/components/ui/`, `components.json`, `src/lib/utils.ts` |
| Packages | `package.json`, `pnpm-workspace.yaml`, `src-tauri/Cargo.toml` |
| Icons | `src-tauri/icons/`, `monolens.png` |
