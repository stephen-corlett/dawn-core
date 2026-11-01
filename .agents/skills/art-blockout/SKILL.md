---
name: art-blockout
description: Put a rough placeholder for a planned asset into the real game, at gate 2 of the art pipeline, to prove its size, placement, and silhouette before any final art. Use when art-integration reaches gate 2, or when someone asks for a blockout.
---

# Art blockout

You're doing gate 2 of the art pipeline. You can't ask anyone anything, and you can't see any
earlier conversation, so work only from the brief you were given.

Read [docs/art-pipeline.md](../../../docs/art-pipeline.md), especially "Rules for every job",
and then the brief.

This is the cheapest place to catch a mistake, so keep it rough.

## Steps

1. Build the placeholder from primitive shapes, or from an existing model at the right size. It
   shouldn't have final art or detail.
2. Place it where the brief says, using the owning game module's normal configuration.
3. Render it from a player's view in the real game, beside the character and nearby props for
   scale. Use `just lookdev --capture <folder> character <model>` too, with a folder outside the
   repository, if the placeholder is a model.
4. Save the images outside the repository, and add a gate log entry with their paths.

Note in your entry whether the placeholder should stay in the working tree for the next gates.
It can't remain once final art replaces it.

## Report

- **Needs decision:** The images are ready for a person to judge size and silhouette, which is
  the normal result.
- **Failed:** It doesn't fit, so say why.
- **Blocked:** Something needed doesn't exist.
