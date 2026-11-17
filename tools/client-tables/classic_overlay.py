#!/usr/bin/env python3
"""Writes the script that puts rAthena's classic (pre-renewal) NPCs where the new client's maps have room for them.

    classic_overlay.py <rathena folder> <output file>

rAthena's classic scripts place everyone by the old maps of a few places. The 2026 client has the new maps of
them (they are the maps rAthena's renewal scripts are written for), so there the classic coordinates are wrong.
For each such map this compares the two sets of scripts, name by name:

  - someone who stands on the map in both is moved to where the renewal script has them (`movenpc`);
  - a portal on the map, or leading to it, that differs is taken away and put again as the renewal script has it;
  - a portal on the map that the renewal scripts do not have at all is taken away;
  - whoever the renewal scripts do not have there is left where the classic script put them, and listed in a
    comment at the end, to be looked at by hand.

What the scripts do when talked to is not touched: only where things stand. A script that sends a player to such
a map by itself (`warp "izlude",x,y`) still names the old place.
"""
import re
import sys

root, out = sys.argv[1].rstrip('/'), sys.argv[2]
# The maps whose classic form differs from the client's: by walkable cells, against rAthena's
# db/pre-re/map_cache.dat (the other five maps listed there are the same in the client to within 1%).
MAPS = ['izlude', 'alberta', 'morocc']


def listed(conf):
    files = []
    for line in open(f'{root}/{conf}', encoding='latin1'):
        line = line.split('//')[0].strip()
        if line.startswith('npc:'):
            path = line[4:].strip()
            files += listed(path) if path.endswith('.conf') else [path]
        elif line.startswith('import:'):
            files += listed(line[7:].strip())
    return files


PLACED = re.compile(r'^([A-Za-z0-9_@\-]+),(\d+),(\d+),(\d+)\t([a-z]+(?:\([^)]*\))?)\t([^\t]*)\t(.*)$')


def read(conf):
    """What a set of scripts places: [(file, map, x, y, facing, kind, name, the rest of the line)]."""
    found = []
    for path in listed(conf):
        try:
            text = open(f'{root}/{path}', encoding='latin1').read()
        except OSError:
            continue
        for line in text.split('\n'):
            placed = PLACED.match(line.rstrip('\r'))
            if placed:
                at, x, y, facing, kind, name, rest = placed.groups()
                found.append((path, at, int(x), int(y), int(facing), kind, name, rest))
    return found


def unique(name):
    return name.split('::')[-1]


classic, renewal = read('npc/pre-re/scripts_main.conf'), read('npc/re/scripts_main.conf')
lines = [
    '//===== roweb =================================================',
    '//= The classic (pre-renewal) NPCs on the new client\'s maps',
    '//= Written by tools/client-tables/classic_overlay.py: see it for',
    '//= what this does and why. Not to be edited by hand.',
    '//============================================================',
    '',
]
moves, gone, again, left, dropped = [], [], [], [], []
for at in MAPS:
    there = {unique(entry[6]): entry for entry in renewal if entry[1] == at and entry[5] != 'warp'}
    for path, where, x, y, facing, kind, name, rest in classic:
        if where != at or kind == 'warp':
            continue
        new = there.get(unique(name))
        if new is None:
            left.append((at, unique(name), x, y, path))
        elif (new[2], new[3]) != (x, y):
            moves.append((at, unique(name), new[2], new[3], new[4]))
# Portals: on such a map, or leading to one.
portals = {entry[6]: entry for entry in renewal if entry[5] == 'warp'}
for path, where, x, y, facing, kind, name, rest in classic:
    if kind != 'warp':
        continue
    to = rest.split(',')[2] if rest.count(',') >= 4 else ''
    if where not in MAPS and to not in MAPS:
        continue
    new = portals.get(name)
    if new is None and where in MAPS:
        # A portal of the old map that the new one has no place for (the building it led into is entered
        # another way now, or not at all): it would open in the middle of a square, so it goes.
        gone.append(name)
        dropped.append((where, name, x, y, path))
    elif new is None:
        left.append((where, name + ' (portal)', x, y, path))
    elif (new[1], new[2], new[3], new[7]) != (where, x, y, rest):
        gone.append(name)
        again.append(f'{new[1]},{new[2]},{new[3]},{new[4]}\twarp\t{name}_n\t{new[7]}')

lines += ['-\tscript\troweb_classic_maps\t-1,{', 'OnInit:']
lines += [f'\tunloadnpc "{name}";' for name in gone]
lines += [f'\tmovenpc "{name}",{x},{y},{facing};\t// {at}' for at, name, x, y, facing in moves]
lines += ['\tend;', '}', ''] + again + ['']
lines += ['// Portals of the old maps taken away, the new maps having no place for them.']
lines += [f'// {at}: {name} at {x},{y} ({path})' for at, name, x, y, path in dropped]
lines += ['// Left where the classic script put them: the renewal scripts have nobody of that name on the map.']
lines += [f'// {at}: {name} at {x},{y} ({path})' for at, name, x, y, path in left]
open(out, 'w', encoding='utf-8').write('\n'.join(lines) + '\n')
print(f'{len(moves)} moved, {len(again)} portals put again, {len(dropped)} portals taken away, {len(left)} left as they were')
