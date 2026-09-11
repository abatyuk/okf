---
type: Skill
title: okf:init skill
description: Creates a valid OKF v0.2 bundle and scaffolds portable concepts or exact Attested Computations, with ontology treated as an optional tool sidecar.
trigger: start a new bundle from scratch
uses: [/commands/init, /commands/add, /commands/validate, /commands/computation-check]
---
# okf:init skill

Creates a valid empty bundle, distinguishes required, recommended, optional, and extension
metadata, and uses `references/` only for authorized local artifacts.
