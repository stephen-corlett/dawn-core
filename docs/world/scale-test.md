# Scale test

**Result:** Yes for frame rate, with limits. On this machine, a 5 km ground with landmarks readable
at 3 km held an average above 200 frames per second at 720p, in every view, with shadows reaching
3 km. It's a test scene, not the real game, and frames dip below 200 here just as they do in today's
world. Fog is the harder problem: the forest's mood and a 3 km view can't both come from one
distance fog.

This records a test run on 2026-11-01, as evidence for [the world bible](bible.md) and
[the zone pipeline](../zone-pipeline.md). Its numbers come from when sprinting still existed. The
captures and the test's code were throwaway, so they aren't in the repository.

## What was built

A throwaway example in `crates/world`, not game code.

| Layer | Distance | How |
|---|---|---|
| Real forest | Inner 160 m | Today's 4,015 props and 1 m ground, through `WorldClientPlugin` |
| Ground | To 5 km | Tiles that halve with distance, 1.2 m cells at best, about 420 tiles and 870,000 triangles, built off the main thread |
| Near stand-ins | To about 1 km | Two stacked five-sided cones, merged into one mesh per 312 m |
| Far stand-in | Past about 780 m | The ground lifted 11 to 18 m and painted as canopy, with no trees |
| Landmarks | 300 m to 3 km | Grey cones of known size |

Each row below is the mean of about six seconds, in a 1280 x 720 window on the 240 Hz main monitor,
in the `dev` profile that `just dev` uses, with vsync off. Each setup was a separate process.

## Frame rates

"Clearing" is the eye of a player in today's clearing. "Lookout" stands 34 m above the same forest,
with all of it and all of the far world in view, which is the worst case.

| Setup | Clearing | Lookout | Clearing, 1% low | Meets 200 |
|---|---|---|---|---|
| Today's world, the control (two runs) | 241, 253 | | 145, 162 | Yes |
| Far world, shadows to 85 m (three runs) | 249 to 250 | 261 to 265 | 158 to 164 | Yes, but breaks the shadow rule |
| **Far world, three layers to 3 km (two runs)** | **225, 236** | **240, 251** | **132, 158** | **Yes, and keeps the rule** |
| Two layers to 3 km | 265 | 296 | 173 | Yes, but near shadows are lost |
| Three layers to 1 km | 221 | 229 | 138 | Yes |
| Four layers to 3 km | **188** | 204 | 126 | **No** |
| 2,500 real pines to 1 km | 224 | 233 | 143 | Yes |
| 10,000 real pines to 1 km | **171** | **174** | 119 | **No** |
| 40,000 real pines to 1 km | **91** | **86** | 73 | **No** |

Facing away from the real forest, the far world alone runs at 500 to 535 with shadows to 85 m, and
383 to 459 with shadows to 3 km. Walking the 2.3 km sight line at 40 m/s, about five times a sprint,
averaged 440 with shadows to 3 km, and the worst frame took 8 ms.

Runs of the same setup differ by about 3 to 5%, so smaller gaps mean nothing. The control's 1% low
already misses 200. The target doesn't say whether it means an average, so that's a question below.

## What worked

- **Ground detail costs almost nothing.** Between 214 and 766 tiles, or 0.4 to 1.6 million
  triangles, frame rate stayed within noise. A tile builds in 1.2 ms off the main thread.
- **Stand-in trees cost almost nothing, and real ones are the limit.** Cones to 1 km, cones to 3 km,
  and no forest at all landed within noise. Real pines cross 200 at about 5,000, interpolated. At
  today's density, 1 km around a player holds about 43,000, so stand-ins aren't optional. These
  stand-ins are only for frame rate: nobody has judged how they meet the real trees.
- **Fog costs nothing.** Every fog falloff measured the same, so fog is purely an art decision.
- **The canopy shell reads as forest,** and doesn't shimmer in stills. Motion hasn't been judged.

One real problem appeared on the way, and is worth keeping in mind.

- **Bevy throttles unfocused windows to 60 frames per second.** This looks exactly like a monitor
  cap, and it may be what the art pipeline's note about the 59 Hz monitor really saw. The first
  runs read 60 whenever the window lost focus. The test now keeps running at full speed.

## The fog conflict

Fog at 85 m hides everything past it. Thinner fog makes the far world visible, but loses the depth
in the forest.

Contrast is the brightness of a landmark against the land behind it, seen from 2.9 km.

| Fog | 240 m spire | 120 m spire | Forest's mood |
|---|---|---|---|
| Today's, ending at 85 m | 0%, gone | 0%, gone | Kept |
| Exponential, 3 km visibility | 0%, gone | 0%, gone | Lost |
| Exponential, 8 km visibility | 6% | 5% | Lost |
| Today's, held at 70% | 7% | 6% | Mostly kept |
| Today's, held at 50% | 14% | 10% | Partly kept |
| **Today's, held at 35%** | **20%** | **15%** | Weak |
| Today's, held at 20% | 30% | 20% | Nearly gone |
| **Held at 70%, far world left out of it** | **52%** | **29%** | **Kept** |

The strength of a fog can be capped with the alpha of its color, which needs no new shader. Bevy's
atmosphere already adds real haze with distance.

Leaving the far world out of fog is the best result from inside the forest:
its clearing is close to today's. It has two faults. From
a lookout, the nearby trees are hazier than the distant
land, which is backwards. And bright far ground shows through gaps in the trees. At full strength
the forest turns into a flat shape, so that's ruled out.

The real fix is probably fog that follows a place, thick inside forest and thin above it. Bevy's
distance fog can't do that, so it needs a shader, which this test didn't build. Tinting low far
ground towards the fog color looks like valley haze,
for free, on the far ground only.

## The shadow conflict

The rule is that shadows reach as far as the fog. With thin fog, that means 3 km.

- **It can be kept.** Three layers at 2048 pixels
  reach 3 km and look like the 85 m version near the player.
  It costs about 5 to 10%, and stays above 200 in the worst view.
- **Two layers fail on looks, not speed.** The second layer stretches from 10 m to 3 km, so
  the trees lose their shadows. It was actually
  the fastest setup, probably because foliage fills less of the shadow map. That cause is a guess.
- **Four layers miss the target** at 188.

So the rule doesn't need changing at this density. "Two layers" in the technical spec does, because
two can't cover 3 km. Three-layer margins are thin, at 225 to 240 before characters are added.

Baking hillside shade into far ground colors worked as a cheaper stand-in, but changes little
in this light. Keep it in reserve.

## Sight targets

At 720p with the game's camera, one meter is about 869 pixels divided by the distance. These
sizes are what's in the captures. "Reads" is my judgment from stills, for a person to confirm. About
35 pixels gave a clear shape, and about 17 gave a tick that's easy to miss.

| Target in the bible | Result | Evidence |
|---|---|---|
| Tentpole, 3 km or more | **Holds in open ground**, at about 120 m above its surroundings, and 240 m reads far better | 2.9 km |
| Tentpole, seen from most of the zone | **Misses in forest** | Nothing shows from the clearing |
| Major landmarks, 1 to 1.5 km | **Holds** at about 40 m at 1 km and 60 m at 1.5 km. 30 m at 1.5 km is too small. | From 900 m and 1.4 km |
| Town, 500 m or more, by tower | **Holds** for an 18 m tower. Smoke wasn't tried. | Lookout |
| Sites, 200 to 500 m | **Holds at 200 m** for a 6 m shape, which is only a tick at 300 m. A clear shape at 500 m needs about 20 m. | From 200 m |
| Encampments, 300 m | **Not tested.** By the same scale, a clear shape needs about 12 m. | By calculation |
| Vignettes, 50 to 100 m | **Misses under today's fog**, which hides anything past 85 m. Not tested on its own. | By calculation |

The forest miss comes from geometry, not rendering. To see over a 15 m tree, a player needs about
190 m of open ground in front of a 240 m tentpole, or about 375 m in front of a 120 m one. From
today's clearing, with trees 20 m away, nothing at 3 km can be seen. So forest needs planned gaps,
or the target has to allow for it.

## What this doesn't prove

- **It isn't the real game.** There are no characters, network, or interface. These add the same
  cost as they do today, and the margin at the 3 km shadow setup is 12 to 25%.
- **The near forest doesn't follow the player.** Prop streaming isn't built, so the walk only
  exercises the ground and stand-ins. Nothing tested a town or a second forest in view.
- **The landmarks are side-lit.** Nothing was tested looking into the sun, and the spires stand
  against land, not sky.
- **Motion hasn't been judged.** Stand-ins, the switch from real trees to cones, and tile changes may
  pop. A person needs to walk it. Textures also stop at the near mesh, so far ground is flat color.

## What was decided

- **Fog:** We'll build fog that follows a place.
- **Tentpole:** It's seen from open ground, and forest stays enclosed.
- **Frame rate:** The 200 target means the average.
- **Shadows:** Zones with long views use three layers.
- **Sizes:** The bible states minimum heights as starting values.
