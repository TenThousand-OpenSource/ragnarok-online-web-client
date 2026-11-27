#!/usr/bin/env python3
"""Writes the table of the NPCs of the classic server that carry a player to another map, for the navigation.

    navigation_links.py <rathena folder> <server folder> <the client's Navi_Link as JSON> <output table>

    (the client's table: `roweb-gateway dump 'navigation\\navi_link_krpri' Navi_Link > links.json`)

rAthena's generator of the navigation's tables (`map-server-generator --generate-navi`, src/map/navi.cpp) knows
the portals only. An NPC that takes one elsewhere when talked to (a Kafra's teleport, a ship's sailor, a guard)
is a way out only where its script says so: a label `OnNaviGenerate:` with `naviregisterwarp "<name>","<map>",
<x>,<y>;`. None of rAthena's scripts has one. `server/navigation.sh` writes those labels into the generator's own
copy of the scripts, from the table this writes; the running server's scripts are not touched.

Who carries, and where to, is read from the scripts the classic server loads, never typed here:

  - a Kafra with the teleport service: where to and for how much is `F_KafSet`'s list for her map, the cell is
    `F_KafTele`'s (npc/kafras/functions_kafras.txt); kind 204, as the client's table has a Kafra;
  - anyone else who stands on a map from which the client's own table has a way of that kind (201 an NPC that
    asks nothing, 204 one that asks zeny, 205 the airship) to a map, and whose script, or a function it calls,
    has a `warp` to that same map: the cell is the script's. The client's table is the judge of who is a carrier
    (a script sends players elsewhere for many reasons: a quest, a test, a prison); the server's script is the
    judge of where one arrives. Of several on the map the one nearest the client's is taken;
  - `BY_HAND` below: carriers of the classic world the client's table has no row for, each with the script read.

The cells are those the server uses: `roweb_classic.txt`'s moves and `roweb_classic_warps.sed` are applied.
What a script asks before it carries (zeny, an item, a quest) is not looked at: the table says a way exists.
"""
import collections
import json
import re
import sys

root, server, client_links, out = sys.argv[1].rstrip('/'), sys.argv[2].rstrip('/'), sys.argv[3], sys.argv[4]

# (map, the NPC's unique name, kind, the map it carries to): read in the scripts named, not in the client's table.
BY_HAND = [
    # npc/quests/quests_morocc.txt, `MocConGuard`: the guards of the gorge around Morroc let one through to the
    # town (`warp "morocc",160,61`) in every state of the quest; the classic fields all lead into the gorge.
    *[('moc_fild20', name, 201, 'morocc') for name in ['MocConGuard'] + [f'Continental Guard#0{at}' for at in range(2, 9)]],
]
# How far from the client's carrier the server's may stand and still be the same one (the classic scripts
# place some by the old maps).
NEAR = 60


def listed(conf, base=None):
    """The scripts a conf names, in order, and those it takes away (`delnpc`)."""
    files = []
    try:
        lines = open(f'{base or root}/{conf}', encoding='latin1').read().split('\n')
    except OSError:
        return files
    for line in lines:
        line = line.split('//')[0].strip()
        for word, sign in (('npc:', 1), ('delnpc:', -1)):
            if line.startswith(word):
                path = line[len(word):].strip()
                files += listed(path) if path.endswith('.conf') else [(sign, path)]
        if line.startswith('import:'):
            files += listed(line[7:].strip())
    return files


def loaded():
    """The scripts the classic server loads: rAthena's for the classic rules, and the server's own conf over them."""
    order = listed('npc/pre-re/scripts_main.conf') + listed('conf/map_conf.txt', server)
    files = []
    for sign, path in order:
        if sign > 0 and path not in files:
            files.append(path)
        elif sign < 0 and path in files:
            files.remove(path)
    return files


CELLS = [re.match(r's/(.*)/(.*)/g$', line).groups()
         for line in open(f'{server}/npc/roweb_classic_warps.sed', encoding='utf-8') if line.startswith('s/')]
PLACED = re.compile(r'^([A-Za-z0-9_@\-]+),(\d+),(\d+),(\d+)\t(script|duplicate\(([^)]*)\))\t([^\t]*)\t(-?\w+)')
FLOATING = re.compile(r'^(?:-\t(script)|function\t(script))\t([^\t]*)\t')
WARP = re.compile(r'\bwarp\s*\(?\s*"(\w+)"\s*,\s*(\d+)\s*,\s*(\d+)')
CALLED = re.compile(r'callfunc\s*\(?\s*"(\w+)"')


def unique(name):
    return name.split('::')[-1]


def shown(name):
    return name.split('::')[0].split('#')[0]


class Scripts:
    """What the loaded scripts hold: who stands where, and the body of every script by its unique name."""

    def __init__(self):
        self.placed, self.bodies, self.homes = [], {}, {}
        for path in loaded():
            where = server if path.startswith('npc/custom/roweb_') else root
            try:
                text = open(f"{where}/{path.replace('npc/custom/', 'npc/') if where == server else path}", encoding='latin1').read()
            except OSError:
                continue
            for old, new in CELLS:
                text = re.sub(old, new, text)
            self.read(path, text.replace('\r', '').split('\n'))

    def read(self, path, lines):
        at = 0
        while at < len(lines):
            placed, floating = PLACED.match(lines[at]), FLOATING.match(lines[at])
            name = unique(placed.group(7)) if placed else unique(floating.group(3)) if floating else None
            if placed:
                where, x, y, _, kind, source, _, sprite = placed.groups()
                self.placed.append({'map': where, 'x': int(x), 'y': int(y), 'name': name, 'shown': shown(placed.group(7)),
                                    'sprite': sprite, 'body': source if source is not None else name, 'file': path})
            if name is not None and not (placed and placed.group(6) is not None):
                end = at
                # A script's body ends with a brace alone at the start of a line; one written on one line is
                # passed by.
                if not lines[at].rstrip().endswith('}'):
                    end = next((line for line in range(at + 1, len(lines)) if lines[line].rstrip() == '}'), at)
                self.bodies[name] = '\n'.join(lines[at:end + 1])
                self.homes[name] = (path, lines[at], end > at)
                at = end
            at += 1

    def moved(self):
        """`roweb_classic.txt`'s moves and removals, as the server makes them when it starts."""
        text = open(f'{server}/npc/roweb_classic.txt', encoding='utf-8').read().split('\tend;\n\t}\n', 1)[-1]
        gone = set(re.findall(r'unloadnpc "([^"]+)";', text))
        moves = {name: (int(x), int(y)) for name, x, y in re.findall(r'movenpc "([^"]+)",(\d+),(\d+),', text)}
        self.placed = [npc for npc in self.placed if npc['name'] not in gone]
        for npc in self.placed:
            npc['x'], npc['y'] = moves.get(npc['name'], (npc['x'], npc['y']))

    def text(self, npc):
        """The script of an NPC with the functions it calls, and those they call."""
        seen, text, names = set(), '', [npc['body']]
        while names:
            name = names.pop()
            if name in seen or name not in self.bodies:
                continue
            seen.add(name)
            text += self.bodies[name] + '\n'
            names += CALLED.findall(self.bodies[name])
        return text


def cell(text, to):
    """The cell a script sends one to on a map: of a pair by the rules (`if (checkre(0)) … else …`), the
    classic one, which is the second."""
    cells = [(int(x), int(y), found.start()) for found in WARP.finditer(text) for where, x, y in [found.groups()] if where == to]
    if not cells:
        return None
    for (x, y, start), later in zip(cells, cells[1:]):
        if 'checkre(0)' in text[max(0, start - 80):start] and 'else' in text[start:later[2]]:
            return later[:2]
    return cells[0][:2]


def kafras(scripts):
    """The Kafras' teleports: `F_KafSet`'s list for the map, at `F_KafTele`'s cells."""
    lists, tele = scripts.bodies['F_KafSet'], scripts.bodies['F_KafTele']
    cells = {name: (where, int(x), int(y)) for name, where, x, y in
             re.findall(r'@wrpD\$\[\.@j\] == "([^"]+)"\)\s*(?:\{[^}]*?else\s*)?warp "(\w+)",\s*(\d+),\s*(\d+)', tele)}
    rows = []
    for where, block in re.findall(r'\.@map\$ == "(\w+)"\s*\) \{(.*?)\n\t\}', lists, re.S):
        names = re.findall(r'"([^"]+)"', re.search(r'setarray @wrpD\$\[0\],([^;]*);', block).group(1))
        prices = re.findall(r'\d+', re.search(r'setarray @wrpP\[0\],([^;]*);', block).group(1))
        for npc in scripts.placed:
            call = re.search(r'callfunc\s*\(?\s*"F_Kafra"\s*,\s*\d+\s*,\s*(\d+)', scripts.bodies.get(npc['body'], ''))
            # (The second word of the call switches the teleport off.)
            if npc['map'] != where or not call or call.group(1) == '1':
                continue
            rows += [(npc, 204, *cells[name], int(price), 'npc/kafras/functions_kafras.txt') for name, price in zip(names, prices)]
    return rows


def carriers(scripts, taken, kafras):
    """Whoever the client's table has a way of for a map, and the server's script confirms. (Not a way for zeny
    between two maps a Kafra of the server joins: in the client's table that row is the Kafra's.)"""
    wanted = collections.defaultdict(list)
    for row in json.load(open(client_links, encoding='utf-8')):
        if row[2] in (201, 204, 205) and row[0] != row[8] and not (row[2] == 204 and (row[0], row[8]) in kafras):
            wanted[(row[0], row[8], row[2])].append((row[6], row[7], row[4]))
    rows = []
    for (where, to, kind), theirs in sorted(wanted.items()):
        best = None
        for npc in scripts.placed:
            if npc['map'] != where or npc['sprite'] in ('-1', '111', '139', 'HIDDEN_NPC', 'HIDDEN_WARP_NPC') or (npc['name'], to) in taken:
                continue
            arrives = cell(scripts.text(npc), to)
            if arrives is None:
                continue
            far = min(abs(npc['x'] - x) + abs(npc['y'] - y) for x, y, _ in theirs)
            if far <= NEAR and (best is None or far < best[0]):
                best = (far, npc, arrives)
        if best:
            rows.append((best[1], kind, to, *best[2], 0, best[1]['file']))
    return rows


def by_hand(scripts):
    rows = []
    for where, name, kind, to in BY_HAND:
        for npc in scripts.placed:
            arrives = cell(scripts.text(npc), to) if (npc['map'], npc['name']) == (where, name) else None
            if arrives:
                rows.append((npc, kind, to, *arrives, 0, npc['file']))
    return rows


scripts = Scripts()
scripts.moved()
rows = kafras(scripts)
joined = {(row[0]['map'], row[2]) for row in rows}
rows += by_hand(scripts)
rows += carriers(scripts, {(row[0]['name'], row[2]) for row in rows}, joined)
lines = [
    '# The NPCs of the classic server that carry a player to another map, for the navigation.',
    '# Written by tools/client-tables/navigation_links.py: see it for what this is and where each row is from.',
    '# Not to be edited by hand. server/navigation.sh reads it.',
    '#',
    '# kind (201 an NPC that asks nothing, 204 one that asks zeny, 205 the airship), the map and the cell the NPC',
    '# stands on, its unique name, the name shown, the map and the cell it carries to, the zeny a Kafra asks (the',
    "# client's table has no price; kept to be read), the script whose body holds the NPC's, that script's first",
    '# line, the file.',
]
skipped = 0
for npc, kind, to, x, y, price, source in sorted(rows, key=lambda row: (row[0]['map'], row[0]['name'], row[2])):
    path, first, whole = scripts.homes.get(npc['body'], (None, None, False))
    if not whole or to == npc['map']:
        skipped += 1
        continue
    lines.append('\t'.join(str(word) for word in (kind, npc['map'], npc['x'], npc['y'], npc['name'], npc['shown'], to, x, y, price,
                                                  npc['body'], path)))
open(out, 'w', encoding='utf-8').write('\n'.join(lines) + '\n')
print(f'{len(lines) - 8} carriers ({sum(1 for line in lines if line.startswith("204"))} for zeny), {skipped} passed by')
