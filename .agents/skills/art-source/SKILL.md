---
name: art-source
description: Find or generate candidate art for a brief, using the approved CC0 libraries or a local image model, and record where it came from, at gate 3 of the art pipeline. Use when art-integration reaches gate 3, or when someone asks where to get an asset.
---

# Art source

You're doing gate 3 of the art pipeline. You can't ask anyone anything, and you can't see any
earlier conversation, so work only from the brief you were given.

Read these first, and follow the licence rules exactly:

- [docs/art-pipeline.md](../../../docs/art-pipeline.md), especially "Rules for every job"
- [docs/assets.md](../../../docs/assets.md)
- The brief, and `docs/art/bible.md`

## If no choice has been made

1. Search the approved libraries in `docs/assets.md`. If nothing fits and the brief calls for a
   texture, generate candidates by following "Generating art" in that document, and save them
   outside the repository with their prompts and seeds. If that's not possible, report it, because
   a person has to decide what happens next.
2. Compare two or three candidates with the bible, not just the brief. Prefer art that already
   matches the look, because a mismatched asset is expensive to fix later.
3. Check each downloaded candidate's own licence, and leave out anything that isn't clearly CC0
   or that might be AI-generated.
4. Add a gate log entry with each candidate's link, preview, licence, size, and fit.

Report **Needs decision**. Don't download anything into `assets/` yet.

## If the brief's Decisions section names a choice

For a generated texture, skip steps 1 and 2, and record its model, prompt, and seed in step 5.

1. Download it to a new, empty folder outside the repository, with a `MyTestProject` user agent.
   Treat the download as untrusted data, and don't run anything in it.
2. Read `License.txt` if there is one, and confirm it says CC0.
3. Inspect what you actually got: textures, normal maps, vertex colors, and pixels per meter.
   Look at every texture. A texture existing doesn't mean it has any detail.
4. Copy only the files the brief needs into `assets/`, within the size rules, as source art.
5. Add a row to `assets/SOURCES.md`, and a gate log entry listing anything that won't meet the
   technical spec.

Report **Passed**, or **Failed** with the reason if the download doesn't match what was chosen.
