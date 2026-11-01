---
name: art-integrate
description: Wire a cooked asset into its game module with tests, replacing any placeholder, at gate 5 of the art pipeline. Use when art-integration reaches gate 5, or when someone asks to put an approved asset in the game.
---

# Art integrate

You're doing the first half of gate 5. You can't ask anyone anything, and you can't see any
earlier conversation, so work only from the brief you were given.

Read these first:

- [docs/art-pipeline.md](../../../docs/art-pipeline.md), especially "Rules for every job"
- [docs/architecture.md](../../../docs/architecture.md)
- The brief. Gate 4 has to have passed, or report **Blocked**.

## Steps

1. Put the asset in the game through its owning module's configuration, the way similar assets
   are added. Replace the placeholder from gate 2.
2. Don't put paths or tuning values in a game module. How things look belongs in
   `crates/client/src/look.rs`. Forest models and plantings belong in
   `crates/protocol/src/forest.rs`, which the server also uses, so add a new planting only at the
   end of the list. Anywhere else moves every later prop, including solid trees. Report
   **Needs decision** if a change would move anything a player can collide with.
3. Add `#[spec]` tests at the lowest level that shows the behavior. A test that reads a model or
   texture has to handle a Git LFS pointer, and has to check something useful in CI too.
4. Run `just check`, `just test`, and `just coverage`.
5. Run `just dev`, or a scripted client if you can't watch it, and confirm that the asset loads
   with no new warnings. If a game is already running, report **Blocked** and ask for it to be
   closed.
6. Add a gate log entry with what changed.

Don't judge how it looks. `art-review` does that, so that nobody marks their own work.

## Report

Report **Passed** when it's wired in and the checks pass, or **Failed** with the output.
