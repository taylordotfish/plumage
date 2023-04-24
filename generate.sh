#!/bin/bash
set -euo pipefail

USAGE=$(cat << EOF
Usage: $(basename "$0") [out-dir [count]]
\$PARALLEL controls number of jobs.
EOF
)

if [[ "${1:-}" =~ -h|--help ]]; then
    echo "$USAGE"
    exit
fi

dir=${1:-.}
count=${2:-100}
fmt_len=$(printf '%s' "$count" | wc -c)
parallel=${PARALLEL:-$(nproc)}

binary=target/release/plumage
if ! ([ -e "$binary" ] || binary=$(which plumage)); then
    echo >&2 'Error: Could not find `plumage` in $PATH or ./target/release'
    exit
fi

gen-chunk() {
    local start=$1
    local end=$2
    for i in $(seq -f "%0$fmt_len.0f" $start $end); do
        "$binary" $dir/out$i
        convert $dir/out$i.bmp $dir/out$i.png
        rm $dir/out$i.bmp
        echo $i
    done
}

mkdir -p "$dir"
chunk_len=$(($count / $parallel))
extra=$(($count - $chunk_len * $parallel))
current=1
for i in $(seq 0 $(($parallel - 1))); do
    end=$(($current + $chunk_len + ($i < $extra)))
    gen-chunk $current $(($end - 1)) &
    current=$end
done
wait
