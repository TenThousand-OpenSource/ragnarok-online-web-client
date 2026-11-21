#!/usr/bin/env python3
"""Writes out the tables of skills and effects the original client has compiled into it.

    skill_effects.py <rodecomp folder> <output folder>

Read from the decompiled client (`rodecomp`):
  Source/Contents/Skills/SkillId.h                    the skills' numbers
  Source/Contents/Effects/Core/EffectTypes.h          the effects' numbers (EF_…)
  Source/Contents/Effects/Core/RagEffectInitialization.cpp   which .str file an effect plays
  Source/Contents/Effects/Particles/EZ2STREffect.cpp  CEZ2STRScreenEffect::InitParticle: the .str file of an
                                                      effect that is nothing but its file; InitEvents: its sound
  Source/Contents/Effects/Core/RagEffect.cpp          which sound an effect starts with: the arms of its
                                                      switches by the effect, and of the second switch of
                                                      `CRagEffect::OnProcess`, which stands a step deeper
  Source/Client/Mode/GameModePackets.cpp              CGameMode::ReceiveSkillUse: the effects of a skill that
                                                      strikes nobody, on the one who used it or on its target
  Source/Client/Mode/GameModeNotices.cpp              CGameMode::ReceiveNotifyGroundSkill: the sounds of a skill
                                                      used on a place (with CGameActor::PlayGroundSkillEffect)
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
  effect_sounds.txt    <effect>  <sound.wav>  [<tick>]   (the first sound of the effect; for an effect of the
                                                  second switch of `OnProcess` every sound, a line each, with
                                                  the tick of the effect it starts at where that is not its first)
                                                  (an arm that hands the effect to a function of its own is
                                                  read one step into that function)
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
  skill_sounds.txt     <skill>   <u|h|g>  <s|t>  <sound.wav>   (the sounds the client's handlers of a skill start
                                                  beside its effects: u as it is used, h as it strikes, g as it
                                                  is used on a place; s heard from the one who used it, or for g
                                                  from the place, t from its target)
  effects_missing.txt  <effect>  <EF_NAME>  <skills and items that use it>   effects of the five tables of skills
                       above, and of `item_effects.txt`, that have no .str file here and are not drawn by code here.
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
    waves = re.findall(r'Play(?:Effect)?Wave(?:At)?\([^"]*"([^"]+\.wav)"', '\n'.join(body))
    if waves:
        for label in labels:
            if label in effects and effects[label] not in sounds:
                sounds[effects[label]] = waves[0].replace('\\\\', '/')

# The effects under 300 and from 374 to 490 are handed out by a second switch of `CRagEffect::OnProcess`, which
# stands inside the `else` of the first and so one step deeper. An arm there starts its sounds at ticks of the
# effect it names (`if (this->state.stateCnt == 20)`): each is kept with its tick, so that one started late is
# not heard early. A sound started under any other condition (by lot: Quagmire's one time in twelve; again and
# again) is left to the code that launches the effect.
#
# An arm that hands the effect to a function of its own (`if (!this->Aspersio())`, `this->TwoHandQuicken();`) has its sounds there, started
# the same way: that function is read as the arm is (one step in, no deeper). There a sound may start at one of
# several ticks (`stateCnt == 7 || stateCnt == 20`): a line for each.
more, skipped = {}, []
effect_code = read('Source/Contents/Effects/Core/RagEffect.cpp')
process = function(effect_code, 'bool CRagEffect::OnProcess() {')
TICKS = r'if \((this->state\.stateCnt == \d+(?: \|\| this->state\.stateCnt == \d+)*)\) \{'


def timed_sounds(lines, label, found):
    """The sounds these lines start at ticks of the effect, added to `found`; the others are noted as skipped."""
    under = None
    for line in lines:
        if re.match(r'\s*(if|\} else|for|while)\b', line):
            under = line.strip()
        wave = re.search(r'Play(?:Effect)?Wave(?:At)?\([^"]*"([^"]+\.wav)"', line)
        if wave:
            counted = re.fullmatch(TICKS, under or '')
            if counted:
                found.extend((wave.group(1).replace('\\\\', '/'), int(tick)) for tick in re.findall(r'\d+', counted.group(1)))
            else:
                skipped.append((label, under))


for labels, body in switch(process[process.index('    switch (this->state.type) {'):], '    '):
    found, own = [], []
    for line in body:
        if re.match(r'^    \}', line):
            break  # (the switch is over: what follows is not this arm's)
        own.append(line)
        called = re.search(r'^\s*(?:if \(!)?this->(\w+)\(\)(?:\) \{|;)$', line)
        for start in [f'{kind} CRagEffect::{called.group(1)}() {{' for kind in ('bool', 'void')] if called else []:
            if start in effect_code:
                timed_sounds(function(effect_code, start).split('\n'), labels[-1], found)
    timed_sounds(own, labels[-1], found)
    for label in labels:
        if found and label in effects and label.startswith('EF_') and effects[label] not in sounds:
            more[effects[label]] = found

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
    # An effect launched on the player's own character and told the place of one of the two (`AM_SETPOS`) is
    # seen at that one's place: written as on that one (`NPC_SUICIDE`).
    found += [(who, effect) for effect, who in re.findall(r'\bplayer->LaunchEffect\((EF_[A-Z0-9_]+)[^\n]*\n[^\n]*AM_SETPOS[^\n]*&(source|target)->render\.position', text)]
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
        # (One launched on the player's own character and then told a place, as `RG_INTIMIDATE`'s: the place
        # is what is kept.)
        own = re.search(r'\bplayer->LaunchEffect\((EF_[A-Z0-9_]+)', line)
        spawned = re.search(r'g_effectMgr->Spawn\([^,]*,\s*(EF_[A-Z0-9_]+),\s*(source|target)->', line)
        placed = re.search(r'position = (source|target)->render\.position', line)
        # A thing thrown: launched by the source and told the target's place (`CGameMode::LaunchTargetEffect`,
        # which names the effect by its number).
        thrown = re.search(r'LaunchTargetEffect\((0x[0-9A-Fa-f]+|\d+), source, target', line)
        if thrown:
            name = next((name for name, number in effects.items() if number == int(thrown.group(1), 0)), None)
            found.append(['target', name, True])
        elif launched:
            found.append([launched.group(1), launched.group(2), False])
        elif own:
            found.append([None, own.group(1), False])
        elif spawned:
            found.append([spawned.group(2), spawned.group(1), True])
        elif placed and found and not found[-1][2]:
            found[-1][0], found[-1][2] = placed.group(1), True
    for label in labels:
        if label not in skills:
            continue
        for who, effect, _ in found:
            if who and effect in effects and (skills[label], who[0], effects[effect]) not in skill_hits:
                skill_hits.append((skills[label], who[0], effects[effect]))

# The sounds the client starts in its handlers of a skill, beside the effects (`g_soundMgr->PlayWave` in the
# skill's arm): `u` as the skill is used (`CGameMode::ReceiveSkillUse`), from the one who used it or, where the
# arm asks only for a target, from that one; `h` as a skill that strikes is shown (`CSession::PlaySkillEffect`),
# from the one struck; `g` as a skill is used on a place (`CGameActor::PlayGroundSkillEffect`, and the switch
# of `CGameMode::ReceiveNotifyGroundSkill`, which stands a step deeper), from the place. A sound this client
# starts already with an effect of the skill (it is named in the effect's arm of a file of effects, whose
# `known` has the handler's sound too) is left out: `effect_sounded` below.
def handler_sounds(text, indent, when):
    found = []
    for labels, body in switch(text, indent):
        under = ''
        for line in body:
            if re.match(r'^' + indent + r'\}', line):
                break  # (the switch is over)
            if re.match(r'\s*(if|\} else)\b', line):
                under = line
            elif re.match(r'^' + indent + r'  \}$', line):
                under = ''
            wave = re.search(r'g_soundMgr->PlayWave\("([^"]+\.wav)"', line)
            if wave:
                who = 't' if when == 'h' or ('target' in under and 'source' not in under) else 's'
                found += [(skills[label], when, who, wave.group(1).replace('\\\\', '/')) for label in labels if label in skills]
    return found


skill_sounds = handler_sounds(used, '  ', 'u') + handler_sounds(struck, '  ', 'h')
skill_sounds += handler_sounds(function(read('Source/Contents/Actors/RenderObject.cpp'), 'void CGameActor::PlayGroundSkillEffect('), '  ', 'g')
skill_sounds += handler_sounds(function(read('Source/Client/Mode/GameModeNotices.cpp'), 'void CGameMode::ReceiveNotifyGroundSkill('), '    ', 'g')

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

# The sounds a file of effects here names in an effect's own arm (`266 => (300, "effect/….wav")`).
effect_sounded = set()
for folder, _, names in os.walk(prims):
    for name in names:
        if name.endswith('.rs'):
            for arm, sound in re.findall(r'^\s*(\d[\d\s|.=]*?)\s*(?:if [^=]*)?=>.*?"([^"]+\.wav)"', open(os.path.join(folder, name), encoding='utf-8').read(), re.M):
                for part in arm.split('|'):
                    first, _, last = part.strip().partition('..=')
                    effect_sounded.update((number, sound.lower()) for number in range(int(first), int(last or first) + 1))
shown_by = {}
for skill, _, effect in skill_effects + skill_hits:
    shown_by.setdefault(skill, set()).add(effect)
for skill, effect in list(launch.items()) + list(follow.items()):
    shown_by.setdefault(skill, set()).add(effect)
doubled = [row for row in skill_sounds if any((effect, row[3].lower()) in effect_sounded for effect in shown_by.get(row[0], ()))]
skill_sounds = sorted(set(skill_sounds) - set(doubled))

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
write('effect_sounds.txt', sorted([(number, sound) for number, sound in sounds.items()]
                                  + [(number, sound) + ((tick,) if tick else ()) for number, waves in more.items() if number not in sounds for sound, tick in waves]))
write('skill_effects.txt', sorted(skill_effects))
write('skill_hits.txt', sorted(skill_hits))
write('skill_launch.txt', sorted(launch.items()))
write('skill_follow.txt', sorted(follow.items()))
write('skill_struck.txt', sorted(struck_by.items()))
write('skill_motions.txt', sorted(motions.items()))
write('skill_sounds.txt', skill_sounds)
# The header has other lists of numbers beside the effects' (states, options): a number is named as an effect.
names = {}
for name, number in effects.items():
    if number not in names or (name.startswith('EF_') and not names[number].startswith('EF_')):
        names[number] = name
missing = {}
for skill, effect in [(skill, effect) for skill, _, effect in skill_effects + skill_hits] + list(launch.items()) + list(follow.items()) + list(struck_by.items()):
    if effect not in known and effect not in ported:
        missing.setdefault(effect, []).append(skill)
# The effects the items show as they are used (`item_effects.txt`, the folder's own table of the client's
# `CGameMode::PlayItemUseEffect`) are counted too, each user written as `item <number>`.
item_users = {}
items = os.path.join(out, 'item_effects.txt')
if os.path.exists(items):
    for line in open(items, encoding='utf-8'):
        item, _, effect = line.strip().partition('\t')
        if item.isdigit() and effect.isdigit() and int(effect) not in known and int(effect) not in ported:
            item_users.setdefault(int(effect), []).append(f'item {item}')
skill_names = {number: name for name, number in skills.items()}
write('effects_missing.txt', [
    (effect, names.get(effect, ''), ' '.join([skill_names.get(skill, str(skill)) for skill in sorted(set(missing.get(effect, ())))] + item_users.get(effect, [])))
    for effect in sorted(set(missing) | set(item_users))
])
print('sounds of a skill left to its effect here:', ', '.join(f'{skill} {sound}' for skill, _, _, sound in sorted(set(doubled))))
print('sounds left to the code:', ', '.join(f'{label} ({under})' for label, under in skipped))
print(f'{len(skills)} skills, {len(effects)} effects; {len(known)} effect files ({len(files)} and {len(plain)} from the code), {len(set(sounds) | set(more))} effects with sounds, '
      f'{len(skill_effects)} skill effects of {len(set(skill for skill, _, _ in skill_effects))} skills, '
      f'{len(skill_hits)} hit effects of {len(set(skill for skill, _, _ in skill_hits))} skills, {len(launch)} launched and {len(follow)} following effects, {len(struck_by)} effects of a hit, {len(motions)} motions, {len(skill_sounds)} sounds of {len(set(row[0] for row in skill_sounds))} skills, {len(ported)} effects drawn by code here, {len(set(missing) | set(item_users))} effects neither a file nor drawn here')
