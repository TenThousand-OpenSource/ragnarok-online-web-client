#!/bin/sh
# Sets the rules the game is played by, and rebuilds and restarts the server for them:
#
#   ./rules.sh classic     the classic game (pre-renewal)
#   ./rules.sh renewal     Renewal: third and fourth jobs, their monsters, items, skills and NPCs
#
# What it writes is server/.env: the server's build, its image and its database of characters (each set of rules
# has its own, an item or a job of one not existing in the other). The gateway reads the same file for the item
# database that goes with the rules; restart it afterwards. The first build of a set of rules takes a while.
set -e
cd "$(dirname "$0")"
case "$1" in
  classic) printf 'ROWEB_PRERE=yes\nROWEB_IMAGE=prere\nROWEB_DB_VOLUME=server_db\n' > server/.env ;;
  renewal) printf 'ROWEB_PRERE=no\nROWEB_IMAGE=re\nROWEB_DB_VOLUME=server_db_renewal\n' > server/.env ;;
  *) echo "usage: $0 classic|renewal" >&2; exit 1 ;;
esac
docker compose -f server/docker-compose.yml up -d --build
echo "The server plays by the $1 rules now. Restart the gateway."
