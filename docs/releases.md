# Maintaining releases

The `Validate and release` workflow runs on pull requests, pushes to `main`, and manual
dispatch. Pull requests validate metadata and test/build all four target platforms.
Main pushes publish when the current workspace version has no published release.
Ordinary pushes at an already-published version run validation but do not republish.
Only dispatch on `main` can publish. The initial version is 0.3.2.

## Prepare locally

1. Update `[workspace.package].version` in `Cargo.toml` and both plugin manifests.
2. Update `plugins/okf/cli-compatibility.txt` if the skills require newer CLI behavior.
   Update the compatibility range shown in README.md too.
   Its two stable versions are inclusive minimum and exclusive maximum. Patch releases
   must preserve CLI behavior within the supported range. Never accept a new minor
   version without reviewing compatibility. The OKF document spec version is separate.
3. Run Cargo to update `Cargo.lock`, then `cargo xtask docs` to regenerate references.
4. Write detailed `releases/VERSION.md` notes, including behavior changes, migration
   instructions where applicable, and platform/feature caveats.
5. Run `python3 scripts/release.py validate`, `python3 -m unittest discover -s scripts/tests -v`,
   `cargo fmt --all -- --check`, `cargo test --workspace --locked`,
   `cargo test --workspace --locked --features okf-cli/url-sources`, and `cargo xtask skills`.
6. Commit and push to `main`. Review the workflow result and the published release.

`rust-toolchain.toml` pins the compiler. Builds use `--locked` and enable `url-sources`.
Linux builds use a native musl compiler and verify there is no ELF interpreter.
macOS uses separate native builds with deployment target 13.0. Full target CLI tests
and a packaged-binary smoke test run before publication. Older OS versions still need
separate testing before promising support.

## Publication and retries

The workflow serializes runs per ref, tests all targets, then creates a `vVERSION` tag
at the exact workflow commit. It uploads all archives, checksums, and standalone installer
to a draft and publishes only after uploads succeed. It does not mutate published releases.
A failed upload can be retried using **Re-run failed jobs** on the same commit. Existing
tags must resolve to that commit. If source changes are needed after a tag has been created,
use a new version; do not move the old tag. A draft with no tag requires manual inspection.
Never edit a released version's notes expecting the public release to change.

GitHub Actions must be enabled. Repository/organization policy must permit `contents: write`
for the publishing job, and tag rules must permit its `v*` tags. No personal token is required.
Optionally enable release immutability in repository settings. Build/test jobs have read-only
repository permission. Do not add signing credentials to pull-request jobs.

Signing/notarization is deferred: the first macOS downloads may encounter Gatekeeper blocks.
A later release will require Apple Developer ID credentials and secrets configured in GitHub.
Checksums detect damaged/mismatched downloads; they do not independently authenticate a
compromised release account. Users must trust the repository and downloaded installer.

## Installer and plugin boundaries

The optional shell installer requires curl, tar, and sha256sum or shasum, not OKF or Rust.
It accepts an explicit stable version and optional destination. Updating uses the same
command with a newer compatible version. It never edits shell startup files, uses sudo,
or replaces a destination symlink. An explicit older version permits deliberate rollback.
It verifies the checksum and runs the downloaded binary before replacing the destination.

The read-only plugin preflight requires sh and awk. It rejects missing, unparseable,
prerelease, too-old, and unsupported newer versions. It prints the resolved executable path.
Skills show installation instructions and stop CLI-dependent work on failure. They never
invoke the installer. Agent instructions are not a sandbox enforcement mechanism.
