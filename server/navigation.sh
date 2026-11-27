#!/bin/sh
# Makes the navigation's tables for THIS server's world: who stands where, the ways out of each map and the
# monsters, as the server's own scripts have them (rAthena's `map-server-generator --generate-navi`,
# ../../rathena/src/map/navi.cpp). The client's own tables describe the Korean service's Renewal world.
#
#   server/navigation.sh        writes server/navigation/pre-re/navi_*_krpri.lub (or …/re/, by server/.env)
#
# The running game is not touched: the generator is built into an image of its own and run once, with the same
# conf, scripts, databases and map cache as docker-compose.yml gives the map server, against an empty database of
# its own (it would clear the tables of the shops left open of a real one). Run it again whenever the server's
# scripts, map_conf.txt or map cache change, then restart the gateway (it reads the folder: `--navigation-dir`).
# It takes some minutes; the first build of the generator takes longer.
set -e
cd "$(dirname "$0")"
[ -f .env ] && . ./.env
if [ "${ROWEB_PRERE:-yes}" = no ]; then rules=re; else rules=pre-re; fi
image="roweb/rathena-navigen:20260219-${ROWEB_IMAGE:-prere}"
out="$PWD/navigation/$rules"
net=roweb-navigen
db=roweb-navigen-db
moved=

docker build -f navigation/Dockerfile --build-arg PRERE="${ROWEB_PRERE:-yes}" -t "$image" ../../rathena

cleanup() { docker rm -f "$db" >/dev/null 2>&1 || true; docker network rm "$net" >/dev/null 2>&1 || true; [ -z "$moved" ] || rm -f "$moved"; }
trap cleanup EXIT
cleanup
docker network create "$net" >/dev/null
docker run -d --name "$db" --network "$net" --network-alias db \
  -e MARIADB_ROOT_PASSWORD=ragnarok -e MARIADB_DATABASE=ragnarok -e MARIADB_USER=ragnarok -e MARIADB_PASSWORD=ragnarok \
  -v "$PWD/../../rathena/sql-files/main.sql:/docker-entrypoint-initdb.d/01-main.sql:ro" \
  -v "$PWD/../../rathena/sql-files/logs.sql:/docker-entrypoint-initdb.d/02-logs.sql:ro" \
  mariadb:11 >/dev/null
until docker exec "$db" healthcheck.sh --connect --innodb_initialized >/dev/null 2>&1; do sleep 2; done

# The same files docker-compose.yml gives the map server: conf/ as conf/import, db/ as db/import, npc/ as
# npc/custom, and the map cache.
mounts="-v $PWD/mapcache/map_cache.dat:/rathena/db/import/map_cache.dat:ro"
for file in conf/*.txt conf/*.yml; do mounts="$mounts -v $PWD/$file:/rathena/conf/import/${file#conf/}:ro"; done
for file in db/*; do mounts="$mounts -v $PWD/$file:/rathena/db/import/${file#db/}:ro"; done
for file in npc/*; do mounts="$mounts -v $PWD/$file:/rathena/npc/custom/${file#npc/}:ro"; done

mkdir -p "$out"
# Whoever npc/roweb_classic.txt moves as the server starts (`movenpc`) is written into the generator's own copy
# of the scripts where they are moved to: it lists everyone by the cell their script names, not by where they
# stand afterwards.
moved="$out/moved.sed"
python3 - npc/roweb_classic.txt > "$moved" <<'PY'
import re, sys
for line in open(sys.argv[1], encoding='latin1'):
    move = re.match(r'\s*movenpc "([^"]+)",(\d+),(\d+),(\d+);\s*// (\w+)', line)
    if move:
        name, x, y, facing, where = move.groups()
        name = re.sub(r'([.*\[\]\\/^$])', r'\\\1', name)
        print(f's/^{where},[0-9]*,[0-9]*,[0-9]*\\(\t[^\t]*\t{name}\\(::[^\t]*\\)\\{{0,1\\}}\t\\)/{where},{x},{y},{facing}\\1/')
PY
mounts="$mounts -v $moved:/moved.sed:ro"
folder="generated/clientside/data/luafiles514/lua files/navigation"
# The scripts are put right first, as the map server's own command does it.
# shellcheck disable=SC2086
docker run --rm --network "$net" $mounts -v "$out:/out" "$image" sh -c "
  find npc -name '*.txt' -not -path 'npc/custom/*' | xargs sed -i -f npc/custom/roweb_classic_warps.sed -f /moved.sed;
  mkdir -p '$folder' && ./map-server-generator --generate-navi && cp '$folder'/navi_*_krpri.lub /out/"
ls -l "$out"
echo "The navigation's tables of the $rules server are in $out. Restart the gateway."
