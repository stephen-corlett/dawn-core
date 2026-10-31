# Agent instructions

Project Dawn is a 3D MMO built with Bevy and lightyear. The server is
authoritative: clients send requests, and the server decides what happens.

## Before changing code

Read [docs/architecture.md](docs/architecture.md) before adding a crate, adding a
dependency between crates, or changing `foundation`. It's the source of truth for
where code belongs.

## Rules

- Put game features in game module crates, not in the `server` or `client` apps.
- Check a new game module against "What makes a game module" in the architecture
  doc. Use "game module" for code and "domain" for the problem it addresses.
- Give game modules library-style boundaries: inject configuration, and never put
  secrets or tuning values in them. Copy `movement`.
- A game module is one crate. Client and server code go in `src/client/` and
  `src/server/` behind features. Only apps enable those features.
- Build shipped apps with `-p`. A `--workspace` build merges client and server code.
- Treat `foundation` as the last resort, and apply its tests in the architecture doc.
- Keep server-only rules out of client-reachable crates.
- Don't add game crates to `xtask`.
- Don't weaken a check or raise a limit to get a change through. Fix the design,
  or ask a human to approve a rule change.

## Commits

Follow [CONTRIBUTING.md](CONTRIBUTING.md), including its Conventional Commits format.

## Commands

| Command | Use |
|---|---|
| `just dev` | Run the server and two clients |
| `just sandbox <name>` | Run one game module's sandbox |
| `just check` | Run formatting, architecture, lint, and isolated-build checks |
| `just test` | Run every test |
| `just coverage` | Fail if tests cover under 90% of changed lines, and list the gaps |

Run `just check`, `just test`, and `just coverage` before you call work finished. If
coverage reports lines, add tests at the suggested level unless a different level
fits the behavior better.

## Testing

Use the testing levels in [docs/architecture.md](docs/architecture.md), and test at
the lowest level that can show the behavior. Write every test with `#[spec]` from
`libs/bdd`, using `given`, `when`, and `test` steps. Plain `#[test]` is rejected:

- **Unit:** `#[cfg(test)]` modules beside the code, for rules and plugins with no
  network.
- **Integration:** `tests/integration/` in `server` or `client`, using the
  fake client or stub server from `test_support`.
- **End-to-end:** `e2e`, for the few flows that need the real server and client.
