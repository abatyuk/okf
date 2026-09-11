---
type: Skill
title: okf:update skill
description: Separately triages lifecycle expiry, standard source evidence, fingerprint drift, and content changes, then reconciles concepts without overstating refresh.
trigger: resync concepts after source changes
uses: [/commands/stale, /commands/affected, /commands/diff, /commands/refresh, /commands/edit]
---
# okf:update skill

Refresh updates supported fingerprint extensions only; meaningful rewrites update generation
state and require a new document review.
