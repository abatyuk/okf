---
type: 'DesignDecision'
title: 'validate is conformance; lint is opinion'
description: 'validate enforces only OKF''s three hard rules and stays permissive about everything else.'
decision_status: 'decided'
affects:
  - /components/check
---
# validate is conformance; lint is opinion

[okf validate](../commands/validate.md) enforces only OKF's three hard rules and stays
permissive about everything else. [okf lint](../commands/lint.md) is where opinions live, with
error/warn/info [Severity](../types/Severity.md) levels, so conformance and taste never get
conflated in the [check module](../components/check.md).
