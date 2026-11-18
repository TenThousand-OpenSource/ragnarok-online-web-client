#!/bin/sh
# Opens the local server to invited friends through a Cloudflare quick tunnel (cloudflared must be installed).
#
# The gateway is started asking for the invitation word (server/public/invite.txt, made on the first run) and
# counting visitors by their own addresses; the tunnel gives it an address on trycloudflare.com that lasts until
# this script is stopped (Ctrl+C stops both). The address is new every time.
#
# Before the first time: change the passwords of the test accounts of server/conf/accounts.sql in the database.
# They are for a server nobody else can reach.
set -e
cd "$(dirname "$0")"

mkdir -p server/public
if [ ! -s server/public/invite.txt ]; then
    (umask 077; printf '%s-%s-%s\n' "$(openssl rand -hex 3)" "$(openssl rand -hex 3)" "$(openssl rand -hex 3)" > server/public/invite.txt)
    echo "the invitation word is in server/public/invite.txt"
fi

cargo build --release -p roweb-gateway
target/release/roweb-gateway --invite-file server/public/invite.txt --behind-proxy on &
gateway=$!
trap 'kill $gateway 2>/dev/null' EXIT INT TERM

cloudflared tunnel --url http://127.0.0.1:8080
