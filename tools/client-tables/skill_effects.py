#!/usr/bin/env python3
"""Writes out the tables of skills and effects the original client has compiled into it.

    skill_effects.py <rodecomp folder> <output folder>

Read from the decompiled client (`rodecomp`):
  Source/Contents/Skills/SkillId.h                    the skills' numbers
  Source/Contents/Effects/Core/EffectTypes.h          the effects' numbers (EF_…)
  Source/Contents/Effects/Core/RagEffectInitialization.cpp   which .str file an effect plays
  Source/Contents/Effects/Core/RagEffect.cpp          which sound an effect starts with
  Source/Client/Mode/GameModePackets.cpp              CGameMode::ReceiveSkillUse: the effects of a skill that
                                                      strikes nobody, on the one who used it or on its target
  Source/Contents/Skills/SkillActionInfo.cpp          CSession::GetSkillActionInfo2: the motion of the body
  Source/Contents/Jobs/JobInheritance.cpp             CSession::PlaySkillEffect: the effects of a skill that
                                                      strikes somebody, and whose place each is put at

Written, one entry a line, tab between the fields:
  effect_files.txt     <effect>  <file.str>       (merged with what the folder already has)
  effect_sounds.txt    <effect>  <sound.wav>
  skill_effects.txt    <skill>   <s|t>  <effect>  (s: on the one who used the skill, t: on its target)
  skill_hits.txt       <skill>   <s|t>  <effect>  (the same, for a skill that strikes: s at the striker's place,
                                                  t at the place of the one struck)
  skill_motions.txt    <skill>   <state>          (only skills whose state is not the usual one, 7)
  effects_missing.txt  <effect>  <EF_NAME>  <skills that use it>   effects of the list above that have no .str
                       file: the client draws them by code, and this client does not yet
"""
import os
import re
import sys

root, out = sys.argv[1], sys.argv[2]


def read(path):
    return open(os.path.join(root, path), encoding='utf-8', errors='replace').read()


def enum(text):
    """NAME -> number of a C enum's entries, counting on from the last number given."""
    found, last = {}, -1
    for name, value in re.findall(r'^\s*([A-Z][A-Z0-9_]*)\s*(?:=\s*(0x[0-9A-Fa-f]+|\d+))?\s*,', text, re.M):
        last = int(value, 0) if value else last + 1
        found[name] = last
    return found


def switch(text, indent):
    """The arms of a switch whose `case` lines stand `indent` deep: (labels, body), an arm that falls through
    into the next one giving its labels to it."""
    arms, labels, body = [], [], []
    case = re.compile(r'^' + indent + r'case ([A-Za-z0-9_]+):')
    carried = []
    for line in text.split('\n'):
        found = case.match(line)
        if found:
            if body:
                ends = any(re.match(r'^\s*(break|return)\b', row) for row in body)
                arms.append((carried + labels, body))
                carried = [] if ends else carried + labels
                labels, body = [], []
            labels.append(found.group(1))
        elif labels:
            body.append(line)
    if labels:
        arms.append((carried + labels, body))
    return arms


def function(text, start):
    """The text of the function that starts with `start`, to its closing brace in the first column."""
    at = text.index(start)
    return text[at:text.index('\n}\n', at)]


skills = enum(read('Source/Contents/Skills/SkillId.h'))
effects = enum(read('Source/Contents/Effects/Core/EffectTypes.h'))

# Which file an effect plays. One with a lighter twin for the "simple effects" option is taken whole; one picked by
# lot among numbered files is left to the code that plays it (it has a `%d` in its name).
files = {}
for labels, body in switch(read('Source/Contents/Effects/Core/RagEffectInitialization.cpp'), '  '):
    names = re.findall(r'"([^"%]+\.str)"', '\n'.join(body))
    if names:
        for label in labels:
            if label in effects:
                files[effects[label]] = names[-1]

# The sound an effect starts with.
sounds = {}
for labels, body in switch(read('Source/Contents/Effects/Core/RagEffect.cpp'), '  '):
    waves = re.findall(r'Play(?:Effect)?Wave\([^"]*"([^"]+\.wav)"', '\n'.join(body))
    if waves:
        for label in labels:
            if label in effects and effects[label] not in sounds:
                sounds[effects[label]] = waves[0].replace('\\\\', '/')

# The effects of a skill that strikes nobody.
used = function(read('Source/Client/Mode/GameModePackets.cpp'), 'void CGameMode::ReceiveSkillUse(')
used = used[used.index('  CGameActor* target;'):]
skill_effects = []
for labels, body in switch(used, '  '):
    text = '\n'.join(body)
    found = re.findall(r'\b(source|target)->LaunchEffect\((EF_[A-Z0-9_]+)', text)
    found += [(who, effect) for effect, who in re.findall(r'g_effectMgr->Spawn\([^,]*,\s*(EF_[A-Z0-9_]+),\s*(source|target)->', text)]
    for label in labels:
        if label not in skills:
            continue
        for who, effect in found:
            if effect in effects and (skills[label], who[0], effects[effect]) not in skill_effects:
                skill_effects.append((skills[label], who[0], effects[effect]))

# The effects of a skill that strikes somebody. Each is launched by one of the two and then told a place
# (`AM_SETPOS` with the source's or the target's position): the place is what is kept; one told no place is at
# the one who launched it.
struck = function(read('Source/Contents/Jobs/JobInheritance.cpp'), 'void CSession::PlaySkillEffect(')
skill_hits = []
for labels, body in switch(struck, '  '):
    found = []
    for line in body:
        launched = re.search(r'\b(source|target)->LaunchEffect\((EF_[A-Z0-9_]+)', line)
        spawned = re.search(r'g_effectMgr->Spawn\([^,]*,\s*(EF_[A-Z0-9_]+),\s*(source|target)->', line)
        placed = re.search(r'position = (source|target)->render\.position', line)
        if launched:
            found.append([launched.group(1), launched.group(2), False])
        elif spawned:
            found.append([spawned.group(2), spawned.group(1), True])
        elif placed and found and not found[-1][2]:
            found[-1][0], found[-1][2] = placed.group(1), True
    for label in labels:
        if label not in skills:
            continue
        for who, effect, _ in found:
            if effect in effects and (skills[label], who[0], effects[effect]) not in skill_hits:
                skill_hits.append((skills[label], who[0], effects[effect]))

# The motion of the body: the state a skill puts its user in, where it is not 7.
motions = {}
info = function(read('Source/Contents/Skills/SkillActionInfo.cpp'), 'void CSession::GetSkillActionInfo2(')
for labels, body in switch(info, '  '):
    found = re.findall(r'\bmotion = (\d+);', '\n'.join(body))
    if found:
        for label in labels:
            if label in skills and int(found[-1]) != 7:
                motions[skills[label]] = int(found[-1])

os.makedirs(out, exist_ok=True)


def write(name, rows):
    open(os.path.join(out, name), 'w', encoding='utf-8').write(''.join('\t'.join(str(field) for field in row) + '\n' for row in rows))


# The files this client already knew (from the client's own table of effect files) stay; the names found here
# are added to them.
known = {}
path = os.path.join(out, 'effect_files.txt')
if os.path.exists(path):
    for line in open(path, encoding='utf-8'):
        number, _, name = line.rstrip('\n').partition('\t')
        if number.isdigit():
            known[int(number)] = name
for number, name in files.items():
    known.setdefault(number, name)
write('effect_files.txt', sorted(known.items()))
write('effect_sounds.txt', sorted(sounds.items()))
write('skill_effects.txt', sorted(skill_effects))
write('skill_hits.txt', sorted(skill_hits))
write('skill_motions.txt', sorted(motions.items()))
names = {number: name for name, number in effects.items()}
missing = {}
for skill, _, effect in skill_effects + skill_hits:
    if effect not in known:
        missing.setdefault(effect, []).append(skill)
skill_names = {number: name for name, number in skills.items()}
write('effects_missing.txt', [(effect, names[effect], ' '.join(skill_names[skill] for skill in sorted(set(users)))) for effect, users in sorted(missing.items())])
print(f'{len(skills)} skills, {len(effects)} effects; {len(known)} effect files ({len(files)} from the code), {len(sounds)} sounds, '
      f'{len(skill_effects)} skill effects of {len(set(skill for skill, _, _ in skill_effects))} skills, '
      f'{len(skill_hits)} hit effects of {len(set(skill for skill, _, _ in skill_hits))} skills, {len(motions)} motions, {len(missing)} effects without a file')
