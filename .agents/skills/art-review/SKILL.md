---
name: art-review
description: Independently check art that's in the game against its brief and the art bible, using renders and pixel comparisons, at gate 5 of the art pipeline. Use when art-integrate finishes, or when someone asks to review, audit, or inventory how assets look in game.
---

# Art review

You're doing the second half of gate 5. You can't ask anyone anything, and you can't see any
earlier conversation, so work only from the brief you were given.

You didn't make this work, and your job is to find what's wrong with it. Change nothing except
the brief's gate log.

Read [docs/art-pipeline.md](../../../docs/art-pipeline.md), especially "Rules for every job" and
"Definition of done", then the brief and `docs/art/bible.md`.

If the look development scene doesn't exist, report **Blocked**.

## Steps

1. **Capture a before and after.** Use the same views, lighting, and window size, at 1, 5, and 20
   meters, in sun and in shadow. Capture the real game too.
2. **Check the capture.** View the images before reading any numbers. Confirm that the views
   differ from each other, the world and character loaded, and the frame rate is believable. If
   not, the capture is broken, so fix that and start again.
3. **Prove the change.** Compare the before and after pixels of the surface in question. If
   nothing changed, it failed, whatever the tests and logs say.
4. **Look at every asset.** Check each model in the brief, at each distance. Open the images at
   full size, and don't rely on a shrunken contact sheet.
5. **Judge it against the bible.** Check silhouette, value, material, and lighting. Name the cause
   of each problem, such as texture detail, resolution, filtering, or lighting.
6. **Measure.** Report pixels per meter on the surface a player sees, frame rate against the
   previous value, near-black pixels, and the `just dev` log.
7. **Check motion.** Stills can't show shimmer. Move the camera between frames and compare, or
   record that motion is unverified.
8. **Record.** Add a gate log entry with image paths, measurements, misses, and what you couldn't
   verify. Say plainly where it falls short.

## Report

- **Passed:** Every check is met, and you've named anything unverified.
- **Failed:** List each problem with its cause and image.
- **Needs decision:** It meets the spec but a person should judge something, or a target was missed.
- **Blocked:** The scene needed for the review doesn't exist.

Passing here doesn't finish the asset. A person signs off at gate 6.
