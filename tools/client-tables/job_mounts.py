#!/usr/bin/env python3
"""Writes out the client's table of what a job is drawn as in a state: mounted, in its machine, in the air.

    job_mounts.py <rodecomp folder> <output file>

Read from the decompiled client (`rodecomp`):
  Source/Contents/Jobs/JobId.h              the jobs' numbers (JT_…)
  Source/Contents/Actors/RenderObject.cpp   CGameActor::GetEffectiveJob: by the bits of the actor's state

Written, one entry a line, in the client's order: <bit of the state><TAB><job, or * for any><TAB><the job drawn>.
The first line whose bit is set and whose job is the actor's is the one that holds. The client asks for the
Mechanic's machine otherwise than by a bit (`GetMadogearType`); the server tells it by bit 22, which is what is
written here.
"""
import os
import re
import sys

root, out = sys.argv[1], sys.argv[2]


def read(path):
    return open(os.path.join(root, path), encoding='utf-8', errors='replace').read()


jobs, last = {}, -1
for name, value in re.findall(r'^\s*(JT_[A-Z0-9_]+)\s*(?:=\s*(0x[0-9A-Fa-f]+|\d+))?\s*,', read('Source/Contents/Jobs/JobId.h'), re.M):
    last = int(value, 0) if value else last + 1
    jobs[name] = last
text = read('Source/Contents/Actors/RenderObject.cpp')
text = text[text.index('int CGameActor::GetEffectiveJob('):]
text = text[:text.index('\n}\n')]
rows, bit, cases = [], None, []
for line in text.split('\n'):
    flag = re.search(r'if \(StateFlag(\d+)\(', line)
    machine = 'GetMadogearType() != 0' in line
    case = re.match(r'\s*case (JT_[A-Z0-9_]+):', line)
    back = re.match(r'\s*return (JT_[A-Z0-9_]+);', line)
    if flag and '!StateFlag' not in line:
        bit, cases = int(flag.group(1)), []
    elif machine:
        bit, cases = 22, []
    elif re.search(r'\bif \(', line):
        # A condition that is not a bit of the state (the client's own count of mounts): not for this table.
        bit, cases = None, []
    elif case:
        cases.append(case.group(1))
    elif back and bit is not None and back.group(1) in jobs:
        if cases:
            rows += [(bit, jobs[job], jobs[back.group(1)]) for job in cases if job in jobs]
            cases = []
        else:
            rows.append((bit, '*', jobs[back.group(1)]))
open(out, 'w', encoding='utf-8').write(
    '# What a job is drawn as in a state (mounted, in its machine, in the air, dressed for a wedding), as the\n'
    '# original client (Ragexe 2026-02-19) has it: its `CGameActor::GetEffectiveJob`, written out by\n'
    '# tools/client-tables/job_mounts.py. Bit of the state, job (* for any), the job drawn; the first that fits holds.\n'
    + ''.join(f'{bit}\t{job}\t{drawn}\n' for bit, job, drawn in rows))
print(f'{len(rows)} lines')
