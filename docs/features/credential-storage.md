# Credential storage

## Purpose

Passwords of saved connections must not sit in a plain file. Monolens keeps them in the operating system's keyring, and the Rust core loads them only when it needs them for a connection.

## How it works

- Passwords go through the `CredentialStore` trait. `CredentialService` is the OS keyring through `keyring` 4 (its default `v1` mode: Keychain on macOS, Credential Manager on Windows, Secret Service on Linux through `zbus`). The service name is `monolens-connections`, the entry `connection:<connection id>:basic_password`. Keyring calls run in `spawn_blocking`. A missing entry is `CredentialError::NotFound`, anything else `CredentialError::Keyring`.
- Tests use `MemoryCredentialStore`, which keeps passwords in memory and can be told to fail.
- Writing:
  - `create_connection` takes the password out of the request, writes it under the new id, and saves the connection without it.
  - A connection string with user info has it taken out first (`models/normalize.rs`): the username goes into `authentication`, the password to the keyring. Typed credentials win over the ones in the string.
  - `update_connection` writes the password for `BASIC` when one is sent (without one the stored password stays), and deletes it for `NONE`.
  - `delete_connection` deletes the entry and ignores keyring errors.
- Reading:
  - `get_connections` never returns a password.
  - `ConnectorRepository::hydrate` fills the password of a `BASIC` connection. `ConnectionHandler` calls it when it builds a driver, for every command that talks to MongoDB; those commands take a connection id, so the password never passes through the web view for them.
  - A keyring that can't be read is `DriverError::CredentialsUnavailable`: "The password of this connection could not be read from the system keyring. Edit the connection and enter it again."
  - `get_connection_password(id)` returns the password to the UI, or `null`. Only the edit dialog calls it, when it opens.
  - A connection test from the dialog sends the password typed in the form, because the connection may not be saved yet.
- Building the client (`drivers/client_options.rs`): a host target gets a `Credential` with username, password and the authentication database (default `admin`). For a connection string the user info is put back into the string, percent-encoded, only in memory and right before the driver parses it, because the driver checks some options against it (an `authMechanism` needs a username). The authentication database is the form's, else the string's `authSource`, else `admin`.
- Logging: the driver logs hosts and settings of a new client, never the credential. The connection dialog logs no form values.

## Decisions

- **OS keyring through `keyring`** (`eb8fdd9`, issue #15). Tauri Stronghold was added on 2026-02-15 (`8b54cd3`) and reverted two days later (`02a2c3a`). The revert says only that Stronghold "is not needed anymore".
- **Linux: keyutils plus Secret Service, OpenSSL vendored** (`aad2e90`), replaced on 2026-11-03 by keyring 4, which only uses the Secret Service. Passwords saved with keyring 3 are read by keyring 4 unchanged (checked with the three connections on the development machine).
- **Passwords stay in Rust** (`00b3284`, `84839a6`, PR #23), and since `119bf4e` the commands that connect load them by connection id.
- **Exception: the edit dialog shows the password** (`dc020e7`, issue #29). It is fetched only when the dialog opens.
- **A trait for the store** (`a7b4c1a`): the repository can be tested without the real keyring, and the commands no longer downcast the repository to reach it.
- **No credentials in a stored connection string** (`5f6efaa`): they are moved out on save and put back only in memory before parsing.

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
| 2026-11-03 | keyring 4: the Secret Service alone on Linux, no keyutils cache, no OpenSSL | `955c08f` |
| 2026-11-03 | `CredentialStore` trait, `hydrate` and `get_password` on the repository, no downcasts | `a7b4c1a` |
| 2026-11-03 | Credentials moved out of connection strings, `ClientOptions` instead of a URL built by hand | `5f6efaa` |
| 2026-11-03 | Passwords loaded by connection id in Rust, `CredentialsUnavailable` | `119bf4e` |

## Limits and open points

- The password reaches the web view in two places: the edit dialog loads it, and a test sends the typed one.
- Only username and password (MongoDB's default SCRAM mechanism). No X.509, LDAP, Kerberos, AWS or OIDC.
- On Linux the store is the Secret Service (e.g. GNOME Keyring or KWallet). How the app behaves without one was not tested.

## Where the code is

| What | Path |
|---|---|
| Store trait and keyring | `src-tauri/src/connector/repositories/credentials/credential_store.rs`, `credential_service.rs` |
| Writing, reading, hydrating | `src-tauri/src/connector/repositories/file_repository.rs` |
| Credentials in connection strings | `src-tauri/src/connector/models/uri.rs`, `normalize.rs` |
| The driver's credential | `src-tauri/src/drivers/client_options.rs` |
| Password for the edit dialog | `src-tauri/src/connector/commands/get_connection_password.rs`, `src/components/connection/ConnectionDialog.vue` |
