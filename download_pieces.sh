#!/bin/bash

PIECES_DIR="static/assets/pieces"
PIECES="pawn knight bishop rook queen king"
COLORS="white black"

mkdir -p "$PIECES_DIR"

for color in $COLORS; do
    for piece in $PIECES; do
        url="https://raw.githubusercontent.com/lichess-org/lila/master/public/piece/cburnett/${color}-${piece}.svg"
        output="$PIECES_DIR/${color}-${piece}.svg"
        curl -o "$output" "$url"
        echo "Downloaded $output"
    done
done 