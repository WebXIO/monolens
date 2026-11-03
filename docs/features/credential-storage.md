# Credential storage

## Purpose

Passwords of saved connections must not sit in a plain file. Monolens keeps them in the operating system's keyring, and the Rust core loads them only when it needs them for a connection.

## How it works

- `CredentialService` wraps `keyring::Entry`. The service name is `monolens-connections` and the entry is `connection:<connection id>:basic_password`. Set, get and delete run in `spawn_blocking`, because the keyring calls block.
- The `keyring` crate (4.x, its default `v1` mode) uses the native store of each platform: Keychain on macOS, the Credential Manager on Windows, the Secret Service on Linux (through `zbus`, pure Rust, no OpenSSL).
- Writing:
  - `create_connection` takes the password out of the request, writes it to the keyring under the new id, and saves the connection without it.
  - `update_connection` writes the password for `BASIC` when one is sent, and deletes the entry for `NONE`. The file never gets the password.
  - `delete_connection` deletes the entry and ignores keyring errors.
- Reading:
  - `get_connections` and `get_connection` never return a password.
  - `hydrate_connection_with_credentials` loads the password for a `BASIC` connection right before use. `start_connect`, `start_find_documents` and the unused `get_databases` call it. The commands reach the `CredentialService` by downcasting the repository to `FileRepository`.
  - `list_collections` doesn't load the password. It relies on the driver that `start_connect` already put in the pool.
  - `get_connection_password(id)` returns the password to the UI, or `null` on any keyring error. Only the edit dialog calls it, when it opens.
  - A connection test from the dialog sends the password typed in the form, not the stored one.
- Building the connection string (`MongoDbOfficialDriver.build_connection_string`):
  - `Url::set_username` and `set_password` percent-encode the credentials. The username falls back to `root` if it is missing, but the dialog requires one.
  - `authSource` is the authentication database, else `admin`. It is also added to connections without authentication.
- Logging: only in debug builds the built URI is logged, with the password replaced by `***`. Release builds don't log it.

## Decisions

- **OS keyring through `keyring`** (`eb8fdd9`, issue #15). Tauri Stronghold was added on 2026-02-15 (`8b54cd3`) and reverted two days later (`02a2c3a`). The revert says only that Stronghold "is not needed anymore".
- **Linux: keyutils plus Secret Service, OpenSSL vendored** (`aad2e90`), replaced on 2026-11-03 by keyring 4, which only uses the Secret Service. Passwords saved with keyring 3 are read by keyring 4 unchanged (checked with the three connections on the development machine).
- **Passwords stay in Rust** (`00b3284`, `84839a6`, PR #23). Listing doesn't touch the keyring, and only the commands that connect load a password.
- **Exception: the edit dialog shows the password** (`dc020e7`, issue #29). Users wanted to see what they saved. It is fetched only when the dialog opens.
- **Percent-encoding through `Url`** (`8a9638a`, issue #21). Passwords with special characters broke the connection string.

## Building steps

| Date | Step | Commit |
|---|---|---|
| 2026-02-05 | Basic authentication in the connection model | `4100b21` |
| 2026-02-15 | Credentials percent-encoded in the URI | `8a9638a` |
| 2026-02-15 | Stronghold tried | `8b54cd3` |
| 2026-02-17 | Stronghold reverted, passwords in the OS keyring | `02a2c3a`, `eb8fdd9` |
| 2026-02-18 | Linux backend with keyutils and Secret Service, vendored OpenSSL | `aad2e90` |
| 2026-02-18 | Password loaded only on connect, never returned when listing | `00b3284`, `84839a6` |
| 2026-02-20 | The edit dialog shows the stored password | `dc020e7` |
| 2026-02-22 | Credentials removed from logs, URI logged only in debug builds | `8d6bcac` |
| 2026-11-03 | keyring 4: the Secret Service alone on Linux, no keyutils cache, no OpenSSL | `chore: update rust crates` |

## Limits and open points

- The password reaches the web view in two places: the edit dialog loads it, and a test sends the typed one. Everything else keeps it in Rust.
- A keyring failure while connecting becomes `ConnectionFailed("Could not fetch credentials")`, which the user sees as "Failed to connect. Verify your host, port, and that the server is running." That points to the wrong cause. The same happens when the file is copied to another machine without its keyring entries.
- Only username and password (MongoDB's default SCRAM mechanism). No X.509, LDAP, Kerberos, AWS or OIDC.
- On Linux the persistent store is the Secret Service (e.g. GNOME Keyring or KWallet). How the app behaves without one was not tested.
- The commands downcast the repository to `FileRepository`. Another repository implementation would make connect and find fail with "Unsupported repository type".

## Where the code is

| What | Path |
|---|---|
| Keyring access | `src-tauri/src/connector/repositories/credentials/credential_service.rs` |
| Loading the password before use | `src-tauri/src/connector/repositories/credentials/hydrate_connection_with_credentials.rs` |
| Writing and deleting | `src-tauri/src/connector/repositories/file_repository.rs` |
| Password for the edit dialog | `src-tauri/src/connector/commands/get_connection_password.rs`, `src/components/connection/ConnectionDialog.vue` |
| Connection string and log masking | `src-tauri/src/drivers/mongo_official_driver.rs` |
| Keyring features | `src-tauri/Cargo.toml` |
