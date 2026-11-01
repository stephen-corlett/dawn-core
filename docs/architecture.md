# Architecture

Project Dawn is split into crates by game module. The goals are small rebuilds,
clear interfaces, and server rules that never ship to players.

`cargo xtask check-architecture` enforces the dependency rules below. Rules
marked **(review)** need human judgment during code review.

## Vocabulary

These terms come from domain-driven design, except where noted.

| Term | Meaning here | Example |
|---|---|---|
| Domain | A subject area with its own knowledge and rules. Domains nest. | The MMO, its auction house |
| Subdomain | A domain viewed as one part of a larger one | The auction house within the MMO |
| Bounded context | A boundary inside which one model and its terms have one meaning | "Character" means position and speed in movement |
| Game module (ours) | The code for one bounded context, in one crate | `movement` |
| Rust module | The language feature declared with `mod`. Crates contain many. | `mod rules;` |

A domain describes the problem, and a game module is code that addresses it. Say
"game module" when you mean the code. How much domain a game module holds varies: an
auction house has rich rules of its own, while movement is much thinner today.

## Crate layers

| Layer | Crates | May depend on |
|---|---|---|
| Library | Anything in `libs/` | Other libraries only |
| Foundation | `foundation` | Only `bevy`, `lightyear`, and `serde` |
| Game module | `<module>` | Libraries, foundation, other game modules |
| Protocol | `protocol` | Libraries, foundation, game modules |
| Server app | `server` | Libraries, foundation, protocol, game modules |
| Client app | `client` | Libraries, foundation, protocol, game modules |
| Test support | `test_support` | Libraries, foundation, protocol, game modules |
| End-to-end tests | `e2e` | Anything except `xtask` |
| Tooling | `xtask` | No game crates |

Crates sit directly under `crates/` or `libs/`, and each folder has the same name as
its crate. `test_support`, `bdd`, and `sandbox_kit` may only be dev-dependencies, so
test and sandbox code can't ship.

## Inside a game module

A game module is one crate, split by Cargo features:

```text
crates/movement/
  src/lib.rs            Shared types, events, and rules, always compiled
  src/rules.rs          Shared code
  src/client/mod.rs     Compiled only with the `client` feature
  src/server/mod.rs     Compiled only with the `server` feature
  examples/sandbox.rs   This module's sandbox, never part of a game build
```

These rules keep each side's code out of the other app. Features merge across a
build, so the compiler can't guarantee this alone, and the checks do that job:

- **Only apps choose a side:** The `server` crate enables the `server` feature of
  game modules, and the `client` crate enables their `client` feature.
  `check-architecture` rejects any other combination, including a game module
  enabling another module's side. End-to-end tests are the exception.
- **Sides are never on by default:** `check-architecture` rejects this.
- **Don't split sides into crates:** `check-architecture` rejects any game module
  crate named `_server` or `_client`.
- **Check each app alone:** `check-app-sides` confirms what each app really compiles.
- **Build shipped apps separately:** Always use `cargo build -p server` or `-p client`.
  A `--workspace` build merges both sides, so never ship from one.

### Organizing a growing game module

Start as small as `movement`, and split files only when the code calls for it. As a
game module grows, organize it by behavior, as in this example for a future `combat`
crate:

```text
crates/combat/
  assets/               Text data that only the sandbox reads
  examples/sandbox.rs
  src/lib.rs            Public interface and protocol plugin
  src/components.rs     Shared components
  src/messages.rs       Shared network messages
  src/rules/damage.rs   Pure logic, with no systems or queries
  src/server/mod.rs     Plugin wiring only
  src/server/abilities.rs
  src/server/damage.rs
  src/client/mod.rs     Plugin wiring only
  src/client/abilities.rs
  src/client/hud.rs
```

- **Split by behavior:** Inside `src/server/` and `src/client/`, give each behavior a
  file with its own systems, private components, and small plugin. Don't add
  `systems/`, `resources/`, or `plugins/` folders, because they scatter one behavior
  across several places. **(review)**
- **Keep `mod.rs` for wiring:** Once a side has several files, its `mod.rs` holds only
  the side's plugin and `mod` declarations. **(review)**
- **Match names across sides:** If a behavior exists on both sides, use the same file
  name on each. Behavior for one side, such as `hud.rs`, has no counterpart. **(review)**
- **Group shared definitions by type:** `components.rs` and `messages.rs` are the
  exception, because they hold data and no behavior. Keep them in `lib.rs` until it
  gets hard to read. **(review)**
- **Split when a file does two things:** Line count isn't the test. Turn a file into a
  folder only when it has separate sub-behaviors of its own. If a side needs more than
  two levels, consider whether the code is a separate game module. **(review)**
- **Name what the code does:** Avoid `utils.rs` and `helpers.rs`, because they attract
  unrelated code. Move the code next to where it's used, or name the file after its
  job. **(review)**

## Libraries

`libs/` is for generic subdomains: solved problems with no knowledge of this game.
Look for a suitable external crate first, and build one here only if none fits. Move
a crate into `libs/` only when it has:

- No game-specific names or concepts **(review)**
- A stable, documented public API, unless it's an internal development tool
  **(review)**
- Dependencies on nothing outside `libs/` except external crates

The architecture check rejects a library that depends on anything in `crates/`.

## Sandboxes

A sandbox is a small runnable app for trying one game module without the full game.
Each module keeps its own in `examples/sandbox.rs`, which Cargo leaves out of normal
builds, and `just sandbox <module>` runs it. Don't let game features start here.

Sandboxes share their window, ground, lighting, and camera through
`libs/sandbox_kit`, which game modules list as a dev-dependency. The kit knows
nothing about the game, and it may only hold setup that several sandboxes need.

A sandbox also proves a module's independence. If it's hard to run a module here,
that module is probably coupled to something it shouldn't be.

## Where code goes

Ask these questions in order, and stop at the first yes:

1. **Do clients need never to see it?** Examples are loot tables, AI, and
   anti-cheat. Put it in the game module's `src/server/`.
2. **Is it only about presentation or local input?** Examples are UI, effects,
   and sound. Put it in the game module's `src/client/`.
3. **Do the server and client both need it, or do other game modules use it?**
   Put it in the game module's shared code, which is its public interface.
4. **Does it pass every test below?** Put it in `foundation`.

Otherwise, check it against the tests in the next section before creating a new
game module.

## What makes a game module

A new game module must pass all of these **(review)**:

1. **Own model:** It represents shared things in its own way. A character is a
   position and speed to movement, but health and threat to combat.
2. **Own language:** Its terms can be defined without explaining another module.
3. **Own rules:** It makes decisions of its own, rather than only holding data or
   helpers.
4. **Own reason to change:** Most design changes in this area stay inside it.

If it fails, the code belongs in an existing game module or a library.

### Subdomain types

Classify each game module, because the type decides how much to invest in it. Types
can change as the game does.

| Type | Meaning | Approach |
|---|---|---|
| Core | What makes this game worth playing | Custom game module, with the most design care |
| Supporting | Needed and specific to this game, but not why people play | Custom game module, kept simple |
| Generic | A problem others have already solved | External crate first, otherwise `libs/` |

### Context map

Update this table whenever a game module is added or its dependencies change.

| Game module | Type | Owns | Depends on |
|---|---|---|---|
| Movement | Supporting | Where a player is, and how input moves them | None |

## What goes in `foundation`

Every crate can depend on `foundation`, and changing it rebuilds the game. Use your
judgment, because no count or size decides this **(review)**. Something belongs in
`foundation` if all of these are true:

1. **Needed everywhere:** It's reasonable that every crate does or would need it. If
   only part of the game would, it belongs to a game module instead.
2. **No clear owner:** It isn't a concept that one game module could reasonably own.
   Health belongs to combat, even if many game modules read it.
3. **No game rules:** It's an identifier, a setting, or a basic type. It contains
   no formulas, systems, or plugins.
4. **Stable:** You'd expect it to change a few times a year at most. If you're
   still shaping it, it isn't ready.

Before adding to it, ask whether the change is worth rebuilding every crate.

The architecture check limits `foundation` to its allowed external crates, because
every crate inherits them. Don't add to that list without a deliberate design decision.

## Give game modules library-style boundaries

Give each game module the discipline of a third-party crate: a small public API,
injected values, and no reach into the apps. It can still contain this game's
concepts and rules, so working in another game isn't a goal. `movement` is the
reference example.

- **Inject configuration:** Game modules take tuning values through plugin fields, like
  `MovementServerPlugin { config }`. Don't hardcode balance values in a game module, and
  don't give a config type a `Default` containing them. **(review)**
- **Use the engine's time:** Read the fixed step from `Time<Fixed>` and pass elapsed
  time into rules. Don't import game-wide constants. **(review)**
- **Inject what the player or environment decides:** Key bindings and similar
  values come from the app. **(review)**
- **Define ports for external systems:** A game module that needs a database or another
  service declares a trait, such as `CharacterStore`, and the server app supplies
  the real implementation. **(review)**
- **Assemble in the apps:** Only `server` and `client` know the whole game.
- **Keep the API small:** Make Rust modules private and re-export what callers need.
  Document anything public. **(review)**
- **Avoid game crates:** Don't depend on `foundation` or another game module out of habit.
  Add the dependency only when the game module really shares that concept. **(review)**

### Secrets, private content, and private logic

| Kind | Examples | Rule |
|---|---|---|
| Secrets | Signing keys, database passwords | Inject at runtime. Never put them in source code, including game modules. |
| Private content | Loot tables, spawn rates | Load as data on the server, and pass it into server plugins. |
| Private logic | AI, anti-cheat, server checks | Keep in `src/server/`, which client builds don't compile. |

## How game modules work together

- **Public interface:** A game module's shared code is its interface. Keep types and
  Rust modules private unless another crate needs them. **(review)**
- **Translate shared concepts:** Prefer a module's own view of a shared thing over
  reaching into another module's components. Sharing a type couples both sides to
  it, so do it deliberately. **(review)**
- **Allowed communication:** Game modules interact through public components, events,
  and functions in their shared code. One module's `server` code can't call another
  module's `server` code.
- **The writer owns a component:** If several game modules use a component, it belongs
  to the one that changes it, and the others only read it. For example, `combat` would
  own `Health`, and a UI module would display it. **(review)**
- **Export system sets for ordering:** When another game module must run before or
  after yours, expose a public system set, which is a named group of systems. Bevy
  otherwise runs systems in any order, which causes bugs that only appear sometimes.
  **(review)**
- **Two game modules need each other:** Rust won't allow the dependency cycle. Pick the
  more basic game module and let the other depend on it, then send information back
  with an event the basic game module defines. Only move a contract lower if neither
  game module owns it. **(review)**
- **Only create what exists:** Add a `server` or `client` feature when a game module
  has code for that side. Don't add empty folders. **(review)**
- **Thin apps:** `server` and `client` assemble plugins and own the
  connection. Game features belong in game modules. **(review)**

## Testing levels

This project has three levels. People define test names differently, so use these
meanings here:

| Level | What's real | What's fake | Location |
|---|---|---|---|
| Unit | A function, or a plugin with its real ECS systems | No network; inputs are set directly | `#[cfg(test)]` beside the code |
| Integration | One whole app, over UDP | The other side, using `fake_client` or `stub_server` | `tests/integration/` in `server` or `client` |
| End-to-end | The server and clients without windows, over UDP | Nothing in the game logic | `crates/e2e/tests/e2e/` |

To choose a level, ask what the test must prove:

- **Rules and edge cases:** If the behavior would be wrong with no app and no network,
  use a unit test.
- **Wiring and boundaries:** If it depends on plugins being added or on messages
  crossing the network, use an integration test.
- **A flow through both real programs:** Use an end-to-end test.

### Test format

Tests are written with `#[spec]` from `libs/bdd`, which has no dependencies:

```rust
use bdd::spec;

#[spec]
fn walking() {
    given("two players", || {
        let mut players = [0.0, 0.0];

        when("player 1 walks", || {
            players[0] += 1.0;

            test("player 1 moves", || {
                assert_eq!(players[0], 1.0);
            });

            test("player 2 stays still", || {
                assert_eq!(players[1], 0.0);
            });
        });
    });
}
```

- **Fresh setup:** Each `test` becomes a normal `#[test]` function containing a copy of
  the setup above it, like `beforeEach` in Jest. Setup runs again for every test, so
  an integration scenario with two tests starts two servers.
- **Structure:** `given` holds `when`, and `when` holds `test`. Anything else is a
  compile error at that line. Setup must come before nested steps.
- **One behavior per test:** A `test` can have several assertions when they prove one
  behavior. Otherwise, write more `test` steps **(review)**.
- **Not real closures:** `given`, `when`, and `test` only look like function calls, so
  `return` and `?` in a step apply to the generated test.
- **Waiting:** A `test` can poll with `run_until` when the result takes time.
- **Names:** Descriptions become the test path, such as
  `walking::given_two_players::when_player_1_walks::player_1_moves`.

`cargo xtask check-tests` scans every Rust file and `Cargo.toml` in the project. It
rejects test attributes other than `#[spec]`, including `#[test]`, `#[tokio::test]`,
and `#[rstest]`. It also rejects runnable doc tests, so mark doc code blocks `ignore`
or `text`, and it rejects `harness = false`. `#[cfg(test)]` stays allowed.

The check reads source text, so it can't see a macro that assembles a test attribute
from parts. Don't write macros that generate tests, or adopt dependencies that do,
without an explicit design decision **(review)**.

### Test coverage

`just coverage` runs every test except end-to-end with `cargo-llvm-cov`, and compares
the result with the Git diff. It fails if tests executed less than 90% of the changed
executable lines, and it always lists uncovered lines with a suggested level:

```text
  crates/movement/src/rules.rs:21   suggested: unit
  crates/server/src/network.rs:48   suggested: integration
```

CI runs each test once, with coverage on. `Build` is a matrix of three parallel entries,
each with its own cache: `Check` for `Lint`, `Isolated crates` for `check-isolated`, and
`Coverage` for `Test`. `Test` restores that instrumented build for the coverage check,
and end-to-end tests then use the same build. Each pull request gets one comment, updated on every push, with the result and a
short list of missing lines. Uncovered lines are also annotated on the diff. The
comment and annotations have limits, so the job summary is the complete list.

- **Either level passes:** The suggestion is a starting point. Choose the level using
  the guidance above **(review)**.
- **Test code doesn't count:** Changed lines from a file's `#[cfg(test)]` line onward
  are left out, so new tests can't hide untested source lines. Keep test modules at
  the bottom of the file.
- **End-to-end tests don't count:** They run separately, so they can't hide a missing
  unit or integration test.
- **What's compared:** By default, your working tree against where the branch left
  `origin/main`, including uncommitted and new files. Use `just coverage --base <ref>` to
  compare with something else. A repository needs a first commit for this to work.
- **What counts:** Code in `crates/` and `libs/`. `xtask/` is project tooling, so it
  has no tests and doesn't count. `EXCLUDED_PATHS` in `xtask/src/coverage.rs` also leaves
  out code tests can't reach, such as rendering and program entry points. Don't add to
  it to get a change through.
- **Executed isn't verified:** Coverage shows a line ran, not that a test checked the
  result **(review)**.

### Choosing test tools

Use `#[spec]` with `cargo-nextest`. Add `proptest` for rules with many possible
inputs, such as damage formulas.

### General guidance

- Test at the lowest level that can show the behavior, and keep end-to-end tests
  few, because they're the slowest and hardest to debug.
- The fake client and stub server share the real protocol, but never real client or
  server code. If they did, a test could pass because both sides share a bug.
- Give every networked test its own port. Tests run at the same time.
- Rendering isn't covered by automated tests yet.

## Configuration

Each app declares its own environment variables, so client settings never sit beside
server secrets. `libs/env_config` is a small library inspired by Node's `envalid`:

```rust
let mut env = Env::from_process(ENVIRONMENT_VAR);
let bind_addr = env.read(
    "DAWN_BIND_ADDR",
    parsed::<SocketAddr>()
        .desc("Address the server listens on")
        .default(public_addr)
        .dev_default(local_addr),
);
env.validate()?; // Reports every problem at once
```

| Kind | Examples | Where it lives |
|---|---|---|
| Committed settings | Addresses | `default` and `dev_default`, in `crates/server/src/config.rs` or `crates/client/src/config.rs` |
| Secrets | Signing keys, database passwords | Environment variables, supplied locally by a gitignored `.env` |

- **Choosing an environment:** `DAWN_ENV` is `development` or `production`. Unset means
  production, so a misconfigured server refuses to start. `just` commands set
  `development`.
- **Required variables:** A variable with no `default` or `dev_default` is required.
- **Production secrets:** Give a secret no `default`. `DAWN_PRIVATE_KEY` is required in
  production, and `forbid_in_production` rejects the development key there.
- **Development secrets:** `DEV_PRIVATE_KEY` is a shared placeholder, so a fresh checkout
  runs with no setup and the client and server can connect.
- **Client limits:** Clients never receive the production key, because it would ship
  to players. A production client needs a login service to issue connection tokens,
  so it refuses to start until one exists.
- **Adding a variable:** Declare it in the app that uses it, and add it to `.env.example`
  with no real value. Game modules receive values through plugins instead.

## Network protocol

Client and server must register network types in the same order. Each game module
provides a protocol plugin, and `protocol` adds them in one list. Increase
`PROTOCOL_ID` when a change breaks older clients.

`protocol` also holds tuning values that both sides simulate with, because
prediction breaks if they differ. It contains no game module types or logic.

## Dependency cycles

Cargo refuses cycles through normal dependencies, but it allows them through
dev-dependencies. `cargo xtask check-architecture` rejects both.

## Cargo features

Cargo merges a dependency's features across everything in a build, so a crate can
compile only because another crate enabled a feature it needs. Declare what each
crate uses directly. `cargo xtask check-isolated` builds every crate alone, and with
each side on its own, to catch this. Workspace-wide builds and tests hide it.

## Adding a game module

1. Confirm it passes the tests in "What makes a game module", and classify it.
   Then create `crates/<module>`, with `src/server/` and `src/client/` if needed.
2. Add the crate to `[workspace.dependencies]`, and declare any `server` and `client`
   features it needs.
3. Export a protocol plugin from the shared code if the game module has network
   types, and add it to `protocol`.
4. Enable each side in `server` and `client`, and add the plugins there,
   passing in their configuration.
5. Add `examples/sandbox.rs`, with `sandbox_kit` as a dev-dependency and an
   `[[example]]` entry listing any features it needs. Then add the game module to the
   context map.
6. Follow `movement` as the worked example.
7. Run `just check` and `just test`.

## Changing these rules

Change this file, the matching code in `xtask/src/architecture.rs`, and its tests
in the same commit, so the written rules and enforced rules can't drift apart.
