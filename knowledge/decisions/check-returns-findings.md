---
type: 'DesignDecision'
title: 'Check returns findings; the CLI decides the exit code'
description: 'Core produces Findings with severities and never calls process::exit.'
status: 'decided'
affects:
  - /components/check
  - /components/okf-cli
---
# Check returns findings; the CLI decides the exit code

Core produces Findings with severities and never calls process::exit. The 0/1/2/3/4 mapping and the --fail-on threshold live in the CLI, keeping core reusable by other consumers.
