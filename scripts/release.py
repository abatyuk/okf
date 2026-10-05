#!/usr/bin/env python3
"""Validate release metadata; prepare/publish only from the exact workflow commit."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[1]
TARGETS = ('aarch64-apple-darwin', 'x86_64-apple-darwin',
           'aarch64-unknown-linux-musl', 'x86_64-unknown-linux-musl')


def run(*args):
    return subprocess.check_output(args, text=True, cwd=ROOT).strip()


def metadata():
    version = tomllib.loads((ROOT / 'Cargo.toml').read_text())['workspace']['package']['version']
    assert re.fullmatch(r'(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)', version), 'Stable version required'
    for manifest in ('plugins/okf/.codex-plugin/plugin.json', 'plugins/okf/.claude-plugin/plugin.json'):
        assert json.loads((ROOT / manifest).read_text())['version'] == version, manifest
    lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())
    for package in lock['package']:
        if package['name'] in ('okf-cli', 'okf-core', 'xtask'):
            assert package['version'] == version, 'Cargo.lock version mismatch'
    minimum, maximum = (ROOT / 'plugins/okf/cli-compatibility.txt').read_text().split()
    def semver(value):
        assert re.fullmatch(r'(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)', value)
        return tuple(map(int, value.split('.')))
    assert semver(minimum) <= semver(version) < semver(maximum), 'CLI outside plugin range'
    notes = ROOT / 'releases' / f'{version}.md'
    assert notes.is_file() and len(notes.read_text().strip()) > 100, 'Detailed release notes required'
    for skill in (ROOT / 'plugins/okf/skills').glob('*/SKILL.md'):
        assert '../../scripts/check-cli.sh' in skill.read_text(), f'Missing preflight: {skill}'
    return version


def release_state(tag):
    # Listing avoids treating authentication/network errors as a missing release.
    releases = json.loads(run('gh', 'api', '--paginate', '--slurp',
                              f'repos/{os.environ["GITHUB_REPOSITORY"]}/releases?per_page=100'))
    return next((r for page in releases for r in page if r['tag_name'] == tag), None)


def prepare(version):
    tag = f'v{version}'
    state = release_state(tag)
    publish = state is None or state['draft']
    if publish:
        # Existing tags/drafts must refer to this exact commit, never move a tag.
        refs = run('git', 'ls-remote', 'origin', f'refs/tags/{tag}', f'refs/tags/{tag}^{{}}')
        if refs:
            commit = refs.splitlines()[-1].split()[0]
            assert commit == os.environ['GITHUB_SHA'], 'Existing tag belongs to another commit'
        if state is not None:
            assert refs, 'Draft has no tag; inspect it manually before retrying'
    with open(os.environ['GITHUB_OUTPUT'], 'a') as output:
        output.write(f'version={version}\npublish={str(publish).lower()}\n')


def publish(version):
    tag = f'v{version}'
    state = release_state(tag)
    if state and not state['draft']:
        print('Already published; leaving release unchanged.')
        return
    assets = ROOT / 'dist'
    expected = {f'okf-{version}-{target}.tar.gz' for target in TARGETS}
    assert {p.name for p in assets.glob('*.tar.gz')} == expected, 'Incomplete artifact set'
    (assets / 'SHA256SUMS').write_text(''.join(
        f'{hashlib.sha256((assets / name).read_bytes()).hexdigest()}  {name}\n'
        for name in sorted(expected)))
    sha = os.environ['GITHUB_SHA']
    refs = run('git', 'ls-remote', 'origin', f'refs/tags/{tag}', f'refs/tags/{tag}^{{}}')
    if refs:
        assert refs.splitlines()[-1].split()[0] == sha, 'Tag commit mismatch'
    else:
        run('git', 'tag', tag, sha)
        run('git', 'push', 'origin', f'refs/tags/{tag}')
    notes = str(ROOT / 'releases' / f'{version}.md')
    if state is None:
        run('gh', 'release', 'create', tag, '--verify-tag', '--draft', '--title', tag, '--notes-file', notes)
    else:
        run('gh', 'release', 'edit', tag, '--notes-file', notes)
    run('gh', 'release', 'upload', tag, '--clobber',
        *(str(assets / name) for name in sorted(expected)), str(assets / 'SHA256SUMS'),
        str(ROOT / 'scripts/install.sh'))
    run('gh', 'release', 'edit', tag, '--draft=false', '--latest')


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('command', choices=('validate', 'prepare', 'publish'))
    args = parser.parse_args()
    version = metadata()
    if args.command == 'prepare':
        prepare(version)
    elif args.command == 'publish':
        publish(version)
    else:
        print(f'Release metadata valid: {version}')
