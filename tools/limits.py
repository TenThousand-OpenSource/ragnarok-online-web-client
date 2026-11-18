#!/usr/bin/env python3
"""Checks what of ARCHITECTURE.md's rules a script can: the two sizes (rules 4 and 10: no source file past about
500 lines, no function past about 100), and that nothing on the page is named by hand (rule 12: what can be
pressed, used, dragged or sent is written through `web/parts.rs` from a value of `crate::press`). Run from the
project's folder; `check.sh` does. Prints what breaks a rule and leaves with 1 if anything does.

A function that only hands work out by kind (rule 10's exception) is named in DISPATCHERS, with its file.
"""
import glob
import re
import sys

FILE_LINES = 500
FUNCTION_LINES = 100
# Plain dispatchers: a `match` over packets or actions whose arms are a line or two each.
DISPATCHERS = {
    ("client/src/web/app/incoming.rs", "packet"),
    ("client/src/web/app/actions.rs", "handle"),
}
# A name of a press, a use, a drag or a form written into the page by hand.
BY_HAND = re.compile(r'data-(click|use|drag|submit)=\\?"[a-z{]')
# The one file that writes those names, from values.
WRITES_NAMES = "client/src/web/parts.rs"
STARTS = re.compile(r"^(\s*)(?:pub(?:\([^)]*\))? )?(?:const )?(?:async )?fn (\w+)")


def functions(lines):
    """Each function of a file: its name, its first line (from 1) and how many lines it takes."""
    for at, line in enumerate(lines):
        found = STARTS.match(line)
        if not found:
            continue
        closing = found.group(1) + "}"
        end = next((end for end in range(at, len(lines)) if lines[end].rstrip() == closing), None)
        # One written on a single line, or declared without a body, ends where it starts.
        if end is not None:
            yield found.group(2), at + 1, end - at + 1


def main():
    over = []
    for path in sorted(glob.glob("client/src/**/*.rs", recursive=True) + glob.glob("gateway/src/**/*.rs", recursive=True)):
        lines = open(path, encoding="utf-8").read().split("\n")
        if len(lines) - 1 > FILE_LINES:
            over.append(f"{path}: {len(lines) - 1} lines (a file is at most about {FILE_LINES})")
        for name, at, long in functions(lines):
            if long > FUNCTION_LINES and (path, name) not in DISPATCHERS:
                over.append(f"{path}:{at}: fn {name} is {long} lines (a function is at most about {FUNCTION_LINES})")
        if path != WRITES_NAMES:
            for at, line in enumerate(lines):
                if BY_HAND.search(line) and not line.lstrip().startswith("//"):
                    over.append(f"{path}:{at + 1}: a name of the page written by hand (use `pressed`, `used`, `dragged` or `sent` of web/parts.rs)")
    for line in over:
        print(line)
    if not over:
        print("rules: every file and function is within the limits, and no name of the page is written by hand")
    return 1 if over else 0


if __name__ == "__main__":
    sys.exit(main())
