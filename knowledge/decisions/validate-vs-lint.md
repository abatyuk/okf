---
type: 'DesignDecision'
title: 'validate is conformance; lint is opinion'
description: 'validate enforces only OKF''s three hard rules and stays permissive about everything else.'
decision_status: 'decided'
affects:
  - /components/check
---
# validate is conformance; lint is opinion

validate enforces only OKF's three hard rules and stays permissive about everything else. lint is where opinions live, with error/warn/info severities, so conformance and taste never get conflated.
