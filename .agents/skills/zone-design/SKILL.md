---
name: zone-design
description: Take a zone from an idea to something a person has walked in the game, running every gate of the zone pipeline in order and asking a person for each decision. Use whenever someone wants to design, plan, build, extend, or review a zone, region, town, or the layout of the world, even if they don't say "skill".
---

# Zone design

This runs the process in [docs/zone-pipeline.md](../../../docs/zone-pipeline.md), which you must
read first. You coordinate. The job skills do the work, and a person makes every design decision.

Stay in the main conversation. Sub-agents can't ask a person anything, so asking is your job.

## 1. Check what exists

Check "What's built" in the pipeline document against the repository, because it may be out of
date. Note which pieces are missing, but keep going. Each gate below says what it needs.

If `docs/world/bible.md` isn't signed off, or still has open decisions, say which ones matter for
this zone and ask about them first. Don't design against targets nobody has agreed.

## 2. Find the brief

Look in `docs/world/zones/` for a brief that matches the request. If there is one, read its gate
log and continue from the first gate that hasn't passed. Otherwise, start at gate 1.

Gates 4 to 6 run per region, starting from where players arrive. Agree which region you're on. Don't
start a second region until a person has walked the first.

## 3. Run the gates

Run each job with the Skill tool, passing the brief's path and the region. Don't run the next gate
until the current one has passed.

| Gate | Job | Needs | Who passes it |
|---|---|---|---|
| 1. Premise | `zone-premise` | World bible | Person |
| 2. Skeleton | `zone-skeleton` | Premise | Person |
| 3. Blockout | `zone-blockout` | Skeleton, scale test, zone data, `zone-view`, `zone-check` | Person |
| 4. Sites | `zone-sites` | Blockout, `zone-view`, `zone-check` | Person |
| 5. Dressing | `zone-dressing`, then `art-integration` | Sites, separated dressing, a mood per zone | Art pipeline, then person |
| 6. Walk | `zone-walk` | Dressing, `zone-view` | `zone-walk` |
| 7. Sign-off | None | Every region walked | Person, in the real game |

Pass `zone-walk` only the zone's name, the region, and the place to start, never the brief's path or
any design detail. If it learns the plan, its walk proves nothing.

Act on what each job reports:

- **Passed:** Read the brief's new log entry and look at its evidence yourself. A job's report
  is a claim until you've seen the images. Then continue, or ask for sign-off if a person
  passes this gate.
- **Needs decision:** Ask the person, showing them the options and evidence. Add their answer
  to the brief's Decisions section in their words, then run the job again.
- **Failed:** Run the job again with the reasons, once. If it fails a second time, stop and
  ask the person, because repeated failure usually means the skeleton or the bible is wrong.
- **Blocked:** Say what's missing and which step in the pipeline document builds it. Stop,
  unless the person tells you to build it now.

When a person has to decide, ask one question at a time, and show the map or views. Never pass a gate
on their behalf, and treat silence or "looks fine so far" as not yet passed.

## 4. Change the plan when it's wrong

A region often shows that the plan was wrong. If a walk or a person's feedback says so, ask whether
the fault is in the region, the skeleton, the premise, or the bible. Go back to that gate. If the
bible was wrong, update it with a person's agreement.

## 5. Sign off

At gate 7, ask the person to walk the zone from its arrival in the real game, ideally without
reading the brief first. Show them the walk reports and any recorded misses. Their decision goes in
the brief, and the zone's row in `docs/world/atlas.md` is updated.

## 6. Finish

Run `just check`, `just test`, and `just coverage` if code changed, and confirm temporary tools are
gone. Report what's in the game, what was measured, and any recorded misses.

Don't commit or open a pull request unless asked. When you do, follow `CONTRIBUTING.md`, and list a
person's checks under the template's Manual checks.
