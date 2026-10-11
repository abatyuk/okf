import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class MacOSSigningTests(unittest.TestCase):
    def run_signing(self, status, submit_exit=0):
        with tempfile.TemporaryDirectory() as directory:
            work = Path(directory)
            commands = work / 'bin'
            commands.mkdir()
            log = work / 'commands.log'
            mocks = {
                'security': '''echo "$1" >> "$COMMAND_LOG"
if [ "$1" = create-keychain ]; then touch "$4"; fi
''',
                'codesign': 'echo "codesign $1" >> "$COMMAND_LOG"\n',
                'ditto': 'touch "$4"\n',
                'xcrun': '''echo "xcrun $2" >> "$COMMAND_LOG"
if [ "$2" = submit ]; then
  printf '{"id":"test-submission","status":"%s"}\\n' "$NOTARY_STATUS"
  exit "$SUBMIT_EXIT"
fi
''',
            }
            for name, body in mocks.items():
                path = commands / name
                path.write_text('#!/bin/bash\nset -eu\n' + body)
                path.chmod(0o755)
            binary = work / 'okf'
            binary.write_bytes(b'fixture')
            env = dict(os.environ, PATH=f'{commands}:{os.environ["PATH"]}',
                       COMMAND_LOG=str(log), RUNNER_TEMP=str(work),
                       MACOS_CERTIFICATE_BASE64='Zml4dHVyZQ==',
                       MACOS_CERTIFICATE_PASSWORD='fixture',
                       MACOS_SIGNING_IDENTITY='fixture', APPLE_ID='fixture',
                       APPLE_TEAM_ID='fixture', APPLE_APP_SPECIFIC_PASSWORD='fixture',
                       NOTARY_STATUS=status, SUBMIT_EXIT=str(submit_exit))
            result = subprocess.run(['bash', str(ROOT / 'scripts/sign-macos.sh'), str(binary)],
                                    env=env, capture_output=True, text=True)
            self.assertFalse(list(work.glob('okf-signing.*')), 'Credentials must be cleaned up')
            calls = log.read_text()
            self.assertIn('delete-keychain', calls)
            return result, calls

    def test_accepted_submission(self):
        result, calls = self.run_signing('Accepted')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('codesign --verify', calls)
        self.assertNotIn('xcrun log', calls)

    def test_rejection_blocks_release_and_fetches_log(self):
        result, calls = self.run_signing('Invalid')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('xcrun log', calls)
        self.assertIn('refusing to release', result.stderr)

    def test_submit_failure_blocks_release(self):
        result, calls = self.run_signing('In Progress', submit_exit=1)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('xcrun log', calls)
