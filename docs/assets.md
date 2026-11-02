# Assets

Game assets are free to use and as small as they can be. The goals are
no storage bill, a fast download for contributors and players, and a clear licence for
everything that ships.

Nothing here is checked automatically, so these rules need human judgment during code
review. For the process of getting art into the game, see
[docs/art-pipeline.md](art-pipeline.md).

## What goes where

| What | Where |
|---|---|
| Assets the game uses | `assets/`, stored with Git LFS |
| Art grouped for reuse | `assets/kits/<kit>/`. See [Asset kits](#asset-kits). |
| A zone's layout, as text | `assets/zones/<zone>/`, once the [zone pipeline](zone-pipeline.md) builds it |
| Text data that only a sandbox reads | `crates/<module>/assets/`, as normal text |
| Where each asset came from | [assets/SOURCES.md](../assets/SOURCES.md), as normal text |
| Original downloads and zips | A local folder outside the repository |
| Blender source files | Only if you've edited them and need to keep the changes |

`.gitattributes` sends binary asset types to Git LFS. If you add a new file type, add
it there before the first commit that uses it, or the file goes into normal Git history
for good.

Bevy reads from the folder of the crate being run, such as `crates/client/assets/`,
unless `AssetPlugin::file_path` says otherwise. The client takes its folder from
`DAWN_ASSET_FOLDER`, which defaults to `../../assets` in development. A sandbox that
draws game assets passes the same path to `SandboxPlugin`, and one that only reads its
own text data keeps using its crate's folder.

## Asset kits

Art is grouped into kits by what it is, not by which zone first used it. A zone lists the kits it
uses in [the atlas](world/atlas.md), so kits can be shared and a zone's download stays known.

| Kind | Examples | Shared by |
|---|---|---|
| Common | Props and small rocks that fit anywhere | Every zone |
| Biome | Ground, plants, trees, and rocks for one kind of land | Zones with that land |
| Culture | The buildings and belongings of one people | Everywhere they live |
| Signature | A zone's landmarks and anything unique to it | One zone |

- **Layout:** Each kit has `assets/kits/<kit>/`, and its generated art has a recipe in
  `tools/art-bake/<kit>.json`.
- **New kits:** A new kit is a new asset class, so it starts with a calibration batch in
  [the art pipeline](art-pipeline.md). Reuse a kit if one fits, because a look is more consistent and
  LFS bandwidth is limited.
- **Today's art:** `assets/nature/` and `assets/ground/` predate kits. Move them when a second biome
  arrives, not before, along with the paths in `crates/protocol/src/forest.rs`.

## Keeping files small

- **Textures:** Use 1K JPG by default, and 2K for things players see up close, such as
  characters. Don't commit 4K or 8K textures.
- **Material maps:** Keep only the maps a material uses, usually color, normal, and
  roughness.
- **Models:** Use `.glb` or `.gltf`, which Bevy loads directly.
- **Commit late:** Only commit an asset once the game uses it. Deleting a file doesn't
  reliably free LFS storage, so experiments and old versions keep counting against the
  quota.

## Sources and licences

Only use assets from the approved libraries below, and only under CC0. Agents must not
pull assets from anywhere else, even if the licence looks fine. Adding a library or
allowing another licence needs a human's approval and a change to this file.

| Approved library | Use it for | Licence |
|---|---|---|
| [Poly Haven](https://polyhaven.com) | Textures, HDRIs, and models | CC0 |
| [ambientCG](https://ambientcg.com) | PBR materials | CC0 |
| [Quaternius](https://quaternius.com) | Stylized models, characters, outfits, and animations | CC0 |

- **No other licences:** Don't use CC-BY or any other licence, even from an approved
  library. If an asset isn't marked CC0, leave it out.
- **No downloaded AI art:** Leave out anything from a library that's labeled as AI-generated
  or looks like it might be, because its source can't be checked. We generate our own instead,
  as described in [Generating art](#generating-art).
- **Check each asset:** A site's general licence isn't enough. Confirm the licence on
  the asset you're downloading.
- **Quaternius has two licences:** Newer content can use the Quaternius Asset License
  instead, which isn't CC0. Read `License.txt` inside the download, and only use a pack
  that says CC0 there and on its own page.
- **If nothing fits:** Ask a human instead of looking elsewhere.
- **Record each download:** Add one row to `assets/SOURCES.md` with the asset, source
  link, and licence. CC0 needs no credit, but the record shows an asset's licence and
  how it was made, and it tells us what to replace if a source turns out to be a problem.

## Generating art

This is a hobby project with no planned release, so we can generate textures with an AI image
model running on a local machine. A person still approves everything that goes in the game.

- **Licence:** The model's licence has to allow this use, and it's recorded with the texture.
- **Style sources:** Don't train on other people's art or feed it in, and don't name living
  artists or other games in prompts. A style comes from CC0 art, or from art that's ours.
- **Record each texture:** Add one row to `assets/SOURCES.md` with the model, prompt, and seed.
- **Commit late:** Only commit final textures, because experiments still count against LFS.
- **If this changes:** Review all generated art before any public release. Store disclosure and
  copyright rules apply to it, and this exception assumes that nothing is being sold.

## Staying inside the free tier

GitHub Free includes 10 GiB of LFS storage and 10 GiB of LFS bandwidth a month.
Bandwidth is the tighter limit, because every full download of the assets uses it,
including downloads of this public repository by other people.

- **Spending:** Keep the account's LFS budget at $0. Going over the limit then blocks
  LFS until the next month instead of creating a bill.
- **CI:** Leave `lfs` off in `actions/checkout`. A job that needs assets should fetch
  only those files, because CI downloads count against the same bandwidth.
- **Outgrowing it:** Move to an object store such as Cloudflare R2, which has free
  download traffic, when LFS keeps hitting the limit. Don't use LFS to deliver the game
  to players.
