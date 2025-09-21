#!/bin/bash
set -euo pipefail

modeldata=$(cat <<'EOF'
en_US,amy,medium
en_US,reza_ibrahim,medium
EOF
)

function download-model() {
    lang="$1"
    name="$2"
    size="$3"
    url="https://huggingface.co/rhasspy/piper-voices/resolve/main/en/$lang/$name/$size/$lang-$name-$size.onnx"
    lpath="./models/$lang-$name-$size.onnx"
    echo "Getting $lpath from $url"
    wget "$url" -O "$lpath"
    wget "$url.json" -O "$lpath.json"
}

mkdir -p models
echo "$modeldata" | while IFS=',' read -r lang name size; do
    download-model "$lang" "$name" "$size"
done
