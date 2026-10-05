# Evaluate OKF skill workflows

`cargo xtask skills` checks skill syntax, generated references, and executes the CLI workflows in
`crates/okf-cli/tests/skill_workflows.rs`. Those tests consume the real example files in skill
references and the scenario IDs in `skill-behaviors.json`. They check observable outputs, including
scan recovery, occurrence pairing, effective nested types, source identity/scope, and independent
expansion bounds, plus migration identity collisions, lossless structured conversion, and recovery
from a failed second-bundle write. Structured authoring scenarios also exercise dependent ontology
apply, dry-run nonpersistence, object-map creation, guarded list patches, and failure atomicity. They do not measure whether an agent chooses the right workflow.

For agent evaluation, use each case in `skill-behaviors.json` as follows:

1. Prepare the stated setup in an isolated temporary workspace. For file examples, materialize the
   fenced block immediately under each backticked filename heading. Supply the relevant skill and
   tools to the evaluating agent, the workspace, and only the scenario's `prompt`; keep `pass` and
   `fail` criteria for the evaluator. For nested-authoring, do not supply the completed sidecar as
   the starting artifact.
2. Let the agent complete the task under the prompt's scope. Record tool calls, commands, returned
   completeness/scope evidence, mutations, and final claims. Do not require an exact command order
   when another approach establishes the same evidence at comparable cost.
3. Judge the actual result against every pass criterion and any fail outcome. Inspect before/after
   file diffs for preservation. Count repeated scans and target lookups; distinguish output volume
   from examined documents. Correct answers without sufficient evidence do not pass.
4. Report deterministic CLI checks separately from agent trials. Record the model/runtime, skill
   version, scenario, result, and evidence. An unrun agent trial remains unrun; static phrase checks
   and command success cannot substitute for it.

Run the isolated executable cases alone with:

```sh
cargo test -p okf-cli --test skill_workflows -- --nocapture
```
