#!/usr/bin/env python3
"""Writes out what the client has compiled into it about the states a character is in (its buffs and debuffs).

    statuses.py <rodecomp folder> <output file>

Read from the decompiled client (`rodecomp`):
  Source/Contents/Effects/Core/EffectTypes.h               the states' numbers (EFST_…)
  Source/Contents/Actors/Resources/StatusIconImages.cpp    CSession::GetEFSTImgFileName: the picture of a state the
                                                           client's Lua files (StateIconImgInfo) have none for
  Source/Client/Mode/GameModeNotices.cpp                   BuildStatusFlagEffects, BuildStatusLevelEffects: the
                                                           states that the body's and the health's state stand for

Written, one entry a line, in the client's order:
  icon<TAB><kind><TAB><state><TAB><file>      the picture (in data\\texture\\effect) of a state, and which of the
                                              five kinds the client lists it under (it shows them kind by kind)
  taekwon<TAB><kind><TAB><state><TAB><file>   the same for the jobs of the Taekwon line (IsTaekwonDisplayJob)
  body<TAB><level><TAB><state>                the state shown for a body state (stone, frozen, stunned…)
  health<TAB><bit><TAB><state>                the state shown for a bit of the health state (poison, curse…);
                                              those the client leaves out (ApplyStatusIcons) are not written
"""
import os
import re
import sys

root, out = sys.argv[1], sys.argv[2]


def read(path):
    return open(os.path.join(root, path), encoding='utf-8', errors='replace').read()


states, last = {}, -2
enum = read('Source/Contents/Effects/Core/EffectTypes.h')
enum = enum[enum.index('enum efst_type'):]
for name, value in re.findall(r'^\s*(EFST_[A-Za-z0-9_]+)\s*(?:=\s*(-?0x[0-9A-Fa-f]+|-?\d+))?\s*,', enum[:enum.index('};')], re.M):
    last = int(value, 0) if value else last + 1
    states[name] = last

rows, kind, cases, taekwon = [], None, [], False
text = read('Source/Contents/Actors/Resources/StatusIconImages.cpp')
text = text[text.index('switch (type)'):]
for line in text.split('\n'):
    of_kind = re.match(r'\s{4}case (\d):', line)
    case = re.match(r'\s*case (EFST_[A-Za-z0-9_]+):', line)
    back = re.match(r'\s*return "effect\\\\(.*)";', line)
    if of_kind:
        kind, cases = int(of_kind.group(1)), []
    elif case:
        # A new run of cases starts after a picture was given.
        if cases and cases[-1] is None:
            cases = []
        cases.append(case.group(1))
    elif 'IsTaekwonDisplayJob()' in line:
        taekwon = True
    elif back and kind is not None:
        named = [name for name in cases if name]
        rows += [('taekwon' if taekwon else 'icon', kind, states[name], back.group(1)) for name in named if name in states]
        if taekwon:
            taekwon = False
        else:
            cases.append(None)

notices = read('Source/Client/Mode/GameModeNotices.cpp')
left_out = set(re.findall(r'case (EFST_HEALTHSTATE_\w+):', notices[notices.index('void CGameMode::ApplyStatusIcons('):][:1200]))
for word, function in (('health', 'BuildStatusFlagEffects'), ('body', 'BuildStatusLevelEffects')):
    body = notices[notices.index(f'static std::map<int, int> {function}()'):]
    for key, name in re.findall(r'effects\[(0x[0-9A-Fa-f]+|\d+)\] = (EFST_\w+);', body[:body.index('return effects;')]):
        if name not in left_out:
            rows.append((word, int(key, 0).bit_length() - 1 if word == 'health' else int(key, 0), states[name]))

open(out, 'w', encoding='utf-8').write(
    '# What the original client (Ragexe 2026-02-19) has compiled into it about the states a character is in: the\n'
    '# pictures of those its Lua files have none for (`CSession::GetEFSTImgFileName`), and the states its body and\n'
    '# health states stand for (`BuildStatusLevelEffects`, `BuildStatusFlagEffects`). Written out by\n'
    '# tools/client-tables/statuses.py; the lines are explained there.\n'
    + ''.join('\t'.join(str(field) for field in row) + '\n' for row in rows))
print(f'{len(rows)} lines')
