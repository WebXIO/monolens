# Monolens

**Monolens** is a free, open-source MongoDB GUI IDE — a secure and efficient European alternative to proprietary tools like Studio 3T. It provides a modern, responsive interface for managing MongoDB databases.  

Official homepage: [monolens.org](https://monolens.org)  

Licensed under the [Apache-2.0 License](https://www.apache.org/licenses/LICENSE-2.0).

---

## Tech Stack

- **Rust + Tauri** — high-performance backend with secure system-level access  
- **Vue 3 + TailwindCSS** — modern, responsive, and maintainable frontend  


## Features

Currently supported features include:  

- **Database Connections** — create, save, edit, and delete MongoDB connections  
- **Credential Handling** — secure storage of credentials using the OS-native keyring  
- **Extensible Architecture** — designed to easily add new features, such as data querying, visualization, and advanced operations  

But there are many more to come!


## Development

1. **Install dependencies**  
```bash
pnpm install
```
2. **Make sure Rust is installed on your machine** [installation guide](https://rust-lang.org/tools/install/)
3. **Additionally install Rust Tauri** [installation guide](https://tauri.app/start/prerequisites/)
4. **Run in development mode**
```bash
pnpm tauri dev
```

How Monolens is built, why, and what is planned is documented in [docs/](docs/README.md).

## Knowledge graph for Claude Code (graphify)

[graphify](https://github.com/Graphify-Labs/graphify) gives Claude Code a graph of the code and the docs, which
it queries before searching files: the skill is in `.claude/skills/graphify/`, the hooks in `.claude/settings.json`,
the rules at the end of `CLAUDE.md`. The graph (`graphify-out/`) is built on each machine and not committed. Only
`graphify-out/cache/semantic/` is: what the model read from the docs, so a build pays only for docs that changed.
`.graphifyignore` keeps `docs/plans/`, generated code, icons and the lockfiles out of the graph.

Once per clone (pipx: `sudo apt install pipx`, then `pipx ensurepath`):

```bash
pipx install graphifyy==0.9.92
```

```bash
graphify hook install
```

Then build the graph in a Claude Code session in the repo with `/graphify .`. Everyone uses the same version: an
upgrade can invalidate the stored doc extraction. The git hooks rebuild the code part after commit, checkout,
merge and rebase, without model calls. After changing docs, run `/graphify . --update` in a session and commit the
new files in `graphify-out/cache/semantic/`. Without graphify installed the hooks fail and Claude works as before.