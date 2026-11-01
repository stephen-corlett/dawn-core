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
| Text data that only a sandbox reads | `crates/<module>/assets/`, as normal text |
| Where each pack came from | [assets/SOURCES.md](../assets/SOURCES.md), as normal text |
| Downloaded packs and zips | A local folder outside the repository |
| Blender source files | Only if you've edited them and need to keep the changes |

`.gitattributes` sends binary asset types to Git LFS. If you add a new file type, add
it there before the first commit that uses it, or the file goes into normal Git history
for good.

Nothing loads from `assets/` yet. Bevy reads from the folder of the crate being run,
such as `crates/client/assets/`, unless `BEVY_ASSET_ROOT` or `AssetPlugin::file_path`
says otherwise. Set this up in the apps when the first asset goes in. Sandboxes don't
need this, because each already runs from its own crate's folder.

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

Don't add AI-generated assets. Open-upload sites don't always label them, so prefer
sources where one creator or curator makes or checks the work.

| Source | Use it for | Licence |
|---|---|---|
| [Quaternius](https://quaternius.com) | Characters, monsters, nature, and animations | CC0 |
| [KayKit](https://kaylousberg.itch.io) | Stylized characters and environment kits | Free packs are CC0, but check each pack |
| [Kenney](https://kenney.nl/assets) | Filler and prototype kits | CC0 |
| [Poly Haven](https://polyhaven.com) | Textures, HDRIs, and models | CC0 |
| [ambientCG](https://ambientcg.com) | PBR materials | CC0 |

- **Visual style:** Pick one main creator's style, and only use others to fill gaps.
- **Allowed licences:** Use CC0 where possible. CC-BY is allowed if the pack's row in
  `assets/SOURCES.md` names the author, and the shipped game credits them. Don't use
  anything that bans commercial use or redistribution, because the game has to ship.
- **Check each asset:** A site's general licence isn't enough. Confirm the licence on
  the asset or pack you're downloading.
- **Record each pack:** Add one row to `assets/SOURCES.md` with the pack, source link,
  and licence. CC0 needs no credit, but the record shows an asset's licence and that it
  isn't AI-generated, and it tells us what to replace if a source turns out to be a
  problem.

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
