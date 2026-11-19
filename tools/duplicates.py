#!/usr/bin/env python3
"""Measures what is written twice (ARCHITECTURE.md, rule 11) in `client/src` and `gateway/src`.

Run from the project's folder:

    python3 tools/duplicates.py                 # blocks of 6 and more significant lines found in two places
    python3 tools/duplicates.py --lines 8       # the same with a longer least block
    python3 tools/duplicates.py --loose --lines 10   # blocks that are the same but for their names and literals
                                                # (many are only alike: read them, do not count them)
    python3 tools/duplicates.py --shape         # functions of one shape: the same tokens once every name and
                                                # every literal is taken for "a name" and "a literal"
    python3 tools/duplicates.py --no-tests      # leave out `tests.rs` files and what follows `#[cfg(test)]`
    python3 tools/duplicates.py --most 40       # print the 40 largest groups only (the total counts them all)
    python3 tools/duplicates.py --max 150       # leave with 1 if more lines than that are repeated

How blocks are found. A line is *significant* unless it is empty, a comment, an attribute, a `use`, a `mod name;`, only braces
and punctuation, or only literals (a row of a table: numbers, texts, brackets and commas). The significant lines of
a file are compared with their whitespace taken out, so indentation and spacing do not count; how a statement is
divided into lines does. Every run of `--lines` significant lines that stands in two or more places is a clone;
clones that follow one another in all their places are joined into one group, so a group is reported once, at its
full length. A group's lines are given as they stand in the file (from its first significant line to its last).

The total is the number of significant lines that could go if every group were written once:
`lines of the group x (places - 1)`, summed.

`--shape` is the second measure. It finds functions that are the same but for their names and literals, which
the first cannot see: each function's tokens are taken, names (not keywords) become `N`, numbers and texts `L`,
and functions of `--tokens` tokens and more with the same result are a group.

Not every group is a debt: two things that are alike today and follow different rules of the Windows client are
left apart, and `docs/DEBTS.md` ("Refactoring left undone on purpose") says which and why. `--max` holds the
number where it is, once what is real has been merged.
"""
import argparse
import glob
import hashlib
import re
import sys

ROOTS = ["client/src", "gateway/src"]
COMMENT = re.compile(r"^\s*//")
# Nothing but brackets and punctuation: `}`, `});`, `},`, `) {`, `];`…
BRACES = re.compile(r"^[\s{}()\[\];,]*$")
# A text, a character or a number, as they are written in a table of literals.
LITERAL = re.compile(r'b?r#*"(?:[^"\\]|\\.)*"#*|b?"(?:[^"\\]|\\.)*"|b?\'(?:[^\'\\]|\\.)\'|\b\d[\d_]*(?:\.\d+)?(?:e-?\d+)?(?:[iuf]\d+|usize|isize)?\b|\b0x[0-9a-fA-F_]+\b')
NAME = re.compile(r"[A-Za-z_]")
TOKEN = re.compile(
    r'b?r#*"(?:[^"\\]|\\.)*"#*|b?"(?:[^"\\]|\\.)*"|\'(?:[^\'\\]|\\.)\'|\'[a-z_]\w*|'
    r"0x[0-9a-fA-F_]+|\d[\d_]*(?:\.\d[\d_]*)?(?:e-?\d+)?(?:[iuf]\d+|usize|isize)?|[A-Za-z_]\w*|"
    r"::|->|=>|==|!=|<=|>=|&&|\|\||\.\.=?|[-+*/%^&|<>]=|<<|>>|\S"
)
KEYWORDS = set(
    "as async await break const continue crate dyn else enum extern false fn for if impl in let loop match mod move "
    "mut pub ref return self Self static struct super trait true type unsafe use where while".split()
)
DECLARES = re.compile(r"^(pub (\( [^)]* \) )?)?(const N : [^=]* = [^;]* ;|static N : .*|N : [^=(]* ,?|N( \( [NL, ]*\))?( = L)? ,|N( :: N)*( \( [NL_, ]*\))? => [NL:( ),]*,|L => [NL:( ),]*,)$")
STARTS = re.compile(r"^(\s*)(?:pub(?:\([^)]*\))? )?(?:const )?(?:async )?fn (\w+)")


def sources(no_tests):
    for root in ROOTS:
        for path in sorted(glob.glob(f"{root}/**/*.rs", recursive=True)):
            if no_tests and path.endswith("/tests.rs"):
                continue
            lines = open(path, encoding="utf-8").read().split("\n")
            if no_tests:
                cut = next((at for at, line in enumerate(lines) if line.strip() == "#[cfg(test)]"), len(lines))
                lines = lines[:cut]
            yield path, lines


def significant(lines, loose=False):
    """The significant lines of a file: (its number from 1, the line without its whitespace); with `loose`,
    the line's tokens with names and literals taken for `N` and `L`."""
    kept = []
    using = False
    for at, line in enumerate(lines):
        bare = line.strip()
        if using:
            using = not bare.endswith(";")
            continue
        if not bare or COMMENT.match(line) or BRACES.match(line) or bare.startswith("#[") or bare.startswith("#!["):
            continue
        if re.match(r"(pub(\([^)]*\))? )?use ", bare):
            using = not bare.endswith(";")
            continue
        if re.match(r"(pub(\([^)]*\))? )?mod \w+;$", bare):
            continue
        if not NAME.search(LITERAL.sub("", bare)):
            continue
        if loose:
            # A declaration (a field, a constant, a variant, an arm that only names) is alike everywhere.
            made = " ".join(shape([bare], whole=True))
            if not DECLARES.match(made):
                kept.append((at + 1, made))
            continue
        kept.append((at + 1, re.sub(r"\s+", "", bare)))
    return kept


def blocks(least, no_tests, loose=False):
    """The groups of repeated blocks: (significant lines, [(file, first line, last line)])."""
    files = {path: significant(lines, loose) for path, lines in sources(no_tests)}
    places = {}
    for path, kept in files.items():
        for start in range(len(kept) - least + 1):
            key = hashlib.md5("\n".join(text for _, text in kept[start:start + least]).encode()).digest()
            places.setdefault(key, []).append((path, start))
    group_of = {}
    for key, found in places.items():
        if len(found) > 1:
            for place in found:
                group_of[place] = key
    groups = []
    for key, found in places.items():
        if len(found) < 2:
            continue
        # A run is begun only where the windows before it are not the same group one line earlier.
        before = {group_of.get((path, start - 1)) for path, start in found}
        if len(before) == 1 and None not in before and len(places[next(iter(before))]) == len(found):
            continue
        long = least
        while True:
            after = {group_of.get((path, start + long - least + 1)) for path, start in found}
            if len(after) != 1 or None in after or len(places[next(iter(after))]) != len(found):
                break
            long += 1
        # Places of one file that run into each other are a pattern repeated in a row, not two copies.
        apart = []
        for path, start in sorted(found):
            if not apart or apart[-1][0] != path or start >= apart[-1][1] + long:
                apart.append((path, start))
        if len(apart) > 1:
            groups.append((long, [(path, files[path][start][0], files[path][start + long - 1][0]) for path, start in apart]))
    return sorted(groups, key=lambda group: (-group[0] * (len(group[1]) - 1), group[1]))


def functions(lines):
    """Each function of a file with a body: its name, its first line (from 1) and its lines."""
    for at, line in enumerate(lines):
        found = STARTS.match(line)
        if not found or (line.rstrip().endswith(";") and "{" not in line):
            continue
        if line.count("{") and line.count("{") == line.count("}"):
            yield found.group(2), at + 1, [line]
            continue
        closing = found.group(1) + "}"
        end = next((end for end in range(at, len(lines)) if lines[end].rstrip() == closing), None)
        if end is not None:
            yield found.group(2), at + 1, lines[at:end + 1]


def shape(body, whole=False):
    """A function's tokens with every name taken for `N` and every literal for `L`; its own name left out
    (or, with `whole`, the tokens of any lines so)."""
    code = "\n".join(line for line in body if not COMMENT.match(line))
    code = code[code.index("("):] if "(" in code and not whole else code
    made = []
    for token in TOKEN.findall(code):
        if token[0].isdigit() or token[0] in "\"'" or re.match(r'b?r?#*"', token):
            made.append("L")
        elif NAME.match(token) and token not in KEYWORDS:
            made.append("N")
        else:
            made.append(token)
    return made


def shapes(least, no_tests):
    """The groups of functions of one shape: (tokens, [(file, line, name, lines)])."""
    found = {}
    for path, lines in sources(no_tests):
        for name, at, body in functions(lines):
            tokens = shape(body)
            if len(tokens) >= least:
                found.setdefault(" ".join(tokens), []).append((path, at, name, len(body), len(tokens)))
    groups = [(group[0][4], [place[:4] for place in group]) for group in found.values() if len(group) > 1]
    return sorted(groups, key=lambda group: (-group[0] * (len(group[1]) - 1), group[1]))


def main():
    asked = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    asked.add_argument("--lines", type=int, default=6, help="the least block, in significant lines (6)")
    asked.add_argument("--shape", action="store_true", help="functions of one shape instead of blocks")
    asked.add_argument("--tokens", type=int, default=60, help="the least function for --shape, in tokens (60)")
    asked.add_argument("--loose", action="store_true", help="blocks alike but for their names and literals")
    asked.add_argument("--no-tests", action="store_true", help="leave the tests out")
    asked.add_argument("--most", type=int, default=0, help="print only so many of the largest groups")
    asked.add_argument("--max", type=int, default=None, help="leave with 1 if the total is larger")
    given = asked.parse_args()
    if given.shape:
        groups = shapes(given.tokens, given.no_tests)
        total = sum(sum(place[3] for place in places[1:]) for _, places in groups)
        for tokens, places in groups[:given.most or None]:
            print(f"{len(places)} functions of one shape, {tokens} tokens:")
            for path, at, name, long in places:
                print(f"    {path}:{at} fn {name} ({long} lines)")
        print(f"total: {len(groups)} groups of functions of one shape ({given.tokens} tokens and more), {total} lines in their second and later copies")
    else:
        groups = blocks(given.lines, given.no_tests, given.loose)
        total = sum(long * (len(places) - 1) for long, places in groups)
        for long, places in groups[:given.most or None]:
            print(f"{long} lines in {len(places)} places:")
            for path, first, last in places:
                print(f"    {path}:{first}-{last}")
        kind = "blocks alike but for names and literals" if given.loose else "repeated blocks"
        print(f"total: {len(groups)} groups of {kind} ({given.lines} significant lines and more), {total} repeated lines")
    return 1 if given.max is not None and total > given.max else 0


if __name__ == "__main__":
    sys.exit(main())
