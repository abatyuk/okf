#!/bin/bash
# Sign and notarize a release binary before the workflow packages it.
set -euo pipefail
umask 077

binary=${1:?Usage: sign-macos.sh BINARY}
for variable in MACOS_CERTIFICATE_BASE64 MACOS_CERTIFICATE_PASSWORD MACOS_SIGNING_IDENTITY APPLE_ID APPLE_TEAM_ID APPLE_APP_SPECIFIC_PASSWORD RUNNER_TEMP; do
  if [ -z "${!variable:-}" ]; then
    echo "Missing required secret or environment variable: $variable" >&2
    exit 1
  fi
done

work=$(mktemp -d "$RUNNER_TEMP/okf-signing.XXXXXX")
keychain="$work/signing.keychain-db"
cleanup() {
  if [ -f "$keychain" ]; then
    security delete-keychain "$keychain" || true
  fi
  rm -rf "$work"
}
trap cleanup EXIT
keychain_password=$(openssl rand -hex 32)
echo "::add-mask::$keychain_password"
printf '%s' "$MACOS_CERTIFICATE_BASE64" | base64 --decode > "$work/certificate.p12"
security create-keychain -p "$keychain_password" "$keychain"
security set-keychain-settings -lut 21600 "$keychain"
security unlock-keychain -p "$keychain_password" "$keychain"
security import "$work/certificate.p12" -P "$MACOS_CERTIFICATE_PASSWORD" \
  -k "$keychain" -t cert -f pkcs12 -T /usr/bin/codesign
security set-key-partition-list -S apple-tool:,apple: -s -k "$keychain_password" "$keychain" > /dev/null
security list-keychains -d user -s "$keychain" "$HOME/Library/Keychains/login.keychain-db"
rm "$work/certificate.p12"

codesign --force --sign "$MACOS_SIGNING_IDENTITY" --keychain "$keychain" \
  --timestamp --options runtime "$binary"
codesign --verify --strict --verbose=2 "$binary"

mkdir "$work/payload"
cp "$binary" "$work/payload/okf"
ditto -c -k "$work/payload" "$work/submission.zip"
# Store credentials in the disposable keychain; do not persist them in the login keychain.
xcrun notarytool store-credentials okf-notary --keychain "$keychain" \
  --apple-id "$APPLE_ID" --team-id "$APPLE_TEAM_ID" \
  --password "$APPLE_APP_SPECIFIC_PASSWORD"
submit_result=0
xcrun notarytool submit "$work/submission.zip" --keychain-profile okf-notary \
  --keychain "$keychain" --wait --timeout 30m --output-format json \
  > "$work/result.json" || submit_result=$?
cat "$work/result.json"
status=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1])).get("status", ""))' "$work/result.json")
if [ "$submit_result" -ne 0 ] || [ "$status" != Accepted ]; then
  submission_id=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1])).get("id", ""))' "$work/result.json")
  if [ -n "$submission_id" ]; then
    xcrun notarytool log "$submission_id" --keychain-profile okf-notary --keychain "$keychain" || true
  fi
  echo "Notarization did not finish with Accepted status; refusing to release." >&2
  exit 1
fi
