# Assets

Game assets are human-made, free to use, and as small as they can be. The goals are
no storage bill, a fast download for contributors and players, and a clear licence for
everything that ships.

Nothing here is checked automatically, so these rules need human judgment during code
review.

## What goes where

| What | Where |
|---|---|
| Assets the game uses | `assets/`, stored with Git LFS |
| Where each asset came from | [assets/SOURCES.md](../assets/SOURCES.md), as normal text |
| Original downloads and zips | A local folder outside the repository |
| Blender source files | Only if you've edited them and need to keep the changes |

`.gitattributes` sends binary asset types to Git LFS. If you add a new file type, add
it there before the first commit that uses it, or the file goes into normal Git history
for good.

Nothing loads from `assets/` yet. Bevy reads from the folder of the crate being run,
such as `crates/client/assets/`, unless `BEVY_ASSET_ROOT` or `AssetPlugin::file_path`
says otherwise. Set this up in the apps when the first asset goes in.

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

- **No other licences:** Don't use CC-BY or any other licence, even from an approved
  library. If an asset isn't marked CC0, leave it out.
- **No AI-generated assets:** Leave out anything that's labeled as AI-generated or looks
  like it might be.
- **Check each asset:** A site's general licence isn't enough. Confirm the licence on
  the asset you're downloading.
- **If nothing fits:** Ask a human instead of looking elsewhere. Neither library is a
  full source of characters or animations, so those need a decision first.
- **Record each download:** Add one row to `assets/SOURCES.md` with the asset, source
  link, and licence. CC0 needs no credit, but the record shows an asset's licence and
  that it isn't AI-generated, and it tells us what to replace if a source turns out to
  be a problem.

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
