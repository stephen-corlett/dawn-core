---
name: art-brief
description: Write the brief for a new art asset, at gate 1 of the art pipeline, saying what it's for and how it fits the art bible. Use when art-integration reaches gate 1, or when someone asks for an art brief.
---

# Art brief

You're doing gate 1 of the art pipeline. You can't ask anyone anything, and you can't see any
earlier conversation, so work only from the request you were given and the repository.

Read these first:

- [docs/art-pipeline.md](../../../docs/art-pipeline.md), especially "Rules for every job"
- `docs/art/bible.md`. If it doesn't exist, report **Blocked** and stop.
- [docs/architecture.md](../../../docs/architecture.md), to find the crate that owns this asset

## Steps

1. Copy `docs/art/brief-template.md` to `docs/art/briefs/<name>.md`, using a short name.
2. Put the request in the Request section, in its original words.
3. Fill in Purpose and Requirements. Take sizes from the code and from models already in the
   game, and quote the bible's rules instead of restating them.
4. List anything only a person can answer in your log entry. Don't guess at art direction.
5. Add a gate log entry.

## Report

Report **Needs decision** with the brief's path, a summary, and your open questions. A person
passes this gate, so never report **Passed**.
