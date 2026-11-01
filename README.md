# Project Dawn Core

A 3D MMO built with Bevy and lightyear. The server is authoritative: clients
send requests, and the server decides what happens.

## Getting started

You need Rust, the Visual Studio C++ build tools, `just`, `cargo-nextest`, and
`cargo-llvm-cov`.

```bash
just dev
```

This builds everything, starts the server, and opens two client windows. Press Ctrl-C or close every client window to stop.

| Control | Action |
|---|---|
| WASD or arrow keys | Move, relative to the camera |
| Left Ctrl | Dodge roll in the direction you're moving |
| C | Switch sneaking on or off |
| Space | Jump |
| Hold right mouse button | Look around |
| Mouse wheel | Zoom |

The first build compiles the whole engine, so it takes a while.

## Commands

| Command | What it does |
|---|---|
| `just dev` | Runs the server and two clients |
| `just dev 4` | Runs the server and four clients |
| `just sandbox movement` | Runs one game module without the full game. Try `world` or `avatar` too |
| `just test` | Runs every test |
| `just test-unit` | Runs only the unit tests |
| `just test-integration` | Runs the real server or client against a fake other side |
| `just test-e2e` | Runs the real server and clients together |
| `just coverage` | Fails if tests cover under 90% of changed lines |
| `just check` | Runs formatting, architecture, test format, lint, and isolated-build checks |
| `just fmt` | Formats the code |

## Layout

```text
assets/          Game assets, stored with Git LFS
crates/
  foundation/    Identifiers and settings used across the game
  protocol/      Registers every game module's network types
  movement/      One game module, with src/client/ and src/server/ folders
  server/        Headless server: assembles plugins, owns connections
  client/        Game client: assembles plugins, owns the world view
  test_support/  Fake client, stub server, and test helpers
  e2e/           Tests that run a real server and clients together
libs/
  bdd/           The #[spec] macro for given, when, and test steps
  env_config/    Typed environment variables, inspired by envalid
  sandbox_kit/   Shared window and scene for each module's examples/sandbox.rs
xtask/           Development launcher and project checks
```

See [CONTRIBUTING.md](CONTRIBUTING.md) to make a change, and
[docs/architecture.md](docs/architecture.md) for where new code belongs, and
[docs/assets.md](docs/assets.md) before adding an asset.
AI coding agents should start with [AGENTS.md](AGENTS.md).

## Tests

There are unit, integration, and end-to-end tests. See the testing
levels in [docs/architecture.md](docs/architecture.md).

## Configuration

`just` commands run in development with no setup. To set real values, copy
`.env.example` to `.env`, which Git ignores. Servers default to production and refuse
to start without a real `DAWN_PRIVATE_KEY`. See
[docs/architecture.md](docs/architecture.md).
