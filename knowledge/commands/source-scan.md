---
type: Command
title: okf source-scan
description: Inventories every regular file in a source directory without parsing it as an OKF bundle.
group: check
mutates: false
implemented_by:
- /components/check
---
# okf source-scan

Maps a repository or directory for ingest workflows. Unlike [okf scan](scan.md), it includes non-Markdown source artifacts and performs no concept parsing.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<directory>` | positional | yes | Source directory to inventory; it need not be an OKF bundle |

Global `--json` requests NDJSON.

Output stream: `source-file`.
