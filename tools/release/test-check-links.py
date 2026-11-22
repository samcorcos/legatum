#!/usr/bin/env python3
"""Regression tests for excluded internal document references in exported Markdown."""
import os
import subprocess
import tempfile
from pathlib import Path

checker = Path(__file__).with_name('check-links.py')
internal = 'docs/open-source/' + 'readiness.md'

with tempfile.TemporaryDirectory() as tmp:
    tree = Path(tmp)
    (tree / 'docs').mkdir()
    source = tree / 'docs/example.md'

    source.write_text(f'The internal checklist is {internal}.\n')
    result = subprocess.run(['python3', str(checker), str(tree)], capture_output=True, text=True)
    assert result.returncode == 1, result
    assert 'references to excluded internal docs: 1' in result.stdout, result.stdout
    assert f'docs/example.md:1: {internal}' in result.stdout, result.stdout

    source.write_text('The internal checklist is not named here.\n')
    result = subprocess.run(['python3', str(checker), str(tree)], capture_output=True, text=True)
    assert result.returncode == 0, result
    assert 'references to excluded internal docs: 0' in result.stdout, result.stdout

print('ok: exported Markdown cannot refer to the excluded readiness checklist')
