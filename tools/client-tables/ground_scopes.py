#!/usr/bin/env python3
"""Writes out how large the circle on the ground is while a skill is cast on a place.

    ground_scopes.py <rodecomp folder> <output file>   (client/src/effect_tables/ground_scopes.txt)

Read from the decompiled client (`rodecomp`):
  Source/Contents/Skills/SkillId.h        the skills' numbers
  Source/Contents/Actors/GroundScope.cpp  CGroundScope::SkillScopeType: the scope of each skill it names

Written, one entry a line: <skill><TAB><scope>. The circle is the scope and two more cells across
(`CGroundScope::SendMsg`, AM_SET_SKILL). A skill without a line is asked of its own action in the client
(`CSkillActionMgr::GetSkillScopeType`), which this table does not have.
"""
import os
import re
import sys

root, out = sys.argv[1], sys.argv[2]


def read(path):
    return open(os.path.join(root, path), encoding='utf-8', errors='replace').read()


skills, last = {}, -1
for name, value in re.findall(r'^\s*([A-Z0-9]+_[A-Z0-9_]+)\s*(?:=\s*(0x[0-9A-Fa-f]+|-?\d+))?\s*,', read('Source/Contents/Skills/SkillId.h'), re.M):
    last = int(value, 0) if value else last + 1
    skills[name] = last
text = read('Source/Contents/Actors/GroundScope.cpp')
text = text[text.index('int CGroundScope::SkillScopeType('):]
text = text[:text.index('default:')]
rows, labels = [], []
for line in text.split('\n'):
    label = re.match(r'\s*case ([A-Z0-9_]+):', line)
    scope = re.match(r'\s*return (-?\d+);', line)
    if label:
        labels.append(label.group(1))
    elif scope:
        rows += [(skills[name], int(scope.group(1))) for name in labels if name in skills]
        labels = []
with open(out, 'w', encoding='utf-8') as file:
    for skill, scope in sorted(rows):
        file.write(f'{skill}\t{scope}\n')
print(f'{len(rows)} skills')
