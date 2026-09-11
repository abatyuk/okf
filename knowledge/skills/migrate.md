---
type: Skill
title: okf:migrate skill
description: Faithfully migrates authored or legacy documents into OKF v0.2 with standard sources, claim footnotes, and preserved extensions.
trigger: convert existing docs into concepts
uses: [/commands/add, /commands/edit, /commands/validate, /commands/artifact-resolve]
---
# okf:migrate skill

Migrates documentation without inventing provenance, translates legacy fields only when their
semantics are known, and keeps verification separate from migration.
