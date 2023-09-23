#!/bin/bash
# Copyright (C) 2023 taylor.fish <contact@taylor.fish>
#
# This file is part of Plumage.
#
# Plumage is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as published
# by the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# Plumage is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with Plumage. If not, see <https://www.gnu.org/licenses/>.

set -euo pipefail

USAGE=$(cat << EOF
Usage: $(basename "$0") <out-dir> <count>

Generates <count> images in <out-dir>.
\$PARALLEL controls number of jobs.
EOF
)

if [[ "${1-}" =~ ^(-h|--help)$ ]]; then
    printf '%s\n' "$USAGE"
    exit
fi

if [ -z "${2+.}" ]; then
    printf >&2 '%s\n' "$USAGE"
    exit 1
fi

dir=$1
count=$2
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
