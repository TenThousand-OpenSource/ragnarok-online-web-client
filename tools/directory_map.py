#!/usr/bin/env python3
"""Writes the directory map of ARCHITECTURE.md from the tree itself: every file of `client/src`, `gateway/src`
and `gateway/tests` on a line of its own, alphabetical within its directory, with the first sentence of its
module comment (`//!`), cut to the line; a file's whole explanation stays in the file. What is no source of the two programs (the
scripts, the tools, the server's folder) is the short list `REST` below, kept by hand.

    python3 tools/directory_map.py            prints the map
    python3 tools/directory_map.py --write    puts it into ARCHITECTURE.md, between the two marks there
    python3 tools/directory_map.py --check    leaves with 1 if ARCHITECTURE.md does not have this very map

`tools/limits.py` (and so `check.sh`) runs the check: a file added, renamed or given another first sentence
needs `--write` run once (rule 9: a change leaves the documents true). Run from the project's folder.
"""
import os
import re
import sys

DOCUMENT = "ARCHITECTURE.md"
BEGIN = "<!-- directory map: written by tools/directory_map.py --write; change a file's `//!`, not these lines -->"
END = "<!-- end of the directory map -->"
ROOTS = ["client/src", "gateway/src", "gateway/tests"]
WIDTH = 115
# What stands beside the two programs' sources.
REST = """\
client/web/index.html   the page: its elements and its one style sheet (a layer of the windows has one rule)
client/dist/            what build-web.sh makes of the client: the gateway serves it (not committed)
server/                 a Docker setup of rAthena and MariaDB for development: docker-compose.yml, Dockerfile,
                        conf/ (the server's settings, groups.yml, the test accounts of accounts.sql),
                        npc/ (the classic scripts put right for the client's maps: tools/client-tables/
                        classic_overlay.py writes them), mapcache/ and public/ (made locally, not committed)
                        navigation.sh with navigation/Dockerfile (rAthena's generator of the navigation's
                        tables, run once beside the game) and navigation/pre-re/ (the tables it writes of
                        this server's world: made locally, not committed; the gateway's --navigation-dir)
translations/           GLOSSARY.md (terms, so that translators agree); src/ (English sources of the
                        translated batches); ru/ (Russian: items-*.json, skills-*.json, and labels.json: the
                        words of an English item description, for the items not translated yet)
docs/                   DONE.md (what is implemented, where from, how far checked), DEBTS.md (deviations
                        from the Windows client, bugs known, refactoring left), a picture for the README
ARCHITECTURE.md  CLAUDE.md  README.md  BACKLOG.md
                        this map and the rules; the rules in short for agents; what the project is and how
                        to run it; what waits for Renewal
build-web.sh            builds the client for the browser into client/dist
dev.sh                  the same again whenever a file of the client is saved; a page opened with ?dev reloads
check.sh                what a change passes before it is committed: tests, the browser build, tools/limits.py
rules.sh                switches the local server between the classic rules and Renewal's
public.sh               opens the local server to invited friends: the invitation word, a Cloudflare tunnel
tools/
  limits.py             rules 4, 10 and 12 as far as a script can hold them (files, functions and lines of
                        `src` and `tests`), and that this map is the tree's
  duplicates.py         measures rule 11: blocks of lines written in two places (`--loose`: but for their
                        names and literals) and functions of one shape (`--shape`); `--max N` fails over N
  directory_map.py      writes this map
  split_module.py       moves functions of a file that grew too large into child modules
  packet-names.py       writes out what rAthena's headers call each packet (client/src/net/packet_names.txt)
  packet-lengths/       builds the table of packet lengths from rAthena's sources
  client-tables/        writes out tables compiled into the Windows client, each script one or two files:
                        skill_effects.py (client/src/effect_tables/), statuses.py (statuses.txt), commands.py
                        (commands.txt), job_bases.py (web/data/job_bases.txt), job_mounts.py
                        (job_mounts.txt), ground_scopes.py (effect_tables/ground_scopes.txt); and for the
                        server: classic_overlay.py (server/npc/roweb_classic*), placement_check.py
"""


def sentence(path):
    """The first sentence of a source file's module comment, on one line."""
    name = os.path.basename(path)
    if name == "tests.rs" or name.endswith("_tests.rs"):
        return "tests"
    words = []
    for line in open(path, encoding="utf-8"):
        if not line.startswith("//!"):
            break
        line = line[3:].strip()
        if not line and words:
            break
        words.append(line)
    text = " ".join(word for word in words if word)
    if not text:
        return "(no module comment)"
    end = re.search(r"[.:;](?= |$)", text[40:])
    return text[: 40 + end.start()] if end else text


def other(path, sources):
    """What a file that is no source is: who reads it."""
    name = os.path.basename(path)
    readers = sorted(source for source, text in sources.items() if f'"{name}"' in text or f'/{name}"' in text)
    return "read by " + ", ".join(os.path.basename(reader) for reader in readers[:3]) if readers else "a table kept beside the code"


def listed(root, sources):
    """The lines of one root: a file, then (indented) what is in the folder of its name; folders after files."""
    lines = [root + "/"]

    def walk(folder, depth):
        names = sorted(os.listdir(folder))
        files = [name for name in names if os.path.isfile(os.path.join(folder, name)) and not name.startswith(".")]
        folders = [name for name in names if os.path.isdir(os.path.join(folder, name))]
        for name in files:
            path = os.path.join(folder, name)
            says = sentence(path) if name.endswith(".rs") else other(path, sources)
            head = "  " * depth + name
            first = head + " " * max(2, 22 + 2 * depth - len(head))
            room = WIDTH - len(first)
            lines.append(first + (says if len(says) <= room else says[: room - 1].rsplit(" ", 1)[0] + "…"))
            child = os.path.join(folder, name[:-3])
            if name.endswith(".rs") and name[:-3] in folders:
                walk(child, depth + 1)
        for name in folders:
            if name + ".rs" not in files:
                lines.append("  " * depth + name + "/")
                walk(os.path.join(folder, name), depth + 1)

    walk(root, 1)
    return lines


def drawn():
    sources = {}
    for root in ROOTS:
        for folder, _, names in os.walk(root):
            for name in names:
                if name.endswith(".rs"):
                    path = os.path.join(folder, name)
                    sources[path] = open(path, encoding="utf-8").read()
    lines = []
    for root in ROOTS:
        lines.extend(listed(root, sources))
    return "```\n" + "\n".join(lines) + "\n" + REST + "```"


def main():
    block = BEGIN + "\n" + drawn() + "\n" + END
    if "--write" in sys.argv or "--check" in sys.argv:
        document = open(DOCUMENT, encoding="utf-8").read()
        begin, end = document.find(BEGIN), document.find(END)
        if begin < 0 or end < 0:
            print(f"{DOCUMENT}: the marks of the directory map are not there")
            return 1
        written = document[begin : end + len(END)]
        if "--check" in sys.argv:
            if written != block:
                print(f"{DOCUMENT}: the directory map is not the tree's (run `python3 tools/directory_map.py --write`)")
                return 1
            return 0
        open(DOCUMENT, "w", encoding="utf-8").write(document[:begin] + block + document[end + len(END) :])
        return 0
    print(block)
    return 0


if __name__ == "__main__":
    sys.exit(main())
