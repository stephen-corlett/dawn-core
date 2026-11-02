# World bible

**Status:** Signed off by the design director on 2026-11-01. The density and count numbers are still
starting values that nobody has played, so expect the first zone to change them.

This is the target every zone is judged against, as [the art bible](../art/bible.md) is for how
things look. For how a zone gets built, see [the zone pipeline](../zone-pipeline.md).

## The promise

Follow your instincts and you find things. The primary loop is quests from NPCs, but a player
should feel they earned each quest by going somewhere, not that the game sent them there.

## Pillars

- **Curiosity leads:** Something in view always makes the player want to walk over. A quest gives
  them a reason, and the world gives them a direction.
- **Small in a large world:** Distance is real. Crossing a zone is a journey, and travel tools
  shorten it without removing it.
- **Hand-crafted:** Everything that matters is where it is for a reason a person can state. This
  describes the result, so generated and scattered content is fine under the rules below.

## Zones, not one world

Elden Ring is the inspiration, but it isn't what we're building. It's one continuous world with a
single order-free progression, built on combat. We're an MMO of zones with level ranges, quest hubs,
and a quest-led loop, so we take its methods and rebuild what depended on its shape.

| What it gets from one world | What a zone does instead |
|---|---|
| Stuck in one place, a player goes somewhere else | A hub always offers several threads leading to different regions |
| The whole world is a single mental map | Each zone is a place a player can learn by heart, with its own tentpole |
| Distant places pull a player onward forever | World-scale landmarks and lookouts show the next zone before it's reached |
| A long-dead history felt across one space | Each zone tells its own history, and [the atlas](atlas.md) ties them together |

A zone is also a complete journey, with an arrival, a middle, and a far end. Each one needs its own
shape, which a single continuous world doesn't.

## Zone themes

The game is level-based, like World of Warcraft. Each zone has a fixed level range, and enemies keep
their levels, so a zone's danger can be read from the world and old zones become easy.

A zone has a theme, not a single storyline. The theme is what life is like there, and each quest
thread is its own small story that comes out of it.

- **Themes show on the land:** Hunger shows as thin fields and empty granaries, before anyone
  mentions it.
- **Threads stand alone:** A player can follow any of them in any order, and none has to be finished
  for the zone to make sense.
- **Stakes fit the place:** Near the human city, problems are small: the army is away at war and has
  no time for trouble at home, so aspiring adventurers step in. Scale grows farther from home.
- **Players earn their way up:** A starting zone doesn't need a world-changing mystery. Helping
  people is the point.

## Scale

Zones are measured in time on foot, not in meters.

| Rule | Starting value |
|---|---|
| Length of a zone | 15 to 30 minutes along the natural route between its far ends |
| Speed that's measured at | The speed a player has in that zone's level range |
| Footprint on foot | About 4 to 8 km across, if routes wind by a factor of 1.4 |

Time is the ruler because of the promise about travel: when players get faster, later zones grow
in meters and stay the same in minutes.

The natural route counts, not a straight line. At 6.4 m/s, 30 minutes in a straight line is an
11.5 km square, or 133 km². That's far more than fan measurements of all of Elden Ring, at about
79 km². It couldn't be filled by hand, and most of it would be empty. Winding routes around
mountains and water give the same walk from about half the area, or 67 km².

## Travel

- **One speed on foot:** Players move at 6.4 m/s, with no sprint. A held button would only make the
  zone shorter, so speed comes from what players earn.
- **Earned speed:** Mounts and movement speed items come later. Each one sets the speed of the zones
  after it, in [the atlas](atlas.md).
- **One joined world:** Zones join with no loading screen, so the walk between them is unbroken and a
  lookout can show the next zone for real.
- **No fast travel:** Traversing the world is part of the game. Flight paths and other transport may
  come later, and they still take time in the world.
- **Going home:** Players can return to a saved town or hub, as a hearthstone does in other MMOs. This
  only goes back, so reaching somewhere new always means traveling there.

## Density

Density is measured along routes, because a player experiences a route and not an area. These are
the longest gaps allowed, not a rhythm. A steady drip of content feels like a checklist, so quiet
stretches are fine, and a noticed thing is often only something to look at.

| Beat | Longest gap on a route | At 6.4 m/s |
|---|---|---|
| Something new to notice | Every 60 to 90 seconds | 385 to 575 m |
| A site worth stopping at | Every 3 to 5 minutes | 1.2 to 1.9 km |
| A change of region | Every 8 to 10 minutes | 3.1 to 3.8 km |

Between 40% and 60% of a zone's footprint isn't walkable: cliffs, deep water, and impassable woods.
This is what makes the world affordable. A player sees that land and feels its size, but nobody has
to fill it.

## How much a zone holds

For a zone of about 25 minutes:

| Thing | Count | Seen from | Built |
|---|---|---|---|
| Tentpole, the zone's one unmistakable sight | 1 | Open ground across the zone, 3 km or more | By hand |
| Major landmarks | 3 to 5 | 1 to 1.5 km | By hand |
| Town or village | At least 1 | 500 m or more, by smoke or a tower | By hand |
| Encampments, used sparingly | 0 to 2 | 300 m | By hand |
| Set pieces: a large, level-like place to conquer | 1 to 2, often a landmark | As their landmark | By hand |
| Sites: mines, tombs, and other places with a few rewards | 10 to 16 | 200 to 500 m | By hand, from kits |
| Vignettes: a ruined caravan, a lone grave, usually one reward or none | 30 to 50 | 50 to 100 m | Placed by hand, from kits |
| Regions, each with its own identity | 3 to 5 | | Painted by hand |

These counts are ceilings. Fewer and better beats more, and a site that repeats another site's
task should be cut.

To read at those distances, a shape has to rise above what's around it. At 720p, a clear shape takes
about 35 pixels, and one meter fills about 869 pixels divided by the distance. These minimums come
from grey shapes in [the scale test](scale-test.md), judged from stills:

| Thing | Seen from | At least |
|---|---|---|
| Tentpole | 3 km | 120 m, and 240 m reads far better |
| Major landmark | 1 to 1.5 km | 40 to 60 m |
| Town tower | 500 m | 18 m |
| Site | 200 m, or 500 m | 6 m, or 20 m |

- **Unusual means something:** Anything strange in the distance leads to something unique. If a
  player walks to an odd shape and finds nothing, they stop trusting the world.
- **No repeated chores:** No zone-wide tasks to clear, and no site type that exists to be counted.

## What's crafted, and what's scattered

| Layer | Examples | Who decides where |
|---|---|---|
| Authored | Landmarks, settlements, sites, roads, the big terrain shapes | A named decision with a reason |
| Directed | Where forests stop, which slopes are bare | Painted regions and rules |
| Ambient | Individual trees, grass, pebbles | Scattered inside what's directed |

Nothing that attracts or rewards a player may come from the ambient layer. A player can't tell who
placed one fern, but they can tell whether a place means something.

Each authored thing records why it's there: what it lures the player towards, or what it rewards.
If nobody can say, it comes out.

## Sight lines

- **Always a next thing:** From every place a player can stop, at least one unvisited thing is in
  view. At a junction, at least two compete.
- **Reveal, don't list:** Terrain hides as much as it shows. A hill or mountain draws the eye to its
  top and hides what's behind it, and cresting it shows the next thing.
- **Forest is enclosed:** Nothing far away can be seen from inside a forest, and that's part of its
  feel. The tentpole appears from hills, lookouts, and planned gaps, so each glimpse is earned. Lures
  inside a forest are close by, and use light, sound, or a path.
- **Use three sizes of shape:** Mountains are landmarks, hills hide things and frame routes, and
  rocks set the rhythm up close.
- **Silhouettes do the work:** A distant thing is read by its shape, smoke, or light. See the shape
  rules in [the art bible](../art/bible.md).
- **Going around is allowed:** The direct way and the way around both work, and each gets something
  the other doesn't.
- **Rewards draw players too:** Each landmark type should promise something players come to want.

## Settlements

- Every zone has at least one town or village as its main quest hub. Most are small, and a few
  critical zones get a slightly larger town.
- A player finds the hub, rather than starting in it. New characters start at a small encampment or
  home, and the arrival route brings the hub into view, so reaching it feels like the first discovery.
- From the edge of the hub, a player can see where most of its quests lead.
- Encampments are sparing. Each needs a reason a hub can't serve, such as a far-off region.
- A settlement says who lives there and how they live, by its shape and placement.

## Finding quests

| Source | How it's found | Share of a zone's quests |
|---|---|---|
| Hub NPCs | Reaching the town | Most |
| Notice boards | At hubs, roads, and crossings | Some |
| Situations in the world | Exploring: a ruined caravan starts a search for what happened | The ones players remember |

- **Quests are hung on places:** Terrain and sites come first, and a quest uses what's already
  there. Quests never need a new thing placed to explain them.
- **Never one way forward:** A player stuck on one quest always has another that goes somewhere else.
- **Give directions a person would give:** "Follow the river to the split tower", where that tower
  can be seen. Players have a map, but it shows no quest pins, and nothing on screen points the way.
  The world does the guiding.
- **A situation shows its own story:** It has a chain of events that a player can read without text,
  and a question that makes them look for the answer.

## Decided

The design director made these on 2026-11-01.

| Decision | Choice |
|---|---|
| What the 15 to 30 minutes measures | The natural winding route |
| Tentpoles and forest | Seen from open ground, not from inside forest |
| Joining zones | Seamless, with no loading screen |
| Sprinting | Removed, with players on foot at 6.4 m/s and faster travel earned later. |
| Levels | Fixed level ranges per zone, with enemies keeping their levels |
| Zone stories | A theme with separate threads, not a single storyline |
| Quest markers | A map, with no quest pins |
| Fast travel | None. A recall returns players to a saved hub, and transport like flight paths may come later. |
