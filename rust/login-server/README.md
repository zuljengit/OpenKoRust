# Login Server (Rust)

A Rust rewrite of the Knight Online 1.298 VersionManager server. Handles client version checking, patching, authentication, server list and login-screen news.

Originally part of the [OpenKO](https://github.com/Open-KO/KnightOnline) project (C++), this is a standalone Rust reimplementation that speaks the same wire protocol, so the client can't tell the difference.

## Prerequisites

- [Rust toolchain](https://rustup.rs/) (rustup + cargo)
- Windows: MSVC build tools (included with Visual Studio)

## Project structure

```
login-server/
  assets/login-server.ico   Icon embedded into the exe
  src/main.rs                All server logic
  build.rs                   Build script (embeds the icon on Windows)
  Cargo.toml                 Dependencies and project config
```

## How to run (development)

```
cargo run
```

This compiles and runs in debug mode. You should see:

```
VersionManager (Rust) listening on 0.0.0.0:15100
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

This is a standalone binary — copy it anywhere, no Rust installation needed to run it.

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
