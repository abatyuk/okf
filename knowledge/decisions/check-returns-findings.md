---
type: 'DesignDecision'
title: 'Check returns findings; the CLI decides the exit code'
description: 'Core produces Findings with severities and never calls process::exit.'
decision_status: 'decided'
affects:
  - /components/check
  - /components/okf-cli
---
# Check returns findings; the CLI decides the exit code

The [check module](../components/check.md) produces [Finding](../types/Finding.md) values with
[Severity](../types/Severity.md) levels and never calls `process::exit`. The 0/1/2/3/4 mapping
and the `--fail-on` threshold live in [okf-cli](../components/okf-cli.md), keeping core reusable
by other consumers.
