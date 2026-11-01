---
name: art-cook
description: Process a chosen asset through the cook step and run the automated rules, at gate 4 of the art pipeline. Use when art-integration reaches gate 4, or when someone asks to cook, process, or validate an asset.
---

# Art cook

You're doing gate 4 of the art pipeline. You can't ask anyone anything, and you can't see any
earlier conversation, so work only from the brief you were given.

Read [docs/art-pipeline.md](../../../docs/art-pipeline.md), especially "Architecture", "Technical
spec", and "Rules for every job", and then the brief.

## Steps

1. Check that the cook step exists. If it doesn't, report **Blocked** and name the step in "What's
   built" that creates it. Don't patch files by hand or write a one-off tool in its place.
2. Run the cook on the brief's source art. Source files in `assets/` stay as they were.
3. Run the rules. Blocking rules have to pass, and warnings go in your log entry.
4. Confirm that the output has what the spec requires, such as mipmaps. Read the processed files
   themselves, because a clean run doesn't prove the output is right.
5. If the asset needs a fix, make it a committed recipe in the pipeline, so that it can be
   reproduced from source. A person has to approve anything that changes how the art looks.

Never relax a rule to get an asset through. Report the failure.

## Report

- **Passed:** The cook ran, blocking rules pass, and the output is confirmed.
- **Failed:** Give the rule, the file, and what would fix it.
- **Needs decision:** A fix would change the look.
- **Blocked:** The cook step doesn't exist yet.
