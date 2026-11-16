#!/bin/sh
# Runs run.sh in a container that has rAthena's build dependencies, and writes the table for the client.
#   tools/packet-lengths/docker-run.sh [packetver]
set -eu
HERE="$(cd "$(dirname "$0")" && pwd)"
VER="${1:-20260219}"
docker run --rm -v "$HERE/../../../rathena:/rathena:ro" -v "$HERE:/tool" alpine:3.23 sh -c \
  "apk add --no-cache bash g++ python3 mariadb-dev pcre-dev linux-headers zlib-dev >/dev/null && mkdir /work && cp /tool/*.cpp /tool/*.py /tool/run.sh /work/ && bash /work/run.sh /rathena /tool/out.txt $VER"
mv "$HERE/out.txt" "$HERE/../../client/src/net/packet_lengths.txt"
