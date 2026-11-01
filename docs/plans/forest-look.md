# Forest look

The forest looks bland, although its models are fine. The cause is what surrounds them:
the ground has no texture, nothing shades the places where objects meet, and the sky is one
flat color. This plan fixes those three things.

## Goal

A player standing in the clearing sees a forest floor with real surface detail, objects that
sit in the ground instead of on it, and light that gives shapes form, while the forest stays
cold, hazy, and a little sinister.

## Out of scope

These are real gaps, but each is a larger job and belongs in its own plan:

- A custom foliage shader for a more painted look
- Wind in the grass and trees
- Normal maps for rocks and leaves
- Blending several ground materials in one surface

## Steps

Each step ends with the game running and its scenarios passing, so a bad result can be
traced to one change.

1. **Contact shadows:** Add Bevy's screen-space ambient occlusion (SSAO) to the camera. It
   needs multisampling off, so anti-aliasing has to change with it.
2. **Generated sky:** Add Bevy's procedural atmosphere, with `AtmosphereEnvironmentMapLight`
   as the source of sky light. It expects physically based light levels, so the sun,
   exposure, and flat ambient light all change in this step.
3. **Ground texture:** Tile ambientCG's `Ground037` across the ground. The painted colors
   stay as a tint, so the clearing, patches, and trail keep their layout.
4. **Mood pass:** Rebalance fog, saturation, and tint against the new lighting.

If the atmosphere can't be pushed towards the mood in step 2, a painted gradient sky is the
fallback. It keeps the rest of the plan intact, but it loses the generated sky light.

## How we know it worked

Rendering isn't covered by automated tests, so success has three layers. Automated
scenarios prove the wiring, measurements prove the picture changed, and a person judges the
result.

### Automated scenarios

These become `#[spec]` tests, in the crate that owns each behavior.

#### Ground (`world`)

```gherkin
Scenario: The ground can carry a tiled texture
  Given a forest
  When its ground mesh is built
  Then every point has texture coordinates
  And the texture repeats every configured tile size
  And every point has a tangent, so a normal map can light it

Scenario: The texture doesn't stretch up slopes
  Given two neighboring points of the ground
  When their texture coordinates are compared
  Then the distance between them follows the terrain's distance on the XZ plane

Scenario: The ground uses every map it was given
  Given a client with the world plugin
  When it starts
  Then the ground material has a color, normal, and roughness texture
  And those textures are loaded so that they repeat

Scenario: The painted layout survives
  Given a forest
  When the trail and the ground beside it are painted
  Then their tints still differ
```

#### Mood (`world`)

```gherkin
Scenario: A camera gets contact shadows
  Given a client with the world plugin
  When a camera appears
  Then it has ambient occlusion
  And multisampling is off

Scenario: The sky lights the world
  Given a client with the world plugin
  When a camera appears
  Then it has atmosphere settings and sky light
  And the world has one atmosphere
  And flat ambient light is off

Scenario: The sun matches the sky's light levels
  Given a mood
  When the world starts
  Then the sun's strength and the camera's exposure come from that mood
```

#### This game's choices (`client`)

```gherkin
Scenario: The ground's files exist
  Given this game's world plugin
  When its ground textures are compared with the asset folder
  Then each one exists
```

### Measured against the current game

A scripted client takes a screenshot at two fixed views, the clearing and further along the
path, before and after this work. A small script measures each one. Only the before and
after matter, so the same views and window size are used for both.

The baseline is a detail score of 4.07 in the clearing and 0.35 on the path, and an average
of 183 frames per second.

| Measure | Pass |
|---|---|
| Ground detail: brightness variation in a patch of ground | At least double the baseline, at both views |
| Crushed shadows: share of pixels that are almost black | No worse than the baseline of 0%, at both views |
| Frame rate | Stays at or above 60 frames per second on the development machine |
| `just dev` log | No warnings and no errors |

### Judged by a person

A measurement can't decide these, so the work isn't finished until you've seen them:

- Up close, the ground reads as a forest floor, not a flat color or a visibly repeating
  pattern.
- Rocks and trunks look planted in the ground.
- The photo-based ground doesn't clash with the stylized models.
- The character stays readable in shadow.
- It still feels cold and a little sinister, not bright and cheerful.

## Results

Steps 1 to 4 are built. Measured at the same two views and window size:

| Measure | Before | After | Target | Result |
|---|---|---|---|---|
| Ground detail, clearing | 4.07 | 10.86 | At least 8.14 | Met |
| Ground detail, path | 0.35 | 5.87 | At least 0.70 | Met |
| Near-black pixels, clearing | 0.0% | 0.0% | 0% | Met |
| Near-black pixels, path | 0.0% | 0.1% | 0% | **Missed** |
| Average frame rate | 183 | 151 | At least 60 | Met |
| `just dev` log | Clean | Clean | No warnings or errors | Met |

The missed target is a small one. The near-black pixels look like contact shadows at the base
of grass tufts, which step 1 adds on purpose, although that comes from viewing the screenshot
and wasn't measured. Removing them would mean brightening the whole scene against the mood,
so the target stands as missed until a person decides what to do with it.

The checks for a person are still open, and two risks need that review most: whether the
scanned ground clashes with the models, and whether the ground shimmers in motion. The
textures have no mipmaps, and still screenshots can't show that.

## Assets

`Ground037` from ambientCG: CC0, created by photogrammetry, with color, normal, and roughness
maps at 1K. It needs a row in `assets/SOURCES.md`. The sky is generated, so it adds no
files.

## Risks

- **Photo meets cartoon:** A scanned texture may fight the stylized models. Tinting it
  should help, and judging this is part of the person's review.
- **Physically based sky:** The atmosphere is built for realism, so a moody sky may take
  tuning. The gradient sky is the fallback.
- **Frame rate:** Contact shadows and a generated sky both cost performance, on top of about
  3,000 scattered props.
- **Anti-aliasing:** Turning multisampling off makes edges rougher, and leaves show it most.
  Another anti-aliasing method may be needed.
