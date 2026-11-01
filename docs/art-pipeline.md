# Art pipeline

**Status:** In use, but not fully built. See [What's built](#whats-built). The look is set by
[the art bible](art/bible.md).

Start with the `art-integration` skill, which runs this whole process.

This is the process for getting new art into the game. It exists because the forest work went
straight from a downloaded pack to fixing what looked wrong, one surface at a time, without
ever deciding what the game should look like.

## How studios do it

Published detail on this is thin. The best studio talks are behind the GDC Vault, and most
open writing comes from outsourcing vendors and practitioners. Treat this as a consistent
pattern across those sources, not as a documented standard.

### It isn't a relay

Art, technical art, and integration don't run one after another. They run together in a short
phase that settles the look, and only then does production become a repeatable line. So there
are two different processes:

| | Look development | Production |
|---|---|---|
| Happens | Once per visual style | For every asset |
| Goal | Decide and prove what the game looks like | Make more of it, consistently |
| Art and tech | Work together on the same few assets | Tech is settled, and art follows the spec |
| Output | Art bible, technical spec, shaders, lighting | Assets that pass the gates |

### Operations

- **Exploration has a hard end.** Style gets its own phase, with a fixed end date, bounded by
  concept art and a few hero assets ([Juego Studios][juego]).
- **Hero assets are proven in the engine.** One character, one environment piece, and one prop
  test the direction on real geometry and lighting before more is built against it
  ([Juego Studios][juego]).
- **The art bible comes from that proof.** It's a working reference, with on-style and
  off-style examples, and it's amended when new cases appear ([Juego Studios][juego]).
- **Every asset passes gates.** Brief, block-out, detail pass, final delivery, and integration
  validation, with a review at each ([Lemonsky][lemonsky]).
- **Block-out is the most valuable gate.** It locks silhouette and proportion. A direction
  error costs a sketch there, and the whole asset at final delivery ([Lemonsky][lemonsky]).
- **Done means in the build.** An asset is finished only once it's placed in the game and
  checked on target hardware ([Lemonsky][lemonsky]).
- **One owner, one decision.** One person resolves feedback into approve, revise, or escalate
  ([Lemonsky][lemonsky]).
- **New asset types start small.** Each gets its own small calibration batch, and batches are
  sized to what can be reviewed ([Lemonsky][lemonsky]).

### Art direction

- **Shape first.** Blizzard's environments rely on strong silhouettes and shape language that
  say who lives there ([80 Level][vergne]).
- **Small noisy detail works against stylization.** Too much of it pushes a scene towards
  realism ([80 Level][vergne]).
- **Keep textures neutral.** Lighting and ambience carry the mood. Very dark or bright textures
  lose detail in shadow or strong light ([80 Level][vergne]).
- **Paint the light.** Hand-painted textures suggest materials by painting how light falls on
  them, often over a sculpt ([80 Level][vergne]).
- **Low detail can be a strength.** Iron Gate says Valheim's look speeds up asset creation and
  runs on older hardware, with ambient effects and post-processing making the world look
  richer ([PCGamesN][valheim]).

### Software architecture

- **Source and runtime are separate.** Runtime code reads cooked data, not authoring files, and
  the cook step is where inputs are checked ([NWBLot engine wiki][cook]).
- **Rules are code.** Unreal's Data Validation plugin checks names, budgets, and dependencies
  on save and in CI, and it has a pre-submit hook ([Epic][unreal]).
- **Import is scriptable.** Unity runs custom importers and post-processors as part of import
  ([Unity][unity]).
- **Only real problems block.** Defining severity levels up front separates blocking failures
  from warnings ([SEELE][seele]).
- **Texel density is set early.** Studios commonly set one early so surfaces seen together
  match ([RebusFarm][texel]).
- **Tiling and trim sheets keep things crisp.** They avoid huge unique textures, and second UV
  sets add detail on top ([Polycount][trim]).
- **Bevy already has this step.** Its asset processor writes processed files to
  `imported_assets`, and `.meta` files configure each asset ([Bevy][bevy]). Its image saver
  compresses textures to KTX2.

## What we got wrong

| What studios do | What we did | Result |
|---|---|---|
| Settle the look first | Started with a pack and no written target | Each fix is a guess at an unstated goal |
| Review hero assets in the engine | Judged one screenshot | Most assets were never inspected |
| Keep stylized surfaces clean | Put a photo scan on the ground, and baked scans into props | The look moved towards realism, and away from the goal |
| Judge against the goal | Scored surfaces by how much detail they had | The score rewarded noise, so the wrong fix looked like a success |
| Fix art at the source | Patched files in a temporary folder | Results can't be reproduced from the repository |
| Cook textures before runtime | Built mipmaps on the CPU at load | Wrong layer, and they don't reach the screen yet |
| Automate the rules | Wrote a one-off check per plan | Nothing stops the next asset going wrong |

## Our process

We're a small team, so these are roles, not people. The human is the art director and has final
sign-off. The agent does technical art and integration, and can generate textures under the rules
in `docs/assets.md`. It never decides what looks right.

### Look lock, once per style

1. Write a one-page art bible: pillars, shape rules, palette roles, material rules, lighting
   intent, and on-style and off-style references.
2. Pick three hero assets: the character, one tree, and one rock.
3. Build a look development scene with fixed lighting and fixed views.
4. Iterate on shaders, lighting, and art together until the art director signs off.
5. Write the technical spec from what worked, and update this document.

### Gates, for every asset

| Gate | Passes when | Who decides |
|---|---|---|
| 1. Brief | It says what the asset is for and points at the bible | Art director |
| 2. Blockout | A placeholder in the game proves its size and silhouette | Art director |
| 3. Source | The art is chosen or made, and its licence is recorded | Art director |
| 4. Cook | The pipeline builds it, and the automated rules pass | Automated |
| 5. Integration | It's correct in the look development scene and the real game | Agent |
| 6. Sign-off | It's in the build, and it matches the bible | Art director |

A pack counts as a new asset class, so it starts with a calibration batch of three models.

### Jobs

Each job is a skill in `.agents/skills/`. Sub-agents can't ask a person anything, and they
don't see the conversation, so `art-integration` and `art-look-lock` run in the main
conversation, and the others run as sub-agents and work only from the brief.

| Skill | Job | Runs as | Gate |
|---|---|---|---|
| `art-integration` | Runs the whole process and asks for each decision | Main conversation | All |
| `art-look-lock` | Agrees the look and writes the art bible | Main conversation | Look lock |
| `art-brief` | Writes the brief | Sub-agent | 1 |
| `art-blockout` | Puts a placeholder in the game | Sub-agent | 2 |
| `art-source` | Finds candidates and records licences | Sub-agent | 3 |
| `art-cook` | Builds the asset and runs the rules | Sub-agent | 4 |
| `art-integrate` | Wires the asset into its game module | Sub-agent | 5 |
| `art-review` | Checks the result independently, and changes nothing | Sub-agent | 5 |

`art-review` is separate from `art-integrate` so that no job marks its own work.

### Handoffs

The brief is the only thing jobs share. It lives at `docs/art/briefs/<name>.md`, starts from
[the template](art/brief-template.md), and holds the request, the requirements, and a log of
every gate. Each job reads it, does one gate, adds its evidence to the log, and reports one of:

| Result | Meaning |
|---|---|
| Passed | The gate's conditions are met, and the evidence is in the brief |
| Needs decision | A person has to choose, and the options are in the brief |
| Failed | The work doesn't meet the gate, and the brief says why |
| Blocked | Something the job needs doesn't exist yet |

Only `art-integration` asks for sign-off, and only a person can record it.

### Rules for every job

Each of these comes from something that went wrong on the forest work.

- **Show it on screen.** A change counts only if a before and after pixel comparison shows it.
  Tests, logs, and a texture having the right data in memory aren't proof.
- **Distrust the measurement first.** Two views with identical file sizes, or a surprising
  frame rate, usually mean the measurement is broken. Look at the image before the number.
- **Check every asset.** Inspect each one up close. One good screenshot says little.
- **Averages hide things.** Measure the surface a player sees, not the whole model.
- **Find the cause.** Name why something looks wrong before changing it. A surface that looks
  flat might have no texture, a bad texture, a low resolution, or a filtering problem.
- **Don't add scan noise to painted art.** Fix a stylized surface with shape, value, and light.
- **Measure against the bible.** More detail, or fewer dark pixels, isn't better on its own. A
  number can show that something changed, but only the bible says whether the change is good.
- **Keep every change reproducible.** Nothing important lives in a temporary folder.
- **Report misses.** Don't change a target to pass it. Record the miss for a person to decide.
- **Leave the machine alone.** Never stop a game someone else started, or use `--target-dir`.
  Ask them to close it. To run the server by hand, set `DAWN_ENV=development`.
- **Remember CI has no assets.** It checks out Git LFS pointers, so a test that reads a model
  or texture has to handle one.
- **Clean up.** Temporary tools never reach a commit.
- **Commit only when asked.** Jobs leave changes in the working tree.

### Architecture

```text
assets/        Source art, as downloaded or authored
    | cook: mipmaps, compression, density and budget rules
    v
imported_assets/   What the game reads. Generated, not committed.
```

- Every change to an asset is a committed recipe in the pipeline, never a patched file.
- Blocking rules: missing mipmaps, unknown licence, over budget, and missing files.
- Warning rules: texel density outside the range for that asset class.

### Technical spec

| Rule | Starting value |
|---|---|
| Texel density | At least 100 pixels per meter on large props. The ground has about 128. |
| Texture size | As `docs/assets.md`: 1K, or 2K for things seen up close |
| Mipmaps | Required on all 3D textures, once the cook step exists |
| Texture values | Neutral, so that lighting carries the mood |
| Frame rate | At least 200 frames per second in the real game, at 720p on the development machine |
| Shadows | Two layers at 2048 pixels, reaching 45 meters. Resolution is what costs the most. |

Measure frame rate on a window on the main monitor, because a window on the 59 Hz second monitor
is capped at 60. Trees cost far more than anything else, so check the frame rate after adding any.

### Definition of done

An asset is done when it's passed all six gates. Its review shows it at 1, 5, and 20 meters, in
sun and shadow, in motion as well as in stills.

We track revisions per asset and defects found after sign-off. If those climb, the bible or
the spec is at fault, not the asset.

## What's built

| Piece | State | Needed by |
|---|---|---|
| Skills, template, and this document | Done | Everything |
| Art bible, `docs/art/bible.md` | Done | Gates 1, 3, and 6 |
| Look development scene, `just lookdev` | Done | Gates 2 and 5 |
| Art recipes, `tools/art-bake` | Done for the forest | Gate 3 |
| Cook step and automated rules | Not built | Gate 4 |

`art-integration` stops at gate 4 until the cook step exists, and says so. To build it, test Bevy's
asset processor with one texture, and prove with a pixel comparison that its mipmaps reach the
screen. The mipmaps built at load in the local `wip/forest-bakes` branch never did.

## Art recipes

Generated art is rebuilt, never patched. `tools/art-bake/forest.json` holds each texture's prompt
and seed, and `forest.sh` generates them through a local ComfyUI and bakes them into `assets/`:

- The ground and bark use the generated texture directly.
- Rocks get a texture of their own, with stone projected onto them in 3D, so that nothing joins at
  UV edges.
- Leaves keep the kit's shapes exactly, with generated brushwork inside them.

Running the recipe reproduced every texture it was checked against, pixel for pixel. ComfyUI
sometimes crashes partway through. The recipe skips finished textures, so run it again.

To change art, change the recipe and run it. Keep the installation itself out of the repository.

## Look development scene

`just lookdev` shows the hero assets under the game's real mood. The arrow keys change the
model, 1 to 3 change the distance, and Tab swaps sun and shadow. The window's title names the
view.

`just lookdev --capture <folder>` saves every view at 1, 5, and 20 meters, in sun and in shadow,
and then exits. Keep the folder outside the repository. Name models after the options to show
those instead, such as `just lookdev --capture ../shots nature/Fern_1.gltf`.

- **Distances are real.** They're measured from the surface the camera faces.
- **Captures repeat exactly.** Two runs of the same view match pixel for pixel, so any difference
  between a before and an after comes from the change.
- **It doesn't show everything.** Characters stand in their rest pose, nothing moves, and the
  ground is bare and flat. Shimmer and how art sits in the forest still need the real game.

## Known problems

- **Rocks are soft up close.** They hold up at 5 meters, but not at 1. A detail layer or a small
  shader is the likely fix.
- **Some art doesn't match yet.** Grass, ferns, mushrooms, bushes, and dead trees are still the
  kit's art, and bark still uses the kit's normal maps.
- **Motion is unjudged.** The ground's bright flecks may look like a pattern while walking, and
  textures have no mipmaps yet, so distant surfaces may shimmer.
- **The sky hasn't been judged.** Bevy's atmosphere is physically based, so it needs testing
  against the mood.
- **Zones share one mood.** The bible expects each zone to have its own.
- **Art and gameplay layout are tied together.** Model paths and plantings live in
  `crates/protocol/src/forest.rs`, which the server also uses. Adding a planting anywhere but the
  end of the list moves everything after it, including solid trees. Separate these before adding
  many models.
- **The world sandbox has its own lighting.** Its mood differs from the game's, so look
  development has to use the game's real settings.
- **The game reads source files.** There's no asset processing yet, so `assets/` is what runs, and
  `imported_assets/` still has to be added to `.gitignore`.

[juego]: https://www.juegostudio.com/blog/art-direction-in-games
[lemonsky]: https://www.lemonskystudios.com/insights/game-art-outsourcing-pipeline
[vergne]: https://80.lv/articles/the-strings-of-environment-design/
[valheim]: https://www.pcgamesn.com/valheim/ps1-graphics
[cook]: https://github.com/limztudio/NWBLot/wiki/Start-Here
[unreal]: https://dev.epicgames.com/documentation/en-us/unreal-engine/data-validation-in-unreal-engine
[unity]: https://docs.unity.com/en-us/engine/current/manual/assets-and-media/asset-database/customizing-workflow
[seele]: https://www.seeles.ai/features/tools/dcc-automated-validation-pipeline-guide
[texel]: https://ja.rebusfarm.net/blog/texel-density-basics-every-artist-should-know
[trim]: https://polycount.com/discussion/comment/2778298/
[bevy]: https://docs.rs/bevy/latest/src/asset_processing/asset_processing.rs.html
