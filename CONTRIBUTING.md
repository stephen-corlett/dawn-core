# Contributing

## Setup

You need Rust, `just`, `cargo-nextest`, and `cargo-llvm-cov`. On Windows, you also
need the Visual Studio C++ build tools. `rust-toolchain.toml` selects the Rust version
and its components.

```bash
just dev
```

## Making a change

1. Create a branch from `main`, such as `feat/quest-log` or `fix/rollback-snap`.
2. Read [docs/architecture.md](docs/architecture.md) if your change adds a crate, adds
   a dependency between crates, or changes `foundation`.
3. Read [docs/assets.md](docs/assets.md) if your change adds an asset.
4. Write tests with `#[spec]` at the lowest level that shows the behavior.
5. Run the checks below.
6. Open a pull request into `main`.

## Before opening a pull request

```bash
just check
```

```bash
just test
```

```bash
just coverage
```

CI runs the same checks. Pull requests must have at least 90% test coverage on changed
lines. The job summary lists any uncovered lines with a suggested test level.

## Commit messages

Commits follow [Conventional Commits](https://www.conventionalcommits.org/):

```text
<type>(<scope>): <description>
```

For example:

```text
feat(movement): add swimming speed to MovementConfig
fix(server): remove players after a client timeout
test(client): cover rollback after a server correction
```

### Types

| Type | Use it for |
|---|---|
| `feat` | A new feature or player-visible behavior |
| `fix` | A bug fix |
| `refactor` | A code change that doesn't alter behavior |
| `perf` | A performance improvement |
| `test` | Adding or changing tests |
| `docs` | Documentation only |
| `build` | Dependencies and build configuration |
| `ci` | GitHub Actions workflows |
| `chore` | Maintenance that doesn't fit the types above |

### Rules

- **Scope:** Use the crate or game module name, such as `movement` or `xtask`. Leave it
  out if a change spans the whole project.
- **Description:** Write in the imperative mood, in lowercase, with no period at the
  end. "add swimming speed" works, but "Added swimming speed." doesn't.
- **Breaking changes:** Add `!` after the type or scope, and explain the change in a
  `BREAKING CHANGE:` footer. Also increase `PROTOCOL_ID` if older clients can no longer
  connect.
- **One purpose:** Keep each commit to one logical change, with its tests included.
- **Body:** Add one when the reason for a change isn't obvious. Explain why, not what.

## Pull requests

- `main` only accepts pull requests, so work on a branch.
- Use a Conventional Commits title, since it describes the whole change.
- Fill in every section of the
  [pull request template](.github/pull_request_template.md). The GitHub website loads
  it automatically, but `gh pr create` skips it with `--body` or `--fill`.
- Keep each pull request focused. Separate unrelated changes.
