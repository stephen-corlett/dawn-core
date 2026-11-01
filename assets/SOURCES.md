# Asset sources

Add one row per download or generated texture, in the same commit as the asset. See
[docs/assets.md](../docs/assets.md) for the rules.

| Asset | Source | Licence |
|---|---|---|
| `animations/UAL1_Standard.glb`: Universal Animation Library, Standard | [itch.io](https://quaternius.itch.io/universal-animation-library) | CC0 |
| `characters/Male_Head`: Universal Base Characters, Standard. Cut down to the head, eyes, and brows of `Superhero_Male_FullBody`, with unused vertex attributes removed. | [itch.io](https://quaternius.itch.io/universal-base-characters) | CC0 |
| `characters/Hair_Beard`: Universal Base Characters, Standard | [itch.io](https://quaternius.itch.io/universal-base-characters) | CC0 |
| `characters/Male_Ranger`: Modular Character Outfits - Fantasy, Standard, with unused vertex attributes removed | [itch.io](https://quaternius.itch.io/modular-character-outfits-fantasy) | CC0 |
| `ground/Painted_Ground_Color`: Generated with SDXL 1.0. The prompt and seed are in `tools/art-bake/forest.json`. | Generated | CreativeML Open RAIL++-M model |
| `ground/Flat_Normal`, `ground/Painted_Ground_Roughness`: Solid colors from `tools/art-bake/forest.sh` | Made here | None needed |
| `nature/`: Stylized Nature MegaKit, Standard. Changed files are listed below. | [itch.io](https://quaternius.itch.io/stylized-nature-megakit) | CC0 |
| `nature/Rock_Medium_*`, `nature/textures/Rock_Medium_*_Color`: The kit's rocks, with a generated stone texture projected onto them and their UVs rescaled. `Rocks_Diffuse` was removed. | Generated, `tools/art-bake/forest.json` | CC0, and CreativeML Open RAIL++-M model |
| `nature/textures/Bark_NormalTree`, `Bark_TwistedTree`: Generated, replacing the kit's color textures | Generated, `tools/art-bake/forest.json` | CreativeML Open RAIL++-M model |
| `nature/textures/Leaf_Pine_C`, `Leaves_NormalTree_C`, `Leaves_TwistedTree_C`: The kit's leaf shapes, with generated brushwork inside them | Generated, `tools/art-bake/forest.json` | CC0, and CreativeML Open RAIL++-M model |
