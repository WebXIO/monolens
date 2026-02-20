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