#!/usr/bin/env python3
"""Writes out the names the client keeps for the files of a job's parts, apart from the body's.

    job_files.py <rodecomp folder> <output file> [<job_sprites.txt to compare the bodies with>]

The client has a table of names for each part of a character, all compiled into it and all numbered by
"sprite job" (the job, less 3950 above 3950). Read from the decompiled client (`rodecomp`),
Source/Client/Session/Session.cpp:
  CSession::InitResourceNameTables  m_imfNames (the `.imf`), m_shieldJobNames (the shield's folder and name),
                                    m_palJobNames (the clothes' palette; two lists, by the service the
                                    client is set up for in clientinfo.xml: Korea's, and everyone else's)
  CSession::InitJobSpriteNames      m_jobSpriteNames (the weapon's folder and name)
and, to compare with the list kept by hand in `job_sprites.txt`, m_bodyJobNames.

Written, one entry a line: <table><TAB><sprite job><TAB><name>, the tables being `imf`, `weapon`, `shield`,
`palette` (Korea's list) and `palette_abroad` (the rows the other services have differently). A job without
a line has the Novice's name. Names the client makes up as it runs (the latest jobs' `…Name().Text()`) are
resolved where the decompilation names their text, and left out otherwise (said on the way).
"""
import re
import sys

root, out = sys.argv[1], sys.argv[2]
text = open(f'{root}/Source/Client/Session/Session.cpp', encoding='utf-8', errors='replace').read()
# The helpers that give a name kept scrambled: `X() { OBFUSCATE_STRING_BODY("…", key); }`, anywhere in the source.
helpers = {}
import glob
for path in glob.glob(f'{root}/Source/**/*.cpp', recursive=True):
    source = open(path, encoding='utf-8', errors='replace').read()
    for name, value in re.findall(r'&\s*(\w+)\(\)\s*\{\s*OBFUSCATE_STRING_BODY\("((?:[^"\\]|\\.)*)"', source):
        helpers[name] = value


def function(name):
    body = text[text.index(f'void CSession::{name}()'):]
    return body[:body.index('\n}\n')]


def rows(body, table):
    """The assignments to one table in order, each with the branch it stands in: '' , 'korea' or 'abroad'."""
    found, branch, skipped = [], '', 0
    for line in body.split('\n'):
        if 'g_serviceType == ServiceKorea' in line:
            branch = 'korea'
        elif branch == 'korea' and line.strip() == '} else {':
            branch = 'abroad'
        elif branch and line == '  }':
            branch = ''
        match = re.match(rf'\s*{table}\[(\d+)\] = (.+);$', line)
        if not match:
            continue
        index, value = int(match.group(1)), match.group(2)
        text_of = re.match(r'(?:OBFUSCATE_ARRAY\()?"((?:[^"\\]|\\.)*)"', value)
        copy = re.match(rf'{table}\[(\d+)\]$', value)
        helper = re.match(r'(\w+)\(\)\.Text\(\)$', value)
        if text_of:
            found.append((branch, index, text_of.group(1).replace('\\\\', '\\')))
        elif copy:
            found.append((branch, index, ('copy', int(copy.group(1)))))
        elif helper and helper.group(1) in helpers:
            found.append((branch, index, helpers[helper.group(1)].replace('\\\\', '\\')))
        else:
            skipped += 1
    return found, skipped


def table(body, name, branch):
    """The table as the client has it after its assignments ran, for a client of that service."""
    found, skipped = rows(body, name)
    names = {}
    for where, index, value in found:
        if where in ('', branch):
            names[index] = names.get(value[1], '초보자') if isinstance(value, tuple) else value
    return names, skipped


resources, sprites = function('InitResourceNameTables'), function('InitJobSpriteNames')
lines, report = [], []
for label, body, name in (('imf', resources, 'imfNames'), ('weapon', sprites, 'jobSpriteNames'), ('shield', resources, 'shieldJobNames')):
    names, skipped = table(body, name, 'korea')
    lines += [f'{label}\t{index}\t{value}' for index, value in sorted(names.items())]
    report.append(f'{label}: {len(names)} ({skipped} left out)')
korea, skipped = table(resources, 'palJobNames', 'korea')
abroad, _ = table(resources, 'palJobNames', 'abroad')
lines += [f'palette\t{index}\t{value}' for index, value in sorted(korea.items())]
lines += [f'palette_abroad\t{index}\t{value}' for index, value in sorted(abroad.items()) if korea.get(index) != value]
report.append(f'palette: {len(korea)} ({skipped} left out), {sum(1 for i, v in abroad.items() if korea.get(i) != v)} differ abroad')
open(out, 'w', encoding='utf-8').write(
    '# The names the original client (Ragexe 2026-02-19) keeps for the files of a job\'s parts, by sprite job (the\n'
    '# job, less 3950 above 3950): its `m_imfNames`, `m_jobSpriteNames` (weapons), `m_shieldJobNames` and\n'
    '# `m_palJobNames` (the last as a client set up for Korea has it, and the rows another service has differently),\n'
    '# written out by tools/client-tables/job_files.py. A job without a line has the Novice\'s name.\n'
    + '\n'.join(lines) + '\n')
print('; '.join(report))
if len(sys.argv) > 3:
    bodies, _ = table(resources, 'bodyJobNames', 'korea')
    kept = dict(line.split('\t') for line in open(sys.argv[3], encoding='utf-8').read().split('\n') if line and not line.startswith('#'))
    differ = [(index, value, kept.get(str(index))) for index, value in sorted(bodies.items()) if kept.get(str(index)) != value]
    print(f'bodies: {len(bodies)} in the client, {len(kept)} kept, {len(differ)} differ: {differ[:20]}')
