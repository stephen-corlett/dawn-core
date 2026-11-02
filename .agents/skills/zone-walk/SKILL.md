---
name: zone-walk
description: Walk a zone as a first-time player, from player's-eye views alone and without knowing the plan, and report where it pulled you and what it missed, at gate 6 of the zone pipeline. Use when zone-design reaches gate 6, or when someone asks for a blind walkthrough of a zone.
---

# Zone walk

You're doing gate 6 of the zone pipeline. You can't ask anyone anything, and you can't see any
earlier conversation. You're a player who has never seen this zone.

**Don't read the plan.** Until step 5, don't open anything in `docs/world/zones/`, the zone's data
files, its maps, or `docs/world/atlas.md`. If you've seen any of them, or were told what's in the
zone, report **Blocked**, because the walk would prove nothing.

You can read "Sight lines" in [docs/world/bible.md](../../../docs/world/bible.md), and how to run
`zone-view`. If it's missing, report **Blocked** and stop. Change nothing except the brief's gate
log.

## Steps

1. **Start at the arrival.** Capture a view from a player's eye in each direction, and open them all.
2. **Choose as a player would.** Write what you notice, what you want to reach, and why. Choose by
   what you can see. Don't use coordinates, data, or knowledge of how worlds are usually built.
3. **Move and repeat.** Go towards your choice, a few hundred meters at a time, and capture again.
   Log each step, what drew you, and whether you were rewarded. Keep going until you've spent about
   the zone's walking time, or nothing draws you.
4. **Note the failures.** Record any moment with nothing to want, a lure that led to nothing, anything
   confusing, and anywhere that looked walkable and wasn't.
5. **Compare with the plan.** Now read the brief. Mark what you found and what you missed, where you
   went against the intended lure, and any place you never wanted to visit.
6. **Record.** Add a gate log entry with your route on a map, the step log, image paths, and those
   differences. Say plainly where the zone failed to pull you.

You're a stand-in with limits. You can't feel pace or see motion, and you read images differently
from a person, so don't claim the zone feels good. Report what you saw and chose.

## Report

- **Passed:** You found the hub and most landmarks by following what you saw, and every miss is listed.
- **Failed:** The zone didn't lead you. Name each place and what was missing.
- **Needs decision:** It works, but a person should look at something.
- **Blocked:** You saw the plan first, a game is already running, or `zone-view` doesn't work.

Passing here doesn't finish the zone. A person walks it at gate 7.
