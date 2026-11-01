---
name: art-look-lock
description: Agree what the game should look like with the art director, prove it on hero assets in the engine, and write the art bible. Use when a visual style hasn't been decided, when docs/art/bible.md doesn't exist, or when someone wants to change the game's overall look.
---

# Art look lock

This settles the look once, before production. Read
[docs/art-pipeline.md](../../../docs/art-pipeline.md) first, especially "Rules for every job".

Stay in the main conversation, because this is a discussion with the art director. They make
every decision here. You offer options, prove them on screen, and write down what they chose.

## 1. Set the limit

Agree a hard end for exploration, as a number of review rounds. Three is a good default.
Without one, style work never finishes.

## 2. Compare where we've been

Nothing built so far is the baseline, because all of it came before this process. Build the game
at commit `b026499`, before the photo-scanned ground, and at the current commit. Show the art
director the same views from both, and ask what's working and what isn't in each. Use their
answers as the starting point, and don't assume that the newer one is better.

## 3. Decide the questions

Ask about these one at a time, and show reference for each:

- **Pillars:** Three to five words the look has to feel like.
- **Fidelity:** How painted or realistic, and how much detail.
- **Shape:** The silhouettes and proportions that belong in this world.
- **Palette:** The role of each color, not just swatches.
- **Materials:** How stone, bark, leaves, skin, and metal are shown.
- **Lighting:** The intended mood, including how dark it gets, and whether Bevy's physically based
  sky can reach it.
- **Art supply:** Where this look's art comes from. Check what the approved libraries in
  `docs/assets.md` actually have, and whether a local image model can reach the look, before
  settling on a style.

Only use reference images the art director gives you, or links to published games and art.
Reference is for discussion, and it never ships.

## 4. Prove it in the engine

1. Agree three hero assets: normally the character, one tree, and one rock.
2. Show them with `just lookdev`, which uses the game's real mood from
   `crates/client/src/look.rs`. Save each round with `--capture`, to a folder outside the
   repository, so that rounds can be compared.
3. Change shaders, lighting, and materials together until the heroes match the answers above.
4. Show the art director each round's images, and record what they say.

If a hero needs art that the approved libraries don't have, generate options under the rules in
`docs/assets.md`. The art director picks which one, if any, goes further.

## 5. Write the bible

When the art director signs off, write `docs/art/bible.md`. Keep it short, and include:

- Each answer from step 3
- The texel density and darkness range that worked
- Three to five on-style and off-style images per pillar, including near misses
- The same asset under correct and incorrect lighting

Then update the technical spec in the pipeline document with the values that worked, and update
"What's built".

If new cases come up later, add them to the bible with an example.
