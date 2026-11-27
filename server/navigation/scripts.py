#!/usr/bin/env python3
"""What `../navigation.sh` does to the generator's own copy of the scripts, and to what the generator writes.

    scripts.py before <work folder>     writes there what changes the copy of the scripts: `moved.sed` (for all
                                        of them) and, for each script that holds a carrier, `<n>.sed`, with
                                        `list` naming the script each is for
    scripts.py after <navi_link file>   gives each carrier's way out its kind

Moved: whoever `../npc/roweb_classic.txt` moves as the server starts (`movenpc`) is written where they are
moved to. The generator lists everyone by the cell their script names, not by where they stand afterwards.

Carriers: rAthena's generator takes an NPC for a way out only where its script says so, under a label
`OnNaviGenerate:` (`naviregisterwarp`, src/map/script.cpp; npc/custom/warper.txt has one). `links.txt` (written
by tools/client-tables/navigation_links.py) names who carries where; each gets that label at the end of the
script whose body is theirs. Several NPCs share a body (`duplicate`), so the label asks who it runs for.

Kinds: the generator writes every such way as 201. The client's table has 204 for one that asks zeny and 205
for the airship, and the navigation takes those only when told to ("service"). The kind is carried through the
generator in front of the way's name ("204:Kafra Employee") and put in its place afterwards.
"""
import collections
import os
import re
import sys

here = os.path.dirname(os.path.abspath(__file__))


def plain(text):
    """A text as a `sed` pattern matches it letter for letter."""
    return re.sub(r'([.*\[\]\\/^$&])', r'\\\1', text)


def before(work):
    with open(f'{work}/moved.sed', 'w', encoding='latin1') as moved:
        for line in open(f'{here}/../npc/roweb_classic.txt', encoding='latin1'):
            move = re.match(r'\s*movenpc "([^"]+)",(\d+),(\d+),(\d+);\s*// (\w+)', line)
            if move:
                name, x, y, facing, where = move.groups()
                moved.write(f's/^{where},[0-9]*,[0-9]*,[0-9]*\\(\t[^\t]*\t{plain(name)}\\(::[^\t]*\\)\\{{0,1\\}}\t\\)/{where},{x},{y},{facing}\\1/\n')
    bodies = collections.defaultdict(lambda: collections.defaultdict(list))
    for line in open(f'{here}/links.txt', encoding='utf-8'):
        if not line.startswith('#') and line.strip():
            kind, _, _, _, name, shown, to, x, y, _, body, path = line.rstrip('\n').split('\t')
            bodies[path][body].append((name, f'naviregisterwarp "{kind}:{shown}","{to}",{x},{y};'))
    with open(f'{work}/list', 'w', encoding='utf-8') as listed:
        for number, (path, scripts) in enumerate(sorted(bodies.items())):
            listed.write(f'{number}\t{path}\n')
            with open(f'{work}/{number}.sed', 'w', encoding='latin1') as rules:
                for body, marks in sorted(scripts.items()):
                    by_name = collections.defaultdict(list)
                    for name, mark in marks:
                        by_name[name].append(mark)
                    asks = ''.join(f'\\n\\tif (strnpcinfo(3) == "{plain(name)}") {{ {plain(" ".join(said))} }}' for name, said in sorted(by_name.items()))
                    # From the script's first line to the brace that ends it, alone at the start of a line.
                    rules.write(f'/^[^\t]*\tscript\t\\([^\t]*::\\)\\{{0,1\\}}{plain(body)}\t/,/^}}/ '
                                f's/^}}/\\tend;\\nOnNaviGenerate:{asks}\\n\\tend;\\n}}/\n')


def after(links):
    text = open(links, encoding='latin1').read()
    text, changed = re.subn(r'(?m)^(\t\{"[^"]*", \d+, )201(, -?\d+, ")(20[145]):', r'\1\3\2', text)
    open(links, 'w', encoding='latin1').write(text)
    kinds = collections.Counter(re.findall(r'(?m)^\t\{"[^"]*", \d+, (\d+),', text))
    print(f'ways out by kind: {dict(sorted(kinds.items()))} ({changed} of NPCs that carry)')


{'before': before, 'after': after}[sys.argv[1]](sys.argv[2])
