# Zone pipeline

**Status:** Designed, and not used yet. Gates 1 and 2 work today, because they only need documents.
See [What's built](#whats-built) for the rest.

Start with the `zone-design` skill, which runs this whole process.

This is how a zone gets from an idea into the game. The target is
[the world bible](world/bible.md), and zones fit together in [the atlas](world/atlas.md). Art inside
a zone goes through [the art pipeline](art-pipeline.md).

## How others do it

Published detail is thin, and much of this comes through secondary coverage. Treat it as a pattern
across sources, not as how any studio works today.

- **Lure, don't lead.** Breath of the Wild first connected its towers with routes. Testers felt
  railroaded, and a heat map showed about 80% following the route and 20% wandering. Replacing the
  routes with landmarks that each promise something removed that split ([GMTK][gmtk]).
- **Terrain hides as well as shows.** Nintendo's triangle rule, from a CEDEC 2017 talk, gives a
  choice of over or around, hides what's beyond, and works at three scales ([80 Level][triangle]).
  Large shapes act as landmarks, medium ones screen things, and small ones set the tempo
  ([Gigazine][gigazine]). The chain of reveals is a breadcrumb trail ([GMTK][gmtk]).
- **Rewards shape where players go.** Scarce resources give players reasons to head for forests and
  mountains, and each landmark type has a clear payoff ([GMTK][gmtk]).
- **Elden Ring navigates by landmark.** Its sight lines push players to find strong landmarks and
  steer by them ([Vice][vice]). Global ones like the great tree work across the world, and local
  ones help players remember a place ([Level Design Lobby][lobby]). This is critics' analysis. We
  found no account from the developers.
- **Elden Ring has fewer, better things.** It has fewer diversions than most open worlds, in three
  sizes: small areas with one reward, larger resource areas, and big set pieces. It avoids repeated
  task types, and doesn't deliver content as a steady drip ([Game Developer][ignoring]). Much of this
  relies on one connected world, so [the world bible](world/bible.md) says what zones do instead.
- **Story comes first, and shows in the land.** Blizzard's zone process starts by asking what story
  to tell, then expresses it through terrain, ecology, and settlements. Designers also pace how long
  players stay in one kind of place, with quest designers ([Artcraft][artcraft]). Blizzard's level
  and quest designers work together on where players go ([Engadget][draenor]).
- **Size has a limit.** One area of Mists of Pandaria's Isle of Thunder was judged too big and open
  to work for quests ([Blizzard Watch][thunder]).
- **Environments tell stories by inference.** A place staged as a discernible chain of events makes
  the player work out what happened, which is why it engages them ([Smith and Worch][worch]).
- **Kits make large worlds possible.** Bethesda builds with modular kits, shared across the world by
  type, such as caves and Nordic forts ([Burgess][burgess]).
- **Hand-crafted worlds use procedural fill.** Horizon Zero Dawn places all its nature procedurally,
  from artist-written rules and paintable maps, around hand-made content ([Guerrilla][guerrilla]).

## Which order

Three questions decide the process.

**Is story first?** A theme is, on a single page, along with enough history to explain the land.
Geography is built from history, so a zone with no "what happened here" ends up with arbitrary
terrain. Detail waits, because quests are hung on places that have to exist first.

**Start from one point, or from key points?** Both, at different times:

1. **Plan from the outside in.** Sight lines are a property of the whole zone. Growing outward from
   one point gives even, wandering results, because each step can only see its neighbors. So the
   tentpole, landmarks, hub, and routes are placed first, across the whole zone, on paper.
2. **Build from the arrival inward.** The first ten minutes are what a player judges the zone by, and
   they prove whether its ideas work. So detail starts where players arrive and goes outward one
   region at a time, each region correcting the plan for the next.

**What does hand-crafted mean for an agent?** Agents are good at producing variety, and bad at
judging experience or holding a reason for every placement. So:

- **A zone is a recipe:** It's text files of named decisions, each with its reason. Nothing important
  lives only in a binary, and changes can be reviewed and reversed.
- **Agents propose, in options:** A person chooses a direction at each gate, the way an art director
  does, and never gets a single take to approve.
- **Rules become measurements:** Walk times, sight lines, and gaps between beats can be checked by
  code on every change. Those measurements are necessary, and they don't prove a place is good.
- **Agents look at what they build:** Maps and views from a player's eye are the evidence, because an
  agent reading coordinates can't tell what a place feels like.
- **The builder doesn't mark its own work:** A separate agent that has never read the plan walks the
  zone from views alone and says where it wants to go. Where that differs from the plan, the plan
  isn't working.
- **People walk it last:** Only a person playing the real game can pass a zone.

## Our process

The human is the design director and art director, and has final sign-off. The agent builds,
measures, and offers options. It never decides what's good.

### The stages

| Stage | Scope | Output |
|---|---|---|
| Plan | The whole zone | Gates 1 and 2: premise and paper skeleton |
| Prove | The whole zone, roughly | Gate 3: terrain and grey shapes in the game, walkable end to end |
| Fill | One region at a time, from the arrival | Gates 4 to 6: sites, dressing, independent walk |
| Finish | The whole zone | Gate 7: a person walks it |

### Gates

| Gate | Passes when | Who decides |
|---|---|---|
| 1. Premise | The zone's theme, threads, and place in the atlas fit on a page | Person |
| 2. Skeleton | One of several paper layouts is chosen, and its sight graph has no dead ends | Person |
| 3. Blockout | Grey terrain and landmarks are in the game, and meet the walk and sight targets | Person |
| 4. Sites | The region's sites, settlements, vignettes, and quest hooks are placed in grey | Person |
| 5. Dressing | The region has its kits, mood, and scatter, through the art pipeline | Art pipeline, then person |
| 6. Walk | An independent walker's route and report are in the brief | `zone-walk` |
| 7. Sign-off | A person has walked the zone in the real game | Person |

Gates 4 to 6 repeat for each region. The first region is a calibration batch, like a new pack in
the art pipeline, so no other region starts until a person has walked it.

Grey comes first because, as with art, a block-out is where a wrong direction is cheapest. Don't
dress a region to rescue it. If a place is dull in grey, fix its shape.

### Jobs

Each job is a skill in `.agents/skills/`. As in the art pipeline, sub-agents can't ask a person
anything and don't see the conversation, so `zone-design` runs in the main conversation.

| Skill | Job | Runs as | Gate |
|---|---|---|---|
| `zone-design` | Runs the whole process and asks for each decision | Main conversation | All |
| `zone-premise` | Offers premises and writes the chosen one | Sub-agent | 1 |
| `zone-skeleton` | Offers paper layouts and their sight graphs | Sub-agent | 2 |
| `zone-blockout` | Builds grey terrain and landmarks, and measures them | Sub-agent | 3 |
| `zone-sites` | Places a region's sites, vignettes, and quest hooks | Sub-agent | 4 |
| `zone-dressing` | Plans a region's kits, mood, and scatter | Sub-agent | 5 |
| `zone-walk` | Walks the zone without knowing the plan, and changes nothing | Sub-agent | 6 |

`zone-walk` can't read the brief. That's the only thing making it independent, so its instructions
are strict about it.

### Handoffs

The zone brief is what jobs share. It lives at `docs/world/zones/<name>.md`, and starts from
[the template](world/zone-template.md). Each job does one gate and adds evidence to the gate log. Jobs
report Passed, Needs decision, Failed, or Blocked, with the meanings in the art pipeline.

### Rules for every job

"Rules for every job" in the art pipeline apply here too. These add to them.

- **Show the view.** A sight line counts when a capture from a player's eye shows it. A ray test can
  find problems, but it can't show that a thing reads at that distance.
- **State the reason.** A placement with no reason is removed, however good it looks.
- **Measure the route.** Check gaps along routes a player would take, not averages over the zone.
- **Don't scatter what matters.** Nothing that lures or rewards a player comes from noise.
- **Keep the plan and the zone together.** When a build changes a landmark, update the brief in the
  same change.
- **Report misses.** Don't change a target to pass it. Only a person can agree a change, and it goes
  in the brief's Decisions.
- **Keep secrets out of public files.** The repository is public, so spawn tables, rewards, and
  hidden logic never go in a brief or anything clients read.

## The gap to today's game

Each of these is a measured fact about the forest. Forest densities are scaled to a 5 km square.

| What | Today | A 5 km zone needs |
|---|---|---|
| Walkable area | 100 m across, 19 seconds at a run | 50 times wider, 2,500 times the area |
| View distance | Fog ends at 85 m | 1 to 3 km for landmarks, so 12 to 35 times farther |
| Ground mesh | One mesh with 1 m cells, about 26,000 cells | 25 million cells at that detail, so level of detail |
| Props | About 4,000, all made at startup | About 8 million at this density, so streaming and thinner far fill |
| Collision | About 300 solids, each checked on every move | About 340,000, so a spatial index |
| Terrain | Noise and code | Authored shapes in a data file |
| Mood | One for the whole game | One per zone, with blending |
| NPCs and quests | None | New game modules |

[The scale test](world/scale-test.md) showed the view is affordable, in a test scene with no
characters, network, or interface, and with motion unjudged. A 5 km ground with landmarks at 3 km
averaged 225 to 251 frames per second at 720p in the worst views. Real trees are the limit, crossing
200 at about 5,000 within a view, so stand-ins past the near forest are required. Ground detail and
stand-ins cost almost nothing.

It also settled the conflicts with the rules we had:

- **Shadows:** Lower quality with distance is how shadow layers already work. Each covers more ground
  with the same pixels, which Unity calls staged reductions in detail ([Unity][cascades]), and Bevy
  gives every layer the same size ([Bevy][bevy-cascade]). Three layers reach 3 km at a cost of 5 to
  10%, two lose the trees' shadows, and four miss the target. Unity notes that distant objects don't
  need to be in the shadow map at all ([Unity][cascades]), and Unreal can use distance fields past its
  shadow layers ([Epic][distance]). So if margins get thin, the next step is dropping small things
  from distant shadows, or baking shade into far ground, which the scale test tried.
- **Fog:** One distance fog can't give both the forest's mood and a 3 km view, so we'll build fog that
  follows a place, thick in forest and thin above it. Until then, the closest match is holding fog at
  70% and leaving the far world out of it, which hazes near trees more than distant land.
- **Frame rate:** The 200 target means an average. Today's 1% lows are 145 to 162, so report them too.
- **Forest and tentpoles:** A player can't see 3 km from inside a forest, and that's accepted. Seeing
  over 15 m trees takes about 190 m of open ground for a 240 m tentpole, and 375 m for a 120 m one.

Coordinates are fine within a zone, because 32-bit floats are accurate to about 1 mm at 10 km. They
reach about 8 mm at 100 km, so zones joined into one world may need a floating origin. `big_space`
does this for Bevy, but we haven't checked that it supports Bevy 0.20. Lightyear's rooms can limit
what each player receives, which an MMO this size will need.

## Architecture

```text
assets/zones/<zone>/      Shape and dressing, as text. Clients and the server may read shape.
  shape                   Terrain strokes, blocked land, and solids. Both sides need these.
  dressing                Regions, scatter rules, and models. Only clients need these.
Server-only data folder   Spawns, quest rewards, and anything clients must never see.
docs/world/zones/<zone>.md  Why everything is where it is.
```

- **Three kinds of data:** Shape is shared, dressing is client-only, and quest secrets are server-only.
  This also solves the known problem that art and gameplay layout are tied together in
  `crates/protocol/src/forest.rs`.
- **Terrain is a recipe:** The shape file is an ordered list of named strokes, such as a ridge along a
  line, a valley, a plateau, or a riverbed, with a reason on each. Fine noise is added on top. Both
  sides build the same result, as they do with the forest today.
- **Tools live in `world`:** `xtask` can't depend on game crates, so zone checks belong in a `world`
  example, run from `just`.
- **New crates pass the tests:** Quests and NPCs each have to pass "What makes a game module".
- **The formats aren't decided.** Work them out when building zone data, and update this section.

## What's built

| Piece | State | Needed by |
|---|---|---|
| Skills, templates, and this document | Done | Everything |
| World bible, `docs/world/bible.md` | Done | Gate 1 |
| Scale test, [report](world/scale-test.md) | Done for frame rate. Motion and the real game are unproven. | Gate 3 |
| Zone data: shape strokes in a file, read by both sides | Not built | Gate 3 |
| Far view: ground level of detail and forest stand-ins | Proven in the scale test, not in the game | Gate 3 |
| Solids in a spatial index | Not built | Gate 3 |
| `zone-view`: top-down maps, and views from a player's eye at named places | Not built | Gates 3 to 6 |
| `zone-check`: route times, sight graph, and gaps between beats | Not built | Gates 3 and 4 |
| Dressing separated from shape | Not built | Gate 5 |
| A mood per zone | Not built | Gate 5 |
| Fog that follows a place | Not built | Gate 5 |
| Removing sprint, with 6.4 m/s on foot | Done | Gate 3, for route times |
| Prop and terrain streaming | Not built | Gate 5, at full size |
| Seamless joins between zones | Not built | The second zone |
| NPC and quest game modules | Not built | Playable quests, not their placement |

Build in that order.

## Starting the first zone

1. Sign off the world bible.
2. Run gates 1 and 2 for the first zone. They need nothing built.
3. Build zone data, the far view, `zone-view`, and `zone-check`.
4. Run gate 3 on a calibration zone of about 3 km, with a 10-minute route holding the arrival, the
   hub, and one landmark.
5. Take that through gate 7, and fix this process using what went wrong.
6. Only then build at full size.

[cascades]: https://docs.unity3d.com/2018.2/Documentation/Manual/DirLightShadows.html
[bevy-cascade]: https://doc.qu1x.dev/bevy_trackball/bevy/light/cascade/index.html
[distance]: https://dev.epicgames.com/documentation/unreal-engine/using-distance-field-shadows-in-unreal-engine
[gmtk]: https://gmtk.substack.com/p/how-nintendo-solved-zeldas-open-world
[triangle]: https://80.lv/articles/the-design-secrets-of-breath-of-the-wild/
[gigazine]: https://gigazine.net/gsc_news/en/20171004-breath-of-the-wild-design-secret
[ignoring]: https://www.gamedeveloper.com/game-platforms/how-elden-ring-succeeds-by-ignoring-20-years-of-open-world-design
[vice]: https://www.vice.com/en/article/elden-ring-review
[lobby]: https://videohighlight.com/v/LSN5LQrjKMw
[artcraft]: https://en.guiaswow.com/warlords-of-draenor/artcradt.html
[draenor]: https://www.engadget.com/2014-10-30-world-of-warcraft-chronicles-the-challenges-of-draenor-zone-desi.html
[thunder]: https://blizzardwatch.com/2016/11/06/wows-level-designers-telling-stories-landscapes
[worch]: https://slidetodoc.com/level-design-workshop-what-happened-here-environmental-storytelling/
[burgess]: https://gamedeveloper.com/design/skyrim-s-modular-approach-to-level-design
[guerrilla]: https://www.guerrilla-games.com/read/gpu-based-procedural-placement-in-horizon-zero-dawn
