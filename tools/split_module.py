#!/usr/bin/env python3
"""Moves functions and methods of one large Rust file into child modules.

    split_module.py <file.rs> <plan.json>

The plan maps a child module to its doc line and to the items it takes:
    {"camera": {"doc": "...", "items": ["Camera::view", "subtract", "GROUND_SHADER"]}}

An item is a free function, a constant, or `Type::method`. Types stay in the parent, so their private fields
remain visible to every child; what moves is made `pub(super)` so the parent and the siblings can still call it.
The parent gains `mod child; pub use child::*;` lines (`use` when `"private": true`, or what `"use"` says).
"""
import json
import re
import sys

KEYWORD = re.compile(r'^(pub(\([a-z]+\))? )?(async )?(fn|struct|enum|const|static|type|impl|trait|thread_local!|macro_rules!|use|mod)\b')
NAME = re.compile(r'^(?:pub(?:\([a-z]+\))? )?(?:async )?(fn|const|static) (\w+)')
SHADER = re.compile(r'^(pub(\([a-z]+\))? )?const \w+: &str = "$')
METHOD = re.compile(r'^    (pub(\([a-z]+\))? )?(async )?fn (\w+)')
IMPL = re.compile(r'^impl(?:<[^>]*>)? (\w+) \{$')


def lead(lines, at, indent):
    """The first line of the comments and attributes standing right over line `at`."""
    while at > 0 and re.match(r'^' + indent + r'(///|//|#\[)', lines[at - 1]):
        at -= 1
    return at


def chunks(lines):
    """Top-level items as (start, end, kind, name)."""
    starts, in_text = [], False
    for number, line in enumerate(lines):
        if in_text:
            in_text = not line.startswith('";')
            continue
        if SHADER.match(line):
            in_text = True
        if KEYWORD.match(line):
            starts.append(number)
    found = []
    for index, start in enumerate(starts):
        end = starts[index + 1] if index + 1 < len(starts) else len(lines)
        end = lead(lines, end, '') if index + 1 < len(starts) else end
        named = NAME.match(lines[start])
        implemented = IMPL.match(lines[start])
        kind, name = (named.group(1), named.group(2)) if named else (('impl', implemented.group(1)) if implemented else ('other', ''))
        found.append((lead(lines, start, ''), end, kind, name, start))
    return found


def methods(lines, start, end):
    """The methods of an impl block as (start, end, name); `end` of the last one is the closing brace."""
    closing = max(number for number in range(start, end) if lines[number] == '}')
    starts = [number for number in range(start + 1, closing) if METHOD.match(lines[number])]
    found = []
    for index, at in enumerate(starts):
        stop = lead(lines, starts[index + 1], '    ') if index + 1 < len(starts) else closing
        found.append((lead(lines, at, '    '), stop, METHOD.match(lines[at]).group(4), at))
    return found, closing


def deeper(body):
    """Paths that started at the parent's parent start one module further up from a child."""
    return [re.sub(r'\bsuper::', 'super::super::', line) for line in body]


def widen(line, indent):
    """Makes a private `fn`/`const` visible to the parent module, and keeps what the parent showed its own
    parent visible there."""
    line = re.sub(r'^' + indent + r'pub\(super\) ', indent + 'pub(in super::super) ', line)
    return re.sub(r'^' + indent + r'(async fn|fn|const|static) ', indent + r'pub(super) \1 ', line)


def main():
    path, plan = sys.argv[1], json.load(open(sys.argv[2]))
    lines = open(path).read().split('\n')
    wanted = {item: module for module, about in plan.items() for item in about['items']}
    taken = {module: {'free': [], 'methods': {}} for module in plan}
    drop = set()
    seen = set()
    for start, end, kind, name, at in chunks(lines):
        if kind == 'impl':
            parts, _ = methods(lines, at, end)
            for first, stop, method, line in parts:
                key = f'{name}::{method}'
                if key in wanted:
                    seen.add(key)
                    body = deeper(lines[first:stop])
                    body[line - first] = widen(body[line - first], '    ')
                    while body and body[-1] == '':
                        body.pop()
                    taken[wanted[key]]['methods'].setdefault(name, []).append(body)
                    drop.update(range(first, stop))
        elif name in wanted:
            seen.add(name)
            body = deeper(lines[start:end])
            body[at - start] = widen(body[at - start], '')
            while body and body[-1] == '':
                body.pop()
            taken[wanted[name]]['free'].append(body)
            drop.update(range(start, end))
    missing = sorted(set(wanted) - seen)
    if missing:
        sys.exit(f'not found in {path}: {missing}')
    folder = path[:-3]
    import os
    os.makedirs(folder, exist_ok=True)
    declarations = []
    for module, about in plan.items():
        text = [f'//! {line}' for line in about['doc'].split('\n')] + ['', 'use super::*;', '']
        for body in taken[module]['free']:
            text += body + ['']
        for owner, bodies in taken[module]['methods'].items():
            text.append(f'impl {owner} {{')
            for index, body in enumerate(bodies):
                text += body + ([''] if index + 1 < len(bodies) else [])
            text += ['}', '']
        open(f'{folder}/{module}.rs', 'w').write('\n'.join(text).rstrip('\n') + '\n')
        declarations.append(f'mod {module};')
    # A child that holds only methods has nothing to import.
    uses = [f'{plan[module].get("use", "use" if plan[module].get("private") else "pub use")} {module}::*;' for module in plan if taken[module]['free']]
    kept = [line for number, line in enumerate(lines) if number not in drop]
    # The declarations go after the parent's own `use` lines.
    last_use = max(number for number, line in enumerate(kept) if line.startswith('use '))
    kept[last_use + 1:last_use + 1] = [''] + declarations + [''] + uses
    text = '\n'.join(kept)
    # Impl blocks left empty by the move go, and runs of blank lines shrink.
    text = re.sub(r'\nimpl \w+ \{\n\}\n', '\n', text)
    text = re.sub(r'\n{3,}', '\n\n', text)
    open(path, 'w').write(text)
    print(f'{path}: {len(lines)} lines -> {len(text.splitlines())}; ' + ', '.join(f'{module} {sum(len(body) for body in taken[module]["free"]) + sum(len(body) for bodies in taken[module]["methods"].values() for body in bodies)}' for module in plan))


if __name__ == '__main__':
    main()
