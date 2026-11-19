#!/usr/bin/env python3
"""Writes out the tables of skills and effects the original client has compiled into it.

    skill_effects.py <rodecomp folder> <output folder>

Read from the decompiled client (`rodecomp`):
  Source/Contents/Skills/SkillId.h                    the skills' numbers
  Source/Contents/Effects/Core/EffectTypes.h          the effects' numbers (EF_…)
  Source/Contents/Effects/Core/RagEffectInitialization.cpp   which .str file an effect plays
  Source/Contents/Effects/Particles/EZ2STREffect.cpp  CEZ2STRScreenEffect::InitParticle: the .str file of an
                                                      effect that is nothing but its file; InitEvents: its sound
  Source/Contents/Effects/Core/RagEffect.cpp          which sound an effect starts with
  Source/Client/Mode/GameModePackets.cpp              CGameMode::ReceiveSkillUse: the effects of a skill that
                                                      strikes nobody, on the one who used it or on its target
  Source/Contents/Skills/SkillActionInfo.cpp          CSession::GetSkillActionInfo2: the motion of the body
  Source/Contents/Jobs/JobInheritance.cpp             CSession::PlaySkillEffect: the effects of a skill that
                                                      strikes somebody, and whose place each is put at
  Source/Client/Session/SessionState.cpp              the tables `skillEffectIds` and `skillFollowEffects`: the
                                                      effect the striker launches as a skill strikes, and the one
                                                      launched on the one struck as the blow lands
  Source/Contents/Actors/RenderObject.cpp             CGameActor::ProcessSkillMessage: the effect of each hit of a
                                                      skill on the one struck (the `hitEffect` of its message), and
                                                      the following effects it puts in place of the table's

Read from this client, beside the output folder: the effects it draws by code already, which are the numbers the
functions `known` of its files of effects answer for (`../web/world/prims/` and the folders in it), and the
effects it plays from files that are not the one file of the client's table (picked by lot, or carried by a
class of the client's), which are the numbers `played` of `../effect/played.rs` names.

Written, one entry a line, tab between the fields:
  effect_files.txt     <effect>  <file.str>       (merged with what the folder already has; without the effects
                                                  a `played` here answers for, whose files the code plays)
  effect_sounds.txt    <effect>  <sound.wav>
  skill_effects.txt    <skill>   <s|t>  <effect>  (s: on the one who used the skill, t: on its target)
  skill_hits.txt       <skill>   <s|t>  <effect>  (the same, for a skill that strikes: s at the striker's place,
                                                  t at the place of the one struck)
  skill_launch.txt     <skill>   <effect>         (launched by the striker, as `CGameActor::ProcessSkillMessage` does)
  skill_follow.txt     <skill>   <effect>         (launched on the one struck when the blow lands)
  skill_struck.txt     <skill>   <effect>         (shown on the one struck at each hit of the skill that hurts; a
                                                  skill not listed shows none. Two skills the client decides as
                                                  the blow comes are written as it decides without that: Resurrection
                                                  that hurts shows 152 and nothing following, written 0 here; Earth
                                                  Drive shows 1708 on anyone but the player's own character, written
                                                  as none here)
  skill_motions.txt    <skill>   <state>          (only skills whose state is not the usual one, 7)
  effects_missing.txt  <effect>  <EF_NAME>  <skills that use it>   effects of the five tables of skills above
                       that have no .str file: the client draws them by code, and this client does not yet.
                       (One is left, and stays: 941 EF_SPR_LIGHTPRINT3, whose colour the client reads from
                       memory nothing has written, so there is nothing to port. The list has no header: a line
                       is an effect.)
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

# The same for the effects that are nothing but a file (`particle->Open` and `particle->Load`, which both play the
# file named unless the "simple effects" option is on): the file that is not the lighter twin, under `mineffect`.
plain = {}
for labels, body in switch(function(read('Source/Contents/Effects/Particles/EZ2STREffect.cpp'), 'void CEZ2STRScreenEffect::InitParticle('), '  '):
    names = [name.replace('\\\\', '/') for name in re.findall(r'particle->(?:Open|Load)\("([^"%]+\.str)"', '\n'.join(body))]
    names = [name for name in names if not name.lower().startswith('mineffect/')]
    if names:
        for label in labels:
            if label in effects:
                plain[effects[label]] = names[-1]

# The sound an effect starts with.
sounds = {}
for labels, body in switch(read('Source/Contents/Effects/Core/RagEffect.cpp'), '  '):
    waves = re.findall(r'Play(?:Effect)?Wave\([^"]*"([^"]+\.wav)"', '\n'.join(body))
    if waves:
        for label in labels:
            if label in effects and effects[label] not in sounds:
                sounds[effects[label]] = waves[0].replace('\\\\', '/')

# An effect that is nothing but its file starts its sound as the file begins (`CEZ2STRScreenEffect::InitEvents`,
# an event at the file's frame 0).
for labels, body in switch(function(read('Source/Contents/Effects/Particles/EZ2STREffect.cpp'), 'void CEZ2STRScreenEffect::InitEvents('), '  '):
    waves = re.findall(r'AddEvent\(0, "([^"]+\.wav)"', '\n'.join(body))
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
    # One told that it guarded (`AM_MAKE_GUARD`) launches the shield of a guard on itself (`CGameActor::SendMsg`).
    found += [(who, 'EF_GUARD') for who in re.findall(r'\b(source|target)->SendMsg\(nullptr, AM_MAKE_GUARD\b', text)]
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

# The two tables the session fills when it starts: `key = <skill>;` and then the table's entry for it. An entry
# of -1 says no effect.
launch, follow, key = {}, {}, None
for line in read('Source/Client/Session/SessionState.cpp').split('\n'):
    keyed = re.match(r'\s*key = (\d+);', line)
    entry = re.match(r'\s*this->(skillEffectIds|skillFollowEffects)\[key\] = (-?\d+);', line)
    if keyed:
        key = int(keyed.group(1))
    elif entry and key is not None and int(entry.group(2)) >= 0:
        (launch if entry.group(1) == 'skillEffectIds' else follow)[key] = int(entry.group(2))

# The effect of each hit on the one struck, and the following effects put in place of the table's. The client
# starts from a rule by the skill's number (`int effect = skillId < … ? 0 : -1;`) and then has a switch of the
# skills that differ. An arm that decides by something else than the skill (an `if`) is left as the rule has it.
message = function(read('Source/Contents/Actors/RenderObject.cpp'), 'void CGameActor::ProcessSkillMessage(')
rule = re.search(r'int effect = skillId < (\d+) && !\(skillId >= (\d+) && skillId <= (\d+)\) \? 0 : -1;', message)
under, hole_from, hole_to = (int(number) for number in rule.groups())
struck_by = {number: 0 for number in skills.values() if 0 < number < under and not hole_from <= number <= hole_to}
for labels, body in switch(message[rule.end():message.index('  int action;', rule.end())], '  '):
    text = '\n'.join(body)
    if re.search(r'\bif \(', text):
        continue
    hit = re.search(r'^\s*effect = (-?\w+);', text, re.M)
    followed = re.search(r'^\s*followEffect = (-?\w+);', text, re.M)
    for label in labels:
        if label not in skills:
            continue
        if hit:
            struck_by[skills[label]] = int(hit.group(1), 0)
        if followed:
            follow[skills[label]] = int(followed.group(1), 0)
struck_by = {skill: effect for skill, effect in struck_by.items() if effect >= 0}
follow = {skill: effect for skill, effect in follow.items() if effect >= 0}

# The effects this client draws by code: the numbers before `=>` in the functions `known` of its files of
# effects. Those `played` names (`effect/played.rs`) are played from files that the code picks or carries: such
# an effect is not one of the plain files, whatever file the client's table has for it.
ported, picked = set(), set()
prims = os.path.join(out, '..', 'web', 'world', 'prims')
for path in sorted(os.path.join(folder, name) for folder, _, names in os.walk(prims) for name in names if name.endswith('.rs')) + [os.path.join(out, '..', 'effect', 'played.rs')]:
    text = open(path, encoding='utf-8').read()
    for start in re.finditer(r'\bfn (known|played)\(', text):
        end = text.find('\n}\n', start.start())
        for arm in re.findall(r'^\s*(\d[\d\s|.=]*?)\s*(?:if [^=]*)?=>', text[start.start():end if end >= 0 else len(text)], re.M):
            for part in arm.split('|'):
                first, _, last = part.strip().partition('..=')
                numbers = range(int(first), int(last or first) + 1)
                ported.update(numbers)
                if start.group(1) == 'played':
                    picked.update(numbers)

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
# A lighter twin written here before gives way to the file itself.
for number, name in plain.items():
    if number not in known or known[number].lower().startswith('mineffect/'):
        known[number] = name
for number in picked:
    known.pop(number, None)
write('effect_files.txt', sorted(known.items()))
write('effect_sounds.txt', sorted(sounds.items()))
write('skill_effects.txt', sorted(skill_effects))
write('skill_hits.txt', sorted(skill_hits))
write('skill_launch.txt', sorted(launch.items()))
write('skill_follow.txt', sorted(follow.items()))
write('skill_struck.txt', sorted(struck_by.items()))
write('skill_motions.txt', sorted(motions.items()))
# The header has other lists of numbers beside the effects' (states, options): a number is named as an effect.
names = {}
for name, number in effects.items():
    if number not in names or (name.startswith('EF_') and not names[number].startswith('EF_')):
        names[number] = name
missing = {}
for skill, effect in [(skill, effect) for skill, _, effect in skill_effects + skill_hits] + list(launch.items()) + list(follow.items()) + list(struck_by.items()):
    if effect not in known and effect not in ported:
        missing.setdefault(effect, []).append(skill)
skill_names = {number: name for name, number in skills.items()}
write('effects_missing.txt', [(effect, names.get(effect, ''), ' '.join(skill_names.get(skill, str(skill)) for skill in sorted(set(users)))) for effect, users in sorted(missing.items())])
print(f'{len(skills)} skills, {len(effects)} effects; {len(known)} effect files ({len(files)} and {len(plain)} from the code), {len(sounds)} sounds, '
      f'{len(skill_effects)} skill effects of {len(set(skill for skill, _, _ in skill_effects))} skills, '
      f'{len(skill_hits)} hit effects of {len(set(skill for skill, _, _ in skill_hits))} skills, {len(launch)} launched and {len(follow)} following effects, {len(struck_by)} effects of a hit, {len(motions)} motions, {len(ported)} effects drawn by code here, {len(missing)} effects neither a file nor drawn here')
