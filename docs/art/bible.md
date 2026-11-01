# Art bible

**Status:** Signed off by the art director on 2026-11-01, after three review rounds.

This is the target every new asset is judged against. Add a case here with an example when a new
one comes up. For how art gets into the game, see [the art pipeline](../art-pipeline.md).

## The look

A high fantasy world in the spirit of classic 2004-era MMOs, not their modern versions: low detail
models, painted surfaces, and light that does most of the work.

## Pillars

- **High fantasy:** The one mood that ties every zone together.
- **Hand-crafted:** It looks painted by a person. This describes the result, because generated
  textures are allowed under the rules in [docs/assets.md](../assets.md).
- **Grounded:** Stylized but serious and weighty, never bouncy, toy-like, or childish.

## World rules

These apply everywhere, and they're what make different zones feel like one game.

| Topic | Rule |
|---|---|
| Shape | Simple, clear silhouettes. Detail comes from painted surfaces, not from small geometry. |
| Surfaces | Soft blended brushwork with no ink outlines. Small painted marks are fine, and scan noise isn't. |
| Materials | Paint how a material takes the light. Generated textures carry color only, with no normal map. |
| Values | Keep textures neutral, so that lighting and fog carry the mood. |
| Large shapes in textures | Tiling textures can't contain big features, because they'd visibly repeat. |

The player's character is the reference for quality: clean, painted, and readable.

![The character at 1 meter, showing clean painted surfaces](images/on-character.jpg)

## Zone rules

Each zone sets its own palette, lighting, and fog, so a dark graveyard and a whimsical forest can
both belong. Zones can't break the world rules.

The game has one `Mood` for now, so zones with their own moods still have to be built.

### The forest

Warm, low sun through haze, with deep but readable shadow. Greens are muted, with warm earth.

![The clearing, the on-style reference for the forest](images/on-clearing.jpg)
![The path, showing painted rocks, trees, and ground together](images/on-path.jpg)

## Readability

A player has to stay readable in shadow. Judge how dark is too dark per zone, by looking at the
character, because a count of dark pixels can't tell mood from a mistake.

## Off style

Each of these was tried, and rejected for the reason given.

| Example | Why it's wrong |
|---|---|
| ![Photo-scanned ground](images/off-photo.jpg) | **Too realistic.** Scanned noise and white light fight the painted models. |
| ![Outlined cartoon ground](images/off-outlines.jpg) | **Too cartoony.** Ink outlines and bright objects read as a mobile game, not Grounded. |
| ![Dark path](images/off-dark.jpg) | **Near miss.** The surfaces are right, but the character sinks into the shadow. |

## Lighting

The same forest, with the lighting that works and with the lighting that doesn't, is shown by the
clearing above and the photo-scanned example. Warm, low light with haze holds the world together,
and white, even light flattens it.

## Art supply

Painted textures are generated with SDXL through a local ComfyUI. The prompts that work describe
the look in plain words: soft blended brushwork, muted colors, even diffuse light, and a uniform
surface with no large shapes. They never name another game or a living artist.

The settings and seeds behind everything in these images are in
[tools/art-bake/forest.json](../../tools/art-bake/forest.json).
