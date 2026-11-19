#!/usr/bin/env python3
"""Checks what of ARCHITECTURE.md's rules a script can: the two sizes (rules 4 and 10: no source file past about
500 lines, no function past about 100; lines so long that they hide size are counted in a note), and that nothing on the page is named by hand (rule 12: what can be
pressed, used, dragged or sent is written through `web/parts.rs` from a value of `crate::press`; and the page's
style sheet names no window of the game to say how it lies over the others, which the window's entry in
`hud/windows.rs` says with its `layer`); and that the directory map of ARCHITECTURE.md is the one
`tools/directory_map.py` writes from the tree (rule 9). Run from the project's folder; `check.sh` does. Prints what breaks a
rule and leaves with 1 if anything does.

A function that only hands work out by kind (rule 10's exception) is named in DISPATCHERS, with its file.
"""
import glob
import re
import subprocess
import sys

FILE_LINES = 500
FUNCTION_LINES = 100
# A line longer than this hides size from the count of lines. Reported, not refused yet: there are some.
LINE_CHARACTERS = 200
# Plain dispatchers: a `match` over packets or actions whose arms are a line or two each, as (file, function).
# None is longer than a function may be now.
DISPATCHERS = set()
# A name of a press, a use, a drag or a form written into the page by hand.
BY_HAND = re.compile(r'data-(click|use|drag|submit)=\\?"[a-z{]')
# The one file that writes those names, from values.
WRITES_NAMES = "client/src/web/parts.rs"
# The page, and where the game's windows are listed: an entry names its element with `id: "…"`, or as the first
# thing given to what writes entries of one kind (`Window::question("…", …)`, `talk_window!(NAME, "…", …)`).
PAGE = "client/web/index.html"
WINDOWS = ["client/src/web/hud/windows.rs", "client/src/web/hud/windows/*.rs"]
WINDOW_ID = re.compile(r'(?:\bid: |Window::\w+\(\s*|talk_window!\(\w+, )"([a-z0-9-]+)"')
STARTS = re.compile(r"^(\s*)(?:pub(?:\([^)]*\))? )?(?:const )?(?:async )?fn (\w+)")


def functions(lines):
    """Each function of a file: its name, its first line (from 1) and how many lines it takes."""
    for at, line in enumerate(lines):
        found = STARTS.match(line)
        if not found:
            continue
        # One written on a single line (its braces all closed on it), or declared without a body (a trait's),
        # ends where it starts: there is no closing brace of its own to look for further down.
        opened = line.count("{")
        if (opened and opened == line.count("}")) or (not opened and line.rstrip().endswith(";")):
            yield found.group(2), at + 1, 1
            continue
        closing = found.group(1) + "}"
        end = next((end for end in range(at, len(lines)) if lines[end].rstrip() == closing), None)
        if end is not None:
            yield found.group(2), at + 1, end - at + 1


def layered_by_name():
    """The rules of the page's style sheet that lay a window of the game over or under the others by its name."""
    ids = {name for pattern in WINDOWS for path in glob.glob(pattern) for name in WINDOW_ID.findall(open(path, encoding="utf-8").read())}
    page = open(PAGE, encoding="utf-8").read()
    sheet = re.sub(r"/\*.*?\*/", "", "".join(re.findall(r"<style[^>]*>(.*?)</style>", page, re.S)), flags=re.S)
    for selector, rule in re.findall(r"([^{}]+)\{([^{}]*)\}", sheet):
        if "z-index" in rule:
            for name in sorted(ids):
                if re.search(r"#" + re.escape(name) + r"(?![\w-])", selector):
                    yield f"{PAGE}: #{name} is laid over the others by its name (give its entry in hud/windows.rs a `layer` instead)"


def main():
    over = []
    long_lines = []
    for path in sorted(glob.glob("client/src/**/*.rs", recursive=True) + glob.glob("gateway/src/**/*.rs", recursive=True)):
        lines = open(path, encoding="utf-8").read().split("\n")
        if len(lines) - 1 > FILE_LINES:
            over.append(f"{path}: {len(lines) - 1} lines (a file is at most about {FILE_LINES})")
        for name, at, long in functions(lines):
            if long > FUNCTION_LINES and (path, name) not in DISPATCHERS:
                over.append(f"{path}:{at}: fn {name} is {long} lines (a function is at most about {FUNCTION_LINES})")
        long_lines.extend((len(line), path, at + 1) for at, line in enumerate(lines) if len(line) > LINE_CHARACTERS)
        if path != WRITES_NAMES:
            for at, line in enumerate(lines):
                if BY_HAND.search(line) and not line.lstrip().startswith("//"):
                    over.append(f"{path}:{at + 1}: a name of the page written by hand (use `pressed`, `used`, `dragged` or `sent` of web/parts.rs)")
    over.extend(layered_by_name())
    # Rule 9: the directory map of ARCHITECTURE.md is the tree's own.
    if subprocess.run([sys.executable, "tools/directory_map.py", "--check"], capture_output=True).returncode:
        over.append("ARCHITECTURE.md: the directory map is not the tree's (run `python3 tools/directory_map.py --write`)")
    for line in over:
        print(line)
    if not over:
        print("rules: every file and function is within the limits, and no name of the page is written by hand")
    if long_lines:
        longest, path, at = max(long_lines)
        by_file = {}
        for _, name, _ in long_lines:
            by_file[name] = by_file.get(name, 0) + 1
        most = sorted(by_file.items(), key=lambda entry: -entry[1])[:3]
        print(f"note: {len(long_lines)} lines are longer than {LINE_CHARACTERS} characters and hide size from the count of lines "
              f"(the longest: {path}:{at}, {longest}; most in " + ", ".join(f"{name} ({count})" for name, count in most) + ")")
    return 1 if over else 0


if __name__ == "__main__":
    sys.exit(main())
