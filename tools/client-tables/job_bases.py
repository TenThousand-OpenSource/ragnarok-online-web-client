#!/usr/bin/env python3
"""Writes out the client's table of which job a mounted or otherwise re-dressed job is.

    job_bases.py <rodecomp folder> <output file>

Read from the decompiled client (`rodecomp`):
  Source/Contents/Jobs/JobId.h       the jobs' numbers (JT_…)
  Source/Contents/Jobs/JobNames.cpp  JobNames::BaseJob: a knight on a bird is a knight, and so on

Written, one entry a line: <job><TAB><the job it is>. A job without a line is itself.
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
text = read('Source/Contents/Jobs/JobNames.cpp')
text = text[text.index('unsigned JobNames::BaseJob('):]
text = text[:text.index('\n}\n')]
rows, labels = [], []
for line in text.split('\n'):
    case = re.match(r'\s*case (JT_[A-Z0-9_]+):', line)
    back = re.match(r'\s*return (JT_[A-Z0-9_]+);', line)
    if case:
        labels.append(case.group(1))
    elif back:
        rows += [(jobs[label], jobs[back.group(1)]) for label in labels if label in jobs and back.group(1) in jobs]
        labels = []
open(out, 'w', encoding='utf-8').write(
    '# The job a mounted or re-dressed job is, as the original client (Ragexe 2026-02-19) has it: its\n'
    '# `JobNames::BaseJob`, written out by tools/client-tables/job_bases.py. A job without a line is itself.\n'
    + ''.join(f'{job}\t{base}\n' for job, base in sorted(rows)))
print(f'{len(rows)} jobs')
