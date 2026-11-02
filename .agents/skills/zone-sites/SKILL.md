---
name: zone-sites
description: Place one region's settlements, sites, vignettes, and quest hooks in grey, at gate 4 of the zone pipeline, so that each is found by exploring. Use when zone-design reaches gate 4, or when someone asks to add towns, points of interest, or quest locations to a zone.
---

# Zone sites

You're doing gate 4 of the zone pipeline, for one region. You can't ask anyone anything, and you
can't see any earlier conversation, so work only from the brief you were given.

Read [docs/zone-pipeline.md](../../../docs/zone-pipeline.md), then
[docs/world/bible.md](../../../docs/world/bible.md), especially "Settlements" and "Finding quests",
then the brief. If gate 3 hasn't been passed by a person, or `zone-view` or `zone-check` is missing,
report **Blocked** and stop.

## Steps

1. **Start from the routes.** Walk the region's routes in views, and list where a player stops,
   turns, or looks. Those are the best places for things.
2. **Place the settlement first,** if this region has one. Lay it out in grey, so that it's seen from
   a distance and its edge shows where its quests lead.
3. **Add sites.** Give each a row: what lures a player from where, what rewards them, and the story it
   shows. Every site needs a lure and a reward. Cut anything that repeats another site's task.
4. **Add vignettes.** Each shows a chain of events a player can read with no text. Say what happened,
   and what a player would conclude from what's placed.
5. **Hook quests onto places.** Note hub, notice board, and situation quests by where they start and
   which existing places they use. Check that a stuck player always has another thread. Don't add a
   place to fit a quest, and don't write rewards or spawns here, because the brief is public.
6. **Measure.** Run `zone-check` for gaps, and capture a view of each lure from where a player would
   first see it. Gaps have limits, but content shouldn't arrive at a steady beat.
7. **Record.** Add the rows to the brief, and a gate log entry with evidence and misses.

## Report

Report **Needs decision** when the region is ready for a person to review. Report **Failed** if the
region's terrain gives no good places, and say what the blockout should change. A person passes this
gate, so never report **Passed**.
