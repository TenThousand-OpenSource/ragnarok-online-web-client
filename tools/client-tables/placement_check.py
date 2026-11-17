#!/usr/bin/env python3
"""Checks that everyone the server's scripts place stands where the client's map lets one stand.

    placement_check.py <rathena folder> <scripts conf> <folder of .gat files> [gateway address]

Reads every script the conf lists (as `classic_overlay.py` does), and for each NPC, shop and portal looks at the
cell of the client's map (`data/<map>.gat`) it is put on. A map's file missing from the folder is fetched from the
gateway, when its address is given. Printed, by map:

  - someone to be seen who stands on a cell nobody can walk on;
  - a portal none of whose cells can be walked on, or that leads to such a cell.

Standing in a wall is not always a mistake (a sign on a wall, a sailor on a ship's rail), so the list is to be
looked at, not obeyed. Somebody in the middle of a wall of a town is the sign of a map that differs from the one
the scripts were written for.
"""
import os
import re
import struct
import sys
import urllib.request

root, conf, folder = sys.argv[1].rstrip('/'), sys.argv[2], sys.argv[3]
gateway = sys.argv[4].rstrip('/') if len(sys.argv) > 4 else None
PLACED = re.compile(r'^([A-Za-z0-9_@\-]+),(\d+),(\d+),(\d+)\t([a-z]+(?:\([^)]*\))?)\t([^\t]*)\t(.*)$')
UNSEEN = {'-1', '111', '139', 'HIDDEN_NPC', 'HIDDEN_WARP_NPC', 'FAKE_NPC', 'INVISIBLE', '844', 'CLEAR_NPC'}


def listed(conf):
    files = []
    try:
        text = open(f'{root}/{conf}', encoding='latin1')
    except OSError:
        return files
    for line in text:
        line = line.split('//')[0].strip()
        if line.startswith('npc:'):
            path = line[4:].strip()
            files += listed(path) if path.endswith('.conf') else [path]
        elif line.startswith('import:'):
            files += listed(line[7:].strip())
    return files


maps = {}


def cells(name):
    """The map's walkable cells: (width, height, bytes of 1 and 0), or None when the client has no such map."""
    if name not in maps:
        path = f'{folder}/{name}.gat'
        if not os.path.exists(path) and gateway:
            try:
                open(path, 'wb').write(urllib.request.urlopen(f'{gateway}/assets/data/{name}.gat').read())
            except OSError:
                pass
        try:
            data = open(path, 'rb').read()
        except OSError:
            data = b''
        if data[:4] != b'GRAT':
            maps[name] = None
        else:
            width, height = struct.unpack_from('<II', data, 6)
            # The kinds of cell one can stand on: 0 ground, 3 water, and 2, 4, 6, which the server takes for
            # ground too (rAthena's `map_gat2cell`).
            maps[name] = (width, height, bytes(data[14 + 20 * i + 16] in (0, 2, 3, 4, 6) for i in range(width * height)))
    return maps[name]


def walkable(name, x, y):
    found = cells(name)
    if found is None:
        return None
    width, height, walk = found
    return 0 <= x < width and 0 <= y < height and walk[y * width + x] == 1


wrong, missing = {}, set()
for path in listed(conf):
    try:
        text = open(f'{root}/{path}', encoding='latin1').read()
    except OSError:
        continue
    for line in text.split('\n'):
        placed = PLACED.match(line.rstrip('\r'))
        if not placed:
            continue
        at, x, y, facing, kind, name, rest = placed.groups()
        x, y = int(x), int(y)
        if cells(at) is None:
            missing.add(at)
            continue
        if kind in ('warp', 'warp2'):
            parts = rest.split(',')
            try:
                wide, high, to, tx, ty = int(parts[0]), int(parts[1]), parts[2], int(parts[3]), int(re.match(r'\d+', parts[4]).group())
            except (ValueError, IndexError, AttributeError):
                continue
            if not any(walkable(at, x + dx, y + dy) for dx in range(-wide, wide + 1) for dy in range(-high, high + 1)):
                wrong.setdefault(at, []).append(f'portal {name} at {x},{y}: no cell of it can be walked on ({path})')
            if walkable(to, tx, ty) is False:
                wrong.setdefault(to, []).append(f'portal {name} of {at} leads to {tx},{ty}, where nobody can stand ({path})')
        elif kind.split('(')[0] in ('script', 'shop', 'cashshop', 'itemshop', 'pointshop', 'marketshop', 'duplicate', 'trader'):
            sprite = rest.split(',')[0].strip()
            if sprite in UNSEEN or (x, y) == (0, 0):
                continue
            if not walkable(at, x, y):
                wrong.setdefault(at, []).append(f'{name} at {x},{y} ({path})')
for at in sorted(wrong):
    print(f'{at}: {len(wrong[at])}')
    for line in wrong[at]:
        print(f'  {line}')
print(f'{sum(map(len, wrong.values()))} to look at on {len(wrong)} maps; {len(missing)} maps the client has no file for: {" ".join(sorted(missing))}')
