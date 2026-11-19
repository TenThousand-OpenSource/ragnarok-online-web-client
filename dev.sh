#!/bin/sh
# For working on the client: builds it again whenever a file of it is saved.
#
# The gateway serves the built client from client/dist as it is on disk, so a build is in the browser at the
# next load of the page; opened with `?dev` in its address, the page loads itself anew when a new build is there
# (see index.html). That is a new page: the game is left and logged into again. A change is not put into a page
# that runs, because the game's state lives in the program being replaced.
#
# Only the client is watched. The gateway is a program of its own: after a change to it, start it again.
cd "$(dirname "$0")"

# The newest moment any of the client's files was saved, as a number.
saved() {
    find client/src client/web client/Cargo.toml -type f -newer client/dist/index.html 2>/dev/null | head -1
}

./build-web.sh || echo "dev: the build failed; waiting for the next save"
echo "dev: watching client/src and client/web (Ctrl+C to stop)"
while sleep 1; do
    if [ -n "$(saved)" ]; then
        echo "dev: $(saved) was saved; building"
        if ./build-web.sh; then
            echo "dev: built at $(date +%H:%M:%S)"
        else
            # So that the same save is not built over and over.
            touch client/dist/index.html
            echo "dev: the build failed; waiting for the next save"
        fi
    fi
done
