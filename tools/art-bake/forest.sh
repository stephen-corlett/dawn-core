#!/bin/bash
# Rebuilds the forest's generated art from forest.json and the Quaternius kit's original files.
#
# Run it from the repository root, with ComfyUI running, and SDXL and its tiling add-on installed:
#   bash tools/art-bake/forest.sh <work folder outside the repository> <ComfyUI folder>
set -e

WORK=$1
COMFY=$2
# The last commit with the kit's unchanged files.
KIT=b6d73c1
ASSETS=assets/nature

python=$COMFY/python_embeded/python.exe
"$python" -I tools/art-bake/generate.py tools/art-bake/forest.json "$WORK/generated" \
    "$COMFY/ComfyUI/output"

cargo build --quiet --release --manifest-path tools/art-bake/Cargo.toml
bake=tools/art-bake/target/release/art-bake.exe

mkdir -p "$WORK/kit"
kit_file() {
    git cat-file --filters "$KIT:$ASSETS/$1" > "$WORK/kit/$(basename "$1")"
}

"$bake" recompress "$WORK/generated/ground.png" assets/ground/Painted_Ground_Color.jpg
"$bake" solid 128 128 255 assets/ground/Flat_Normal.jpg
"$bake" solid 0 220 0 assets/ground/Painted_Ground_Roughness.jpg

for n in 1 2 3; do
    kit_file "Rock_Medium_$n.gltf"
    kit_file "Rock_Medium_$n.bin"
    "$bake" triplanar "$WORK/kit/Rock_Medium_$n.gltf" "$WORK/kit/Rock_Medium_$n.bin" \
        "$ASSETS/Rock_Medium_$n.gltf" "$ASSETS/Rock_Medium_$n.bin" "$WORK/generated/stone.png" \
        3.0 2048 "textures/Rock_Medium_${n}_Color.jpg" "$ASSETS/textures/Rock_Medium_${n}_Color.jpg"
done

"$bake" recompress "$WORK/generated/bark_warm.png" "$ASSETS/textures/Bark_NormalTree.jpg"
"$bake" recompress "$WORK/generated/bark_twisted.png" "$ASSETS/textures/Bark_TwistedTree.jpg"

for leaf in Leaf_Pine_C Leaves_NormalTree_C Leaves_TwistedTree_C; do
    kit_file "textures/$leaf.png"
    "$bake" leaves "$WORK/kit/$leaf.png" "$WORK/generated/foliage.png" 1 0.9 0.5 \
        "$ASSETS/textures/$leaf.png"
done
