---
name: art-integration
description: Take new art from a request all the way into the game, running every gate of the art pipeline in order and asking a person for each decision. Use whenever someone wants to add, replace, or improve models, textures, props, foliage, characters, or an asset pack, or asks to run the art pipeline, even if they don't say "skill".
---

# Art integration

This runs the process in [docs/art-pipeline.md](../../../docs/art-pipeline.md), which you must
read first. You coordinate. The job skills do the work, and a person makes every art decision.

Stay in the main conversation. Sub-agents can't ask a person anything, so asking is your job.

## 1. Check what exists

Check "What's built" in the pipeline document against the repository, because it may be out of
date. Note which pieces are missing, but keep going. Each gate below says what it needs.

If `docs/art/bible.md` doesn't exist, explain that the look has to be agreed first, and run
`art-look-lock`. Don't write a brief against a look nobody has decided.

## 2. Find the brief

Look in `docs/art/briefs/` for a brief that matches the request. If there is one, read its gate
log and continue from the first gate that hasn't passed. Otherwise, start at gate 1.

If the request covers a whole pack or a new kind of asset, tell the person you'll start with a
calibration batch of three models, and agree which three. Only start the rest once those pass.

## 3. Run the gates

Run each job with the Skill tool, passing the brief's path. Don't run the next gate until the
current one has passed.

| Gate | Job | Needs | Who passes it |
|---|---|---|---|
| 1. Brief | `art-brief` | Art bible | Person |
| 2. Blockout | `art-blockout` | Brief | Person |
| 3. Source | `art-source` | Blockout | Person |
| 4. Cook | `art-cook` | Source, cook step | The job's automated rules |
| 5. Integration | `art-integrate`, then `art-review` | Cook, look development scene | `art-review` |
| 6. Sign-off | None | Passed review | Person |

Act on what each job reports:

- **Passed:** Read the brief's new log entry and look at its evidence yourself. A job's report
  is a claim until you've seen the images. Then continue, or ask for sign-off if a person
  passes this gate.
- **Needs decision:** Ask the person, showing them the options and evidence. Add their answer
  to the brief's Decisions section in their words, then run the job again.
- **Failed:** Run the job again with the reasons, once. If it fails a second time, stop and
  ask the person, because repeated failure usually means the brief or the bible is wrong.
- **Blocked:** Say what's missing and which step in the pipeline document builds it. Stop,
  unless the person tells you to build it now.

When a person has to decide, ask one question at a time, and show the images. Never pass a gate
on their behalf, and treat silence or "looks fine so far" as not yet passed.

## 4. Sign off

At gate 6, show the person the review's images and any recorded misses. Their decision goes in
the brief. If they reject it, ask whether the fault is in the asset, the brief, or the bible,
go back to that gate, and update the bible if it was wrong.

## 5. Finish

Run `just check`, `just test`, and `just coverage`, and confirm temporary tools are gone.
Report what's in the game, what was measured, and any recorded misses.

Don't commit or open a pull request unless asked. When you do, follow `CONTRIBUTING.md`, and
list a person's checks under the template's Manual checks.
