# Project Dawn Core

A 3D MMO built with Bevy and lightyear. The server is authoritative: clients
send requests, and the server decides what happens.

## Getting started

You need Rust, the Visual Studio C++ build tools, `just`, `cargo-nextest`, and
`cargo-llvm-cov`.

```bash
just dev
```

This builds everything, starts the server, and opens two client windows. Move
with WASD or the arrow keys. Your character is blue and other players are
orange. Press Ctrl-C or close every client window to stop.

The first build compiles the whole engine, so it takes a while.

## Commands

| Command | What it does |
|---|---|
| `just dev` | Runs the server and two clients |
| `just dev 4` | Runs the server and four clients |
| `just sandbox movement` | Runs one game module without the full game |
| `just test` | Runs every test |
| `just test-unit` | Runs only the unit tests |
| `just test-integration` | Runs the real server or client against a fake other side |
| `just test-e2e` | Runs the real server and clients together |
| `just coverage` | Fails if tests cover under 90% of changed lines |
| `just check` | Runs formatting, architecture, test format, lint, and isolated-build checks |
| `just fmt` | Formats the code |

## Layout

```text
crates/
  foundation/    Identifiers and settings used across the game
  config/        Environment settings and secrets
  protocol/      Registers every game module's network types
  movement/      One game module, with src/client/ and src/server/ folders
  server/        Headless server: assembles plugins, owns connections
  client/        Game client: assembles plugins, owns the world view
  test_support/  Fake client, stub server, and test helpers
  e2e/           Tests that run a real server and clients together
libs/
  bdd/           The #[spec] macro for given, when, and test steps
  sandbox_kit/   Shared window and scene for each module's examples/sandbox.rs
xtask/           Development launcher and project checks
```

See [CONTRIBUTING.md](CONTRIBUTING.md) to make a change, and
[docs/architecture.md](docs/architecture.md) for where new code belongs.
AI coding agents should start with [AGENTS.md](AGENTS.md).

## Tests

There are unit, integration, and end-to-end tests. See the testing
levels in [docs/architecture.md](docs/architecture.md).

## Configuration

`just` commands run in development with no setup. To set real values, copy
`.env.example` to `.env`, which Git ignores. Servers default to production and refuse
to start without a real `DAWN_PRIVATE_KEY`. See
[docs/architecture.md](docs/architecture.md).
