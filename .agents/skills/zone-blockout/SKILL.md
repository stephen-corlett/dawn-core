---
name: zone-blockout
description: Build a zone's terrain and grey landmarks in the real game, and measure its walk times and sight lines, at gate 3 of the zone pipeline. Use when zone-design reaches gate 3, or when someone asks for a zone blockout or greybox.
---

# Zone blockout

You're doing gate 3 of the zone pipeline. You can't ask anyone anything, and you can't see any
earlier conversation, so work only from the brief you were given.

Read [docs/zone-pipeline.md](../../../docs/zone-pipeline.md), especially "Rules for every job" and
"Architecture", then [docs/world/bible.md](../../../docs/world/bible.md), the brief, and "Rules for
every job" in [docs/art-pipeline.md](../../../docs/art-pipeline.md).

Check "What's built" against the repository. If the scale test, zone data, `zone-view`, or
`zone-check` is missing, report **Blocked** with what's missing, and stop.

## Steps

1. **Build the shape as a recipe.** Write the terrain as named strokes in the zone's shape file,
   each with its reason, from the largest shapes to the smallest. Don't patch generated results.
2. **Add landmarks in grey.** Use plain shapes at their real size. No textures, no scatter.
3. **Measure.** Run `zone-check` for route times, the blocked share, and the sight graph.
4. **Look.** Capture a top-down map, and views from a player's eye at the arrival, every junction,
   and every landmark. Open each one. A ray test isn't proof that something reads from there.
5. **Walk it.** Move through the game on the natural route, and confirm that nothing traps a player
   and that blocked land looks blocked.
6. **Fix the shape.** If a sight line fails, move the terrain before you move the landmark, because
   landmarks have reasons. If a change touches the skeleton, update the brief to match.
7. **Record.** Fill in the Targets table, and add a gate log entry with evidence and every miss. Note
   the frame rate at each view.

## Report

- **Needs decision:** The blockout is ready for a person to walk, or a target was missed. A person
  passes this gate, so never report **Passed**.
- **Failed:** The skeleton can't work on real terrain. Say why.
- **Blocked:** A game is already running, or a tool the job needs doesn't exist.
