# Login Server (Rust)

A Rust rewrite of the Knight Online 1.298 VersionManager server. Handles client version checking, patching, authentication, server list and login-screen news.

Originally part of the [OpenKO](https://github.com/Open-KO/KnightOnline) project (C++), this is a standalone Rust reimplementation that speaks the same wire protocol, so the client can't tell the difference.

## Prerequisites

- [Rust toolchain](https://rustup.rs/) (rustup + cargo)
- Windows: MSVC build tools (included with Visual Studio)
- SQL Server with the KN_online 1.298 database schema

## Project structure

```
login-server/
  assets/login-server.ico   Icon embedded into the exe
  config.toml                Server configuration
  src/
    main.rs                  Entry point, accept loop, background tasks
    config.rs                Configuration and state structs
    protocol.rs              Packet framing, wire format, opcode handlers
    db.rs                    Database access with connection pooling
  build.rs                   Build script (embeds the icon on Windows)
  Cargo.toml                 Dependencies and project config
```

## How to run (development)

```
cargo run
```

This compiles and runs in debug mode. You should see:

```
Login Server listening on: 0.0.0.0:15100
```

The server listens on TCP port 15100 and accepts KO client connections.

## How to build the exe (release)

```
cargo build --release
```

The optimized exe is at:

```
target/release/login-server.exe
```

Place `config.toml` next to the exe before running it.

This is a standalone binary. Copy it anywhere, no Rust installation needed to run it.

## Configuration

Edit `config.toml` to configure the server:

```toml
[general]
listen_port = 15100
last_version = 1298

[download]
ftp_url = "127.0.0.1"
ftp_path = "/"

[[servers]]
id = 1
ip = "127.0.0.1"
name = "Server 1"
user_limit = 3000

[news]
title = "Login Notice"
message = "Welcome to Knight Online!"

[database]
host = "localhost"
port = 1433
username = "knight"
password = "knight"
database = "KN_online"
```

Multiple game servers can be listed by repeating the `[[servers]]` block. The `id` field must match the `serverid` column in the CONCURRENT database table for player counts to display correctly.

## Features

- All five login server opcodes implemented (version check, download info, login, server list, news)
- SQL Server authentication via TDS protocol (no ODBC dependency)
- Connection pooling with bb8 (reuses database connections instead of opening one per request)
- Periodic player count updates from the CONCURRENT table (every 30 seconds)
- Robust packet framing with an accumulator buffer that handles TCP fragmentation, merged packets, garbage data and oversized payloads
- Configurable via a single TOML file

## Code quality

Format the code according to the Rust standard style:

```
cargo fmt
```

Run the linter to catch common mistakes and get suggestions for more idiomatic code:

```
cargo clippy
```

Run the unit tests:

```
cargo test
```

Generate a test coverage report:

```
cargo tarpaulin --out html
```

## Protocol

This server implements the KO 1.298 login/version protocol on port 15100:

| Opcode | Hex  | Description                        |
|--------|------|------------------------------------|
| LS_VERSION_REQ      | 0x01 | Returns the latest client version |
| LS_DOWNLOADINFO_REQ | 0x02 | Returns patch file list via FTP   |
| LS_LOGIN_REQ        | 0xF3 | Account authentication            |
| LS_SERVERLIST       | 0xF5 | Returns server list + populations |
| LS_NEWS             | 0xF6 | Returns login-screen news         |

Packet framing: `0xAA 0x55 [length: u16 LE] [payload] 0x55 0xAA`

## Related

The other server components (AIServer, Ebenezer, Aujard) remain in C++ and are built separately via Visual Studio using `Server.slnx`.
