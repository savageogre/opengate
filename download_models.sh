#!/bin/bash
set -euo pipefail

modeldata=$(cat <<'EOF'
en,en_US,kristin,medium
en,en_US,amy,medium
en,en_US,reza_ibrahim,medium
EOF
)

function download-model() {
    shortlang="$1"
    lang="$2"
    name="$3"
    size="$4"
    url="https://huggingface.co/rhasspy/piper-voices/resolve/main/$shortlang/$lang/$name/$size/$lang-$name-$size.onnx"
    lpath="./models/$lang-$name-$size.onnx"
    if [[ ! -f "$lpath" ]] ; then
        echo "Getting $lpath from $url"
        wget "$url" -O "$lpath"
        wget "$url.json" -O "$lpath.json"
    else
        echo "Skipping $lpath because it looks like you have it."
    fi
}

mkdir -p models
echo "$modeldata" | while IFS=',' read -r shortlang lang name size; do
    download-model "$shortlang" "$lang" "$name" "$size"
done
