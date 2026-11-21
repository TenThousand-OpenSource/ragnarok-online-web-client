#!/usr/bin/env python3
"""Walks everything the classic game shows through the rules that name its sprite files, and asks the gateway
for each file: what is listed is drawn by the Windows client's rule and missing here.

    tools/sprite_files.py [--gateway http://127.0.0.1:8080] [--rathena ../rathena] [--all]

The names come from the client's rules themselves (`client/src/dress.rs`, `creatures.rs`, `pose.rs`), asked
through `cargo run -p roweb-client --example sprite_paths`, with the gateway's `/tables.json` for what the
client reads from its files (the jobs' names, the weapons' endings, the service). Walked:

  players    the classic jobs (0-25, 4001-4049), both sexes (a Bard is a man and a Dancer a woman): body
             `.spr`/`.act`, `.imf`, and the clothes' palette for the colours 0 to 4 the server may tell;
  weapons    each of those on foot with every kind of weapon an item of the pre-renewal database lets it hold;
  shields    the same with the shields' views (guard, buckler, shield, mirror shield);
  monsters   every monster of db/pre-re/mob_db.yml;
  NPCs       every sprite the scripts of npc/pre-re/scripts_main.conf give an NPC (but the unseen ones);
  companions the homunculi (6001-6016) and the mercenaries (6017-6046).

A weapon or a shield without a file may be one the Windows client has no picture of either (a Priest with a
knuckle): the item database lets more be held than was ever drawn. Those are listed apart, by the kind and the
jobs, and are no fault of the rule. `--all` lists every missing file instead of the first of each group.
Needs PyYAML and a gateway that runs; it only reads (HEAD requests).
"""
import argparse
import collections
import json
import os
import re
import subprocess
import sys
import urllib.error
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor

import yaml

HERE = os.path.dirname(os.path.abspath(__file__))
# The classic jobs, and those drawn on a mount: the client shows neither weapon nor shield on one
# (`CPc::SetSprWeapon2`, `SetSprShield`).
JOBS = list(range(0, 26)) + list(range(4001, 4050))
MOUNTED = {13, 21, 4014, 4022, 4036, 4044}
UNDRAWN = {22}  # the wedding dress is a state, not a job a server gives
MEN, WOMEN = {19, 4020, 4042}, {20, 4021, 4043}
# What the server's item database calls the owners of an item, by job.
OWNERS = {'Novice': [0, 4001, 4023], 'Swordman': [1, 4002, 4024], 'Mage': [2, 4003, 4025], 'Archer': [3, 4004, 4026],
          'Acolyte': [4, 4005, 4027], 'Merchant': [5, 4006, 4028], 'Thief': [6, 4007, 4029], 'Knight': [7, 4008, 4030],
          'Priest': [8, 4009, 4031], 'Wizard': [9, 4010, 4032], 'Blacksmith': [10, 4011, 4033], 'Hunter': [11, 4012, 4034],
          'Assassin': [12, 4013, 4035], 'Crusader': [14, 4015, 4037], 'Monk': [15, 4016, 4038], 'Sage': [16, 4017, 4039],
          'Rogue': [17, 4018, 4040], 'Alchemist': [18, 4019, 4041], 'BardDancer': [19, 20, 4020, 4021, 4042, 4043],
          'SuperNovice': [23, 4045], 'Gunslinger': [24], 'Ninja': [25], 'Taekwon': [4046], 'StarGladiator': [4047],
          'SoulLinker': [4049]}
# The client's numbers for the kinds of weapon (`crate::jobs::weapon_kind`).
KINDS = {'Dagger': 1, '1hSword': 2, '2hSword': 3, '1hSpear': 4, '2hSpear': 5, '1hAxe': 6, '2hAxe': 7, 'Mace': 8, '2hMace': 9,
         'Staff': 10, 'Bow': 11, 'Knuckle': 12, 'Musical': 13, 'Whip': 14, 'Book': 15, 'Katar': 16, 'Revolver': 17, 'Rifle': 18,
         'Gatling': 19, 'Shotgun': 20, 'Grenade': 21, 'Huuma': 22, '2hStaff': 23}
SHIELDS = {1: '_가드', 2: '_버클러', 3: '_쉴드', 4: '_미러쉴드'}


def ask(requests):
    """The rules' answers to the requests (lists of fields), in order."""
    lines = ''.join('\t'.join(str(field) for field in request) + '\n' for request in requests)
    run = subprocess.run(['cargo', 'run', '-q', '-p', 'roweb-client', '--example', 'sprite_paths'], input=lines, capture_output=True,
                         text=True, cwd=os.path.dirname(HERE), check=True)
    return run.stdout.split('\n')[:len(requests)]


def held(rathena):
    """By job: the kinds of weapon and the views of shield some item of the classic database lets it hold."""
    weapons, shields = collections.defaultdict(set), collections.defaultdict(set)
    for name in ('item_db_equip.yml',):
        for item in yaml.safe_load(open(os.path.join(rathena, 'db/pre-re', name), encoding='utf-8'))['Body']:
            owners = item.get('Jobs') or {'All': True}
            jobs = set(JOBS) if owners.get('All') else set()
            for owner, allowed in owners.items():
                jobs = jobs | set(OWNERS.get(owner, [])) if allowed else jobs - set(OWNERS.get(owner, []))
            classes = item.get('Classes') or {'All': True}
            if not classes.get('All'):
                jobs = {job for job in jobs if classes.get('Upper') and 4001 <= job <= 4022 or classes.get('Baby') and 4023 <= job <= 4045
                        or classes.get('Normal') and not 4001 <= job <= 4045}
            if item.get('Type') == 'Weapon' and item.get('SubType') in KINDS:
                for job in jobs:
                    weapons[job].add(KINDS[item['SubType']])
            elif item.get('Type') == 'Armor' and (item.get('Locations') or {}).get('Left_Hand') and item.get('View') in SHIELDS:
                for job in jobs:
                    shields[job].add(item['View'])
    return weapons, shields


def script_sprites(rathena):
    """The sprite of every NPC the classic scripts place, as a number: {job: how many NPCs}."""
    names = {}
    # The server's own names for the sprites (`enum e_job_types`), numbered as an enum numbers them.
    value = -1
    for line in open(os.path.join(rathena, 'src/map/npc.hpp'), encoding='utf-8'):
        found = re.match(r'\s*(\w+)\s*(?:=\s*(-?\d+))?\s*,', line)
        if found and (found.group(1).startswith('JT_') or found.group(1).startswith('NPC_RANGE')):
            value = int(found.group(2)) if found.group(2) else value + 1
            names[found.group(1).removeprefix('JT_')] = value
    for mob in yaml.safe_load(open(os.path.join(rathena, 'db/pre-re/mob_db.yml'), encoding='utf-8'))['Body']:
        names.setdefault(mob['AegisName'], mob['Id'])

    def files(conf):
        for line in open(os.path.join(rathena, conf), encoding='utf-8', errors='replace'):
            found = re.match(r'(npc|import):\s*(\S+)', line.split('//')[0].strip())
            if found:
                yield from files(found.group(2)) if found.group(1) == 'import' else [found.group(2)]

    used = collections.Counter()
    for file in files('npc/pre-re/scripts_main.conf'):
        if not os.path.exists(os.path.join(rathena, file)):
            continue
        for line in open(os.path.join(rathena, file), encoding='utf-8', errors='replace'):
            fields = line.rstrip('\n').split('\t')
            if line.startswith('//') or len(fields) < 4 or fields[0].startswith('-') or not re.match(r'(script|shop|cashshop|itemshop|pointshop|marketshop|duplicate\()', fields[1]):
                continue
            sprite = fields[3].split(',')[0].split('{')[0].strip()
            job = int(sprite) if re.fullmatch(r'-?\d+', sprite) else names.get(sprite)
            if job is not None and job >= 0:
                used[job] += 1
    return used


def walk(tables, rathena):
    """Everything walked, as (group, what it is, the files it needs)."""
    weapons, shields = held(rathena)
    sexes = lambda job: [1] if job in MEN else [0] if job in WOMEN else [0, 1]
    people = [(job, sex) for job in JOBS if job not in UNDRAWN for sex in sexes(job)]
    korea = 1 if tables.get('korea', True) else 0
    requests = [(part, job, sex) for job, sex in people for part in ('body', 'imf', 'weapon', 'shield')]
    requests += [('palette', job, sex, colour, korea) for job, sex in people for colour in range(5)]
    kinds = sorted({(job, kind) for job in JOBS for kind in weapons[job]})
    requests += [('shown', job, kind) for job, kind in kinds]
    mobs = [mob['Id'] for mob in yaml.safe_load(open(os.path.join(rathena, 'db/pre-re/mob_db.yml'), encoding='utf-8'))['Body']]
    placed = script_sprites(rathena)
    creatures = [('monsters', job) for job in mobs] + [('NPCs', job) for job in sorted(placed) if job not in (45, 111, 139, 32767)]
    creatures += [('companions', job) for job in range(6001, 6047)]
    requests += [('creature', job, tables['sprites'].get(str(job), '')) for _, job in creatures]
    answers = dict(zip(requests, ask(requests)))
    walked = []
    for job, sex in people:
        body = answers[('body', job, sex)]
        walked.append(('players', f'job {job} sex {sex} body', [body + '.spr', body + '.act']))
        walked.append(('players', f'job {job} sex {sex} imf', [answers[('imf', job, sex)]]))
        for colour in range(5):
            palette = answers[('palette', job, sex, colour, korea)]
            if palette != '-':
                walked.append(('players', f'job {job} sex {sex} colour {colour}', [palette]))
        if job in MOUNTED:
            continue
        for kind in sorted(weapons[job]):
            ending = tables['weapons'].get(answers[('shown', job, kind)])
            if ending:
                base = answers[('weapon', job, sex)] + ending
                walked.append(('weapons', f'kind {kind} ({ending}): job {job} sex {sex}', [base + '.spr', base + '.act']))
        for view in sorted(shields[job]):
            base = answers[('shield', job, sex)] + SHIELDS[view]
            walked.append(('shields', f'view {view} ({SHIELDS[view]}): job {job} sex {sex}', [base + '.spr', base + '.act']))
    for group, job in creatures:
        answer = answers[('creature', job, tables['sprites'].get(str(job), ''))].split('\t')
        count = f' x{placed[job]}' if group == 'NPCs' else ''
        if answer[0] == 'sprite':
            walked.append((group, f'job {job}{count}', [answer[1] + '.spr', answer[1] + '.act']))
        elif answer[0] == 'model':
            walked.append((group + ' (models)', f'job {job}{count}', ['data/model/3dmob/' + answer[1]]))
        elif answer[0] == 'character':
            walked.append((group + ' (characters)', f'job {job}{count}', []))
        else:
            walked.append((group, f'job {job}{count}: no name in the client\'s table', ['?']))
    return walked


def main():
    options = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    options.add_argument('--gateway', default='http://127.0.0.1:8080')
    options.add_argument('--rathena', default=os.path.join(HERE, '../../rathena'))
    options.add_argument('--all', action='store_true')
    options = options.parse_args()
    tables = json.load(urllib.request.urlopen(options.gateway + '/tables.json'))

    def there(path):
        if path == '?':
            return False
        try:
            return urllib.request.urlopen(urllib.request.Request(options.gateway + '/assets/' + urllib.parse.quote(path), method='HEAD')).status == 200
        except urllib.error.HTTPError:
            return False

    walked = walk(tables, options.rathena)
    paths = sorted({path for _, _, files in walked for path in files})
    with ThreadPoolExecutor(16) as pool:
        found = dict(zip(paths, pool.map(there, paths)))
    total = 0
    for group in dict.fromkeys(group for group, _, _ in walked):
        rows = [(what, [path for path in files if not found[path]]) for kind, what, files in walked if kind == group]
        missing = [(what, lost) for what, lost in rows if lost]
        total += len(missing)
        print(f'{group}: {len(rows)} walked, {len(missing)} with a missing file')
        if group in ('weapons', 'shields') and not options.all:
            # By kind: a kind no job of a line has a file for was never drawn.
            kinds = collections.defaultdict(list)
            for what, _ in missing:
                kinds[what.split(':')[0]].append(what.split(': ')[1])
            for kind, who in kinds.items():
                print(f'    {kind}: {len(who)} — {", ".join(who[:6])}{" …" if len(who) > 6 else ""}')
        else:
            for what, lost in missing if options.all else missing[:12]:
                print(f'    {what}: {lost[0]}')
            if len(missing) > 12 and not options.all:
                print(f'    … and {len(missing) - 12} more (--all)')
    print(f'missing in all: {total}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
