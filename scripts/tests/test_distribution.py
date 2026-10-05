import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import shutil
import tarfile
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location('release', ROOT / 'scripts/release.py')
release = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(release)


class DistributionTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.bin = self.root / 'bin'
        self.bin.mkdir()
        self.env = dict(os.environ, PATH=f'{self.bin}:/usr/bin:/bin')
        plugin = self.root / 'plugin'
        (plugin / 'scripts').mkdir(parents=True)
        self.checker = plugin / 'scripts/check-cli.sh'
        shutil.copyfile(ROOT / 'plugins/okf/scripts/check-cli.sh', self.checker)
        (plugin / 'cli-compatibility.txt').write_text('0.3.2 0.4.0\n')

    def executable(self, name, text):
        path = self.bin / name
        path.write_text('#!/bin/sh\n' + text)
        path.chmod(0o755)
        return path

    def check(self, version):
        self.executable('okf', f'echo "okf {version} (OKF spec 0.2)"\n')
        return subprocess.run(['sh', str(self.checker)],
                              env=self.env, capture_output=True, text=True)

    def test_compatibility_boundaries(self):
        for version, accepted in [('0.3.1', False), ('0.2.9', False), ('0.3.2', True),
                                  ('0.3.10', True), ('0.4.0', False), ('1.0.0', False),
                                  ('0.3.3-beta.1', False), ('garbage', False)]:
            with self.subTest(version=version):
                self.assertEqual(self.check(version).returncode == 0, accepted)

    def test_missing_cli(self):
        result = subprocess.run(['sh', str(self.checker)],
                                env=self.env, capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('Installation:', result.stderr)

    def installer(self, corrupt=False, binary_version='0.3.2', symlink=False):
        asset = 'okf-0.3.2-x86_64-unknown-linux-musl.tar.gz'
        payload = self.root / 'okf'
        payload.write_text(f'#!/bin/sh\necho "okf {binary_version} (OKF spec 0.2)"\n')
        payload.chmod(0o755)
        archive = self.root / asset
        with tarfile.open(archive, 'w:gz') as tar:
            tar.add(payload, arcname='okf')
        digest = '0' * 64 if corrupt else hashlib.sha256(archive.read_bytes()).hexdigest()
        (self.root / 'SHA256SUMS').write_text(f'{digest}  {asset}\n')
        self.executable('uname', 'case "$1" in -s) echo Linux;; -m) echo x86_64;; esac\n')
        # Fake transport, exercise real extraction/checksum/staging without network.
        self.executable('curl', '''while [ "$#" -gt 0 ]; do
case "$1" in https://*) url=$1;; -o) shift; output=$1;; esac
shift
done
cp "$FIXTURES/${url##*/}" "$output"
''')
        self.env['FIXTURES'] = str(self.root)
        destination = self.root / 'install with spaces'
        destination.mkdir()
        previous = self.root / 'previous'
        previous.write_text('old binary')
        if symlink:
            (destination / 'okf').symlink_to(previous)
        else:
            (destination / 'okf').write_text('old binary')
        result = subprocess.run(['sh', str(ROOT / 'scripts/install.sh'), '0.3.2', str(destination)],
                                env=self.env, capture_output=True, text=True)
        return result, destination / 'okf'

    def test_install_update(self):
        result, binary = self.installer()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('0.3.2', binary.read_text())
        self.assertTrue(os.access(binary, os.X_OK))

    def test_checksum_failure_preserves_existing(self):
        result, binary = self.installer(corrupt=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(binary.read_text(), 'old binary')

    def test_wrong_binary_preserves_existing(self):
        result, binary = self.installer(binary_version='0.2.0')
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(binary.read_text(), 'old binary')

    def test_symlink_preserves_existing(self):
        result, binary = self.installer(symlink=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertTrue(binary.is_symlink())
        self.assertEqual(binary.read_text(), 'old binary')

    def test_published_release_is_not_modified(self):
        with patch.object(release, 'release_state', return_value={'draft': False}), \
                patch.object(release, 'run') as run:
            release.publish('0.3.2')
            run.assert_not_called()

    def test_prepare_rejects_existing_tag_at_other_commit(self):
        with patch.object(release, 'release_state', return_value=None), \
                patch.object(release, 'run', return_value='oldsha\trefs/tags/v0.3.2'), \
                patch.dict(os.environ, {'GITHUB_SHA': 'newsha'}):
            with self.assertRaisesRegex(AssertionError, 'another commit'):
                release.prepare('0.3.2')

    def test_metadata(self):
        self.assertRegex(release.metadata(), r'^\d+\.\d+\.\d+$')


    def test_incomplete_release_never_creates_tag(self):
        (self.root / 'dist').mkdir()
        with patch.object(release, 'ROOT', self.root), \
                patch.object(release, 'release_state', return_value=None), \
                patch.object(release, 'run') as run:
            with self.assertRaisesRegex(AssertionError, 'Incomplete artifact'):
                release.publish('0.3.2')
            run.assert_not_called()

    def test_publish_upload_failure_leaves_draft(self):
        dist = self.root / 'dist'
        dist.mkdir()
        for target in release.TARGETS:
            (dist / f'okf-0.3.2-{target}.tar.gz').write_bytes(b'fixture')
        def command(*args):
            if args[:3] == ('gh', 'release', 'upload'):
                raise subprocess.CalledProcessError(1, args)
            if args[:2] == ('git', 'ls-remote'):
                return 'sha\trefs/tags/v0.3.2'
            return ''
        with patch.object(release, 'ROOT', self.root), \
                patch.object(release, 'release_state', return_value=None), \
                patch.dict(os.environ, {'GITHUB_SHA': 'sha'}), \
                patch.object(release, 'run', side_effect=command) as run:
            with self.assertRaises(subprocess.CalledProcessError):
                release.publish('0.3.2')
            calls = [call.args for call in run.call_args_list]
            self.assertTrue(any('--draft' in args for args in calls))
            self.assertFalse(any('--draft=false' in args for args in calls))
