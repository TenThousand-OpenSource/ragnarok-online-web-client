#!/usr/bin/env python3
"""Writes out the chat commands the client has compiled into it, and the messages its options answer with.

    commands.py <rodecomp folder> <output file>

Read from the decompiled client (`rodecomp`):
  Source/UI/GameSettings/TalkType.h      what a command asks for, by number (TT_…)
  Source/Client/Session/Session.cpp      the table of commands (`m_commandIds`) and the commands `GetTalkType`
                                         compares the first word with itself
  Source/Contents/Tables/MessageTable.cpp  the lines of the message table by name (MSI_…): `messageNames`, the
                                         list compiled into the client, in which a name's place is its line
                                         (`FindMessageId`). `Source/Resource/MSGSTRINGID.h` is NOT read: it
                                         lacks the names the client misspells (`MIS_PUT_GUILD_NAME_IN`,
                                         `MIS_GUILD_TIP`, `MIS_GUILD_SYSTEM` at 2077 to 2079, one more later),
                                         so every number in it from 2077 on is three or four short.

Written, one entry a line:
  command<TAB>/who<TAB>TT_REQ_NUM_USER   a command and what it asks for; the first line of a command holds
  message<TAB>MSI_BGM_ON<TAB>31          where the message table has a message said when something is switched
                                         on or off (the names the client's `GameSettings.lub` gives its options)
"""
import os
import re
import sys

root, out = sys.argv[1], sys.argv[2]


def read(path):
    return open(os.path.join(root, path), encoding='utf-8', errors='replace').read()


asks = {int(number, 0): name for name, number in re.findall(r'^\s*(TT_\w+)\s*=\s*(0x[0-9A-Fa-f]+|\d+)', read('Source/UI/GameSettings/TalkType.h'), re.M)}
session = read('Source/Client/Session/Session.cpp')
rows, seen = [], set()


def add(command, number):
    # Commands are told apart in small letters; those the decompilation could not read are left out.
    if command.startswith('/') and '�' not in command and '\\x' not in command and command not in seen and number in asks:
        seen.add(command)
        rows.append(('command', command, asks[number]))


table = session[session.index('this->m_commandIds.clear();'):]
for command, number in re.findall(r'm_commandIds\.push_back\(\{"([^"]+)", (\d+)\}\)', table[:table.index('std::stable_sort(this->m_commandIds')]):
    add(command, int(number))
parser = session[session.index('int CSession::GetTalkType('):]
parser = parser[:parser.index('\n}\n')]
for condition, number in re.findall(r'if \(([^{]*?)\) \{\s*\*talkType = (0x[0-9A-Fa-f]+|\d+);', parser):
    for command in re.findall(r'tokenizer\[0\] == "([^"]+)"', condition):
        add(command, int(number, 0))

names = read('Source/Contents/Tables/MessageTable.cpp')
names = names[names.index('const char* const messageNames[] = {'):]
names = re.findall(r'^\s*(?:"(\w*)"|(nullptr)),', names[:names.index('};')], re.M)
messages, known = [], set()
for number, (name, _) in enumerate(names):
    # The client takes the first place a name stands at.
    if re.fullmatch(r'MSI_\w+_(?:ON|OFF)\d*', name) and name not in known:
        known.add(name)
        messages.append(('message', name, number))
open(out, 'w', encoding='utf-8').write(
    '# The chat commands of the original client (Ragexe 2026-02-19) and what each asks for (`CSession`\'s table of\n'
    '# commands and `CSession::GetTalkType`), and where its message table has what is said when something is\n'
    '# switched on or off. Written out by tools/client-tables/commands.py; the lines are explained there.\n'
    + ''.join('\t'.join(str(field) for field in row) + '\n' for row in rows + messages))
print(f'{len(rows)} commands, {len(messages)} messages')
