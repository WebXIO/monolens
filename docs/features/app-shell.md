# App shell

## Purpose

The frame around the features: the window, the layout, the theme, the font and the icons.

## How it works

- **Window**: one window titled "Monolens", 1200 × 900, app identifier `org.monolens`, version 0.1.0. `tauri.conf.json` sets no CSP (`"csp": null`). Bundling covers all targets of the platform.
- **Layout** (`App.vue`):
  - A header with the logo and, while connected, the connection dropdown. It stays at the top while the data scrolls.
  - Below it either `ConnectionGrid` (no connection active) or `AppSidebar` next to `TabsContainer`, inside shadcn-vue's `SidebarProvider` and `SidebarInset`.
- **Theme**: dark only. The palette in `src/styles/default.css` is set on `:root` in OKLCH, with a green primary and dark greys. It also defines `success`, `warning` and `info`, which the default shadcn palette lacks. A `.dark` block exists but no element gets the class. Tailwind 4 through `@tailwindcss/vite`, animations from `tw-animate-css`.
- **Components**: shadcn-vue (`components.json`: style `new-york`, base colour `neutral`, CSS variables, lucide icons) in `src/components/ui/`, built on reka-ui. They are generated with the shadcn-vue CLI and then belong to the repo.
- **Font**: Roboto 400, 500 and 700, bundled as TTF in `src/assets/fonts/roboto` (OFL licence next to it).
- **Icons**: generated with `pnpm tauri icon` from the Monolens logo (`monolens.png` in the root) for desktop, iOS and Android, later with a background.
- **Toasts**: `vue-sonner`, dark, top centre, rich colours, see [error-handling.md](error-handling.md).

## Decisions

- **Bundled Roboto** (`226eceb`, issue #30): the built app looked wrong with the system font.
- **1200 × 900** (`b35246a`): the history gives no reason.
- **shadcn-vue on Tailwind 4**: the history gives no reason. The README names Vue 3 and Tailwind.

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

## Limits and open points

- `pnpm build`, and with it `pnpm tauri build`, fails on `develop`: an unused import in `src/utils/queryParser/test/tokens.test.ts` (TS6133). See [overview.md](../architecture/overview.md#building-and-running).
- No CI, no release pipeline, no code signing, no auto-update.
- No CSP. The web view only loads the bundled UI, but a CSP would limit the damage of injected content.
- Dark theme only. No settings page.
- Leftovers of the template: `index.html` uses `/vite.svg` as favicon and the title `monolens`. `public/vite.svg`, `public/tauri.svg` and `src/assets/vue.svg` are unused. `tauri-plugin-opener` is registered but the UI doesn't use it.
- The root README lists only connections and credentials as features.
- Open issues: #31 i18n.

## Where the code is

| What | Path |
|---|---|
| Window and bundle | `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json` |
| Layout | `src/App.vue`, `src/components/Header.vue`, `src/components/Logo.vue` |
| Theme and font | `src/styles/default.css`, `src/assets/fonts/roboto/` |
| UI kit | `src/components/ui/`, `components.json`, `src/lib/utils.ts` |
| Icons | `src-tauri/icons/`, `monolens.png` |
