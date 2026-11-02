---
name: zone-dressing
description: Plan one region's asset kits, mood, ground, and scatter rules, at gate 5 of the zone pipeline, and list the art each needs for the art pipeline. Use when zone-design reaches gate 5, or when someone asks what art or asset kits a zone needs.
---

# Zone dressing

You're doing the planning half of gate 5, for one region. You can't ask anyone anything, and you
can't see any earlier conversation, so work only from the brief you were given.

Read [docs/zone-pipeline.md](../../../docs/zone-pipeline.md), "Asset kits" in
[docs/assets.md](../../../docs/assets.md), [docs/art/bible.md](../../../docs/art/bible.md), and the
brief. If gate 4 hasn't been passed by a person, report **Blocked** and stop.

## Steps

1. **Use what exists.** List the kits in `assets/kits/`, and those the atlas shows on nearby zones.
   Reuse them before proposing another.
2. **List what's needed.** For each landmark, site, and vignette, name the kit that covers it and
   what's missing. A landmark usually needs its own art, because a distant silhouette has to be
   unmistakable.
3. **Plan the scatter.** Write where forests stop, which slopes are bare, and what grows where, by
   region and rule. Keep scatter away from anything that lures a player, and off sight lines.
4. **Plan the mood.** Propose palette, light, and fog for the zone's section of the art bible, keeping
   the bible's world rules. Check that landmarks still read at their target distance through the fog.
5. **Write art requests.** List each new kit or asset as a request `art-integration` can start from,
   giving its purpose, its kit, and how far away it's seen. Don't write art briefs or source art,
   because the art pipeline owns that.
6. **Apply what already exists.** Write the region's dressing data for art that has passed the art
   pipeline, and capture views again to confirm that dressing hasn't closed a sight line.
7. **Record.** Fill in the Dressing section, and add a gate log entry.

## Report

Report **Needs decision** with the list of art requests and the proposed mood. Report **Blocked** if
separated dressing or a mood per zone doesn't exist yet. The art pipeline and a person pass this gate,
so never report **Passed**.
