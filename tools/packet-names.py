#!/usr/bin/env python3
"""Writes out the names rAthena's headers give the packets, by number, for one PACKETVER.

    packet-names.py <rathena folder> <output file> [packetver]

Read from rAthena, each file through its own `#if PACKETVER…` so that only what holds at this PACKETVER counts:
  src/map/packets.hpp, src/map/packets_struct.hpp   `DEFINE_PACKET_HEADER(ZC_…, 0x…)` and `DEFINE_PACKET_ID`
  src/map/clif_packetdb.hpp                         the name in the comment after `packet(0x…, n); // ZC_…`,
                                                    and the function that reads a packet of the client's
                                                    (`parseable_packet(0x…, n, clif_parse_…)`)
  src/map/packets_struct.hpp                        the names of `enum packet_headers` (`…Type = 0x…`)
  src/common/packets.hpp                            the login and character servers' headers

A number named more than once keeps the first of these, in that order. Written one a line, `0x0984 NAME`, as
client/src/net/packet_lengths.txt is; read by client/src/net/unread.rs for the developer's list of the
packets this client does not read (`/packets`).
"""
import os
import re
import sys

root, out = sys.argv[1], sys.argv[2]
version = int(sys.argv[3]) if len(sys.argv) > 3 else 20260219
# What rAthena's src/config/packets.hpp defines for a client of the main kRO line at this PACKETVER.
DEFINED = {'PACKETVER': version, 'PACKETVER_MAIN_NUM': version}


def holds(condition):
    """Whether a condition of `#if` holds: names not defined count as nought, as the preprocessor has it."""
    condition = re.sub(r'//.*|/\*.*?\*/', '', condition)
    condition = re.sub(r'defined\s*\(\s*(\w+)\s*\)|defined\s+(\w+)', lambda found: str(int((found.group(1) or found.group(2)) in DEFINED)), condition)
    condition = re.sub(r'\b[A-Za-z_]\w*\b', lambda found: str(DEFINED.get(found.group(0), 0)), condition)
    condition = condition.replace('&&', ' and ').replace('||', ' or ')
    condition = re.sub(r'!(?!=)', ' not ', condition)
    try:
        return bool(eval(condition, {'__builtins__': {}}))
    except Exception:
        return False


def live(path):
    """The lines of a header that this PACKETVER compiles."""
    # A level of `#if` is [whether its lines count, whether a branch of it was taken, whether the level
    # around it counts].
    levels, kept = [], []
    for line in open(os.path.join(root, path), encoding='utf-8', errors='replace'):
        found = re.match(r'\s*#\s*(ifdef|ifndef|if|elif|else|endif)\b(.*)', line)
        if not found:
            if all(level[0] for level in levels):
                kept.append(line)
            continue
        word, rest = found.group(1), found.group(2).strip()
        around = all(level[0] for level in levels[:-1]) if word in ('elif', 'else') else all(level[0] for level in levels)
        if word in ('if', 'ifdef', 'ifndef'):
            name = rest.split()[0] if rest.split() else ''
            yes = holds(rest) if word == 'if' else (name in DEFINED) == (word == 'ifdef')
            levels.append([around and yes, yes, around])
        elif word == 'elif' and levels:
            yes = not levels[-1][1] and holds(rest)
            levels[-1][0], levels[-1][1] = around and yes, levels[-1][1] or yes
        elif word == 'else' and levels:
            levels[-1][0], levels[-1][1] = around and not levels[-1][1], True
        elif word == 'endif' and levels:
            levels.pop()
    return ''.join(kept)


names = {}


def name(number, called):
    names.setdefault(int(number, 16), called)


HEADER = r'DEFINE_PACKET_(?:HEADER|ID)\s*\(\s*(\w+)\s*,\s*0x([0-9A-Fa-f]+)\s*\)'
structs = live('src/map/packets_struct.hpp')
for path in ('src/map/packets.hpp',):
    for called, number in re.findall(HEADER, live(path)):
        name(number, called)
for called, number in re.findall(HEADER, structs):
    name(number, called)
database = live('src/map/clif_packetdb.hpp')
for number, called in re.findall(r'packet\(\s*0x([0-9A-Fa-f]+)\s*,[^;]*;\s*//\s*((?:ZC|CZ|AC|CA|HC|CH|SC|CS)_\w+)', database):
    name(number, called)
for called, number in re.findall(r'^\s*(\w+)Type\s*=\s*0x([0-9A-Fa-f]+)', structs, re.M):
    name(number, called)
for number, called in re.findall(r'parseable_packet\(\s*0x([0-9A-Fa-f]+)\s*,[^,]*,\s*(clif_parse_\w+)', database):
    name(number, called)
for called, number in re.findall(HEADER, live('src/common/packets.hpp')):
    name(number, called)

with open(out, 'w', encoding='utf-8') as file:
    file.write(f'# The names rAthena gives the packets at PACKETVER {version}, by number.\n')
    file.write('# Written by tools/packet-names.py from rAthena\'s headers; the lines are explained there.\n')
    for number in sorted(names):
        file.write(f'0x{number:04X} {names[number]}\n')
print(f'{len(names)} names -> {out}')
