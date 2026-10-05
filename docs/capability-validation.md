# Multi-bundle and structured metadata validation

This audit exercises the implemented contracts in [multi-bundle capability](multi-bundle-capability.md)
and [structured metadata capability](structured-metadata-capability.md). Provisional examples are
interpreted using each document's implemented-interface section and the current CLI schema.
Tests use isolated temporary bundles and local Git repositories. No real remote content is fetched.

The matrices below map the specified scenario families to executable assertions, including negative
cases and deliberate limits. They are a finite contract regression suite, not proof over every
possible document, platform, or concurrent filesystem change. Agent decision quality is evaluated
separately by [the skill evaluation protocol](../xtask/skill-behavior-evaluation.md).

## Results

After the three corrections below, the default-feature workspace run passed **365 tests**, with
**0 failures and 0 ignored tests**. This includes **45 added capability tests** (36 CLI and 9 core),
many containing tables of positive and negative cases. The matrices map **69 scenario families**
to new and existing assertions. `cargo xtask skills` also passed all 10 skill checks, confirmed
49 generated documentation artifacts were current, and ran all six executable skill scenarios.
Formatting, diff whitespace checks, and the coverage report's relative links passed.

## Reproduce

```sh
cargo test --workspace --offline
cargo xtask skills
cargo fmt --all -- --check
```

The default-feature workspace suite covers both local capabilities. The optional `--all-features`
run was not completed because an uncached dependency download from `static.crates.io` was blocked
by the execution sandbox. The optional URL-source feature is not remote bundle acquisition and
is outside these two capabilities' implemented acquisition scope.

## Defects found and corrected

1. Human expansion with computed target fields such as `$id` panicked because its renderer assumed
   all cells contained `occurrences`. It now handles computed `value` cells too.
2. Standalone affected analysis treated `.md` links as literal concept IDs, missing their consumers.
   Seeds now use the graph's ordinary-link normalization, including fragments/query suffixes.
3. Affected analysis, stats, and document rendering loaded only root `ontology.yaml`, ignoring an
   explicitly configured sidecar. They now use effective bundle interpretation settings.

Regression tests exercise the failing inputs and verify the corrected outcomes. No capability
contract was weakened to make these cases pass.

## Test locations

| Key | Executable evidence |
| --- | --- |
| CLI | [New black-box capability scenarios](../crates/okf-cli/tests/capability_scenarios.rs) |
| CORE | [New structured boundary scenarios](../crates/okf-core/tests/capability_scenarios.rs) |
| INTEGRATION | [Existing catalog/query integration](../crates/okf-cli/tests/capability_integration.rs) |
| BOUNDS | [Existing expansion bounds](../crates/okf-cli/tests/structured_bounds.rs) |
| CATALOG | [Catalog unit tests](../crates/okf-core/src/bundle/catalog.rs) |
| CONTEXT | [Selection/configuration unit tests](../crates/okf-core/src/bundle/context.rs) |
| GRAPH | [Catalog graph and snapshot unit tests](../crates/okf-core/src/graph/catalog.rs) |
| STRUCTURE | [Structured contract tests](../crates/okf-core/tests/structured_contract.rs) and [integration tests](../crates/okf-core/tests/structured_integration.rs) |
| MOVES | [Unsupported structured mutation guards](../crates/okf-core/tests/structured_move.rs) |
| SKILLS | [Six executable skill workflows](../crates/okf-cli/tests/skill_workflows.rs) |

## Multi-bundle scenario matrix

| ID | Specified scenario and expected result | Evidence |
| --- | --- | --- |
| MB01 | Namespaced stable IDs; duplicate IDs, equal/nested roots rejected; unavailable roots remain unavailable | CATALOG: all three tests |
| MB02 | Canonicalized symlink roots cannot register the same directory twice | CLI: `catalog_canonicalization_rejects_symlink_aliases_of_registered_roots` |
| MB03 | Catalog relative to configuration; locations relative to catalog; explicit path/ID, env, containing cwd, default, sole entry, ambiguity | CLI: `selection_precedence_and_catalog_relative_paths`; CONTEXT: `typed_selection_and_cwd_precedence` |
| MB04 | Legacy path alias coexists only with equivalent typed path; ambiguous/empty typed selectors rejected | CONTEXT: `config_alias_accepts_only_equivalent_paths` |
| MB05 | File override paths relative to override file; environment overrides take precedence; effective/configured roots retained | CLI: `override_file_and_environment_precedence_are_reported`; CATALOG: `overrides_preserve_configured_root_and_validate_effective_overlap` |
| MB06 | Invalid catalog is a tool error; direct path access still permits conformant standalone consumption | CLI: `invalid_catalog_is_separate_from_standalone_conformance`; CONTEXT: `invalid_catalog_does_not_prohibit_explicit_standalone_consumption` |
| MB07 | Inspection takes no primary, scope, or revision selector; unsupported commands reject scope/revision | CLI: `catalog_rejects_selection_scope_and_revision_flags` |
| MB08 | Registration does not extend traversal; explicit additional scope; unexamined content not loaded | INTEGRATION: `catalog_registration_does_not_expand_default_query_scope`; GRAPH: `out_of_scope_content_is_not_loaded` |
| MB09 | Missing registrations, unavailable roots and examined scope distinguishable | CLI: `unavailable_scope_members_and_unknown_ids_are_distinct` |
| MB10 | Ordinary relative paths retain above-root traversal; qualified targets preserve bundle identity; unregistered external paths not guessed | GRAPH: `cross_bundle_paths_are_qualified_without_above_root_clamping` |
| MB11 | Standalone subdirectory remains its own boundary; conflicting overlapping scope rejected | GRAPH: `standalone_subdirectory_keeps_its_boundary_and_rejects_ambiguous_scope` |
| MB12 | Resource and bundle_ref must agree; known mismatches cannot redirect ordinary targets | GRAPH: `declaration_never_redirects_a_known_mismatch`; `external_sources_need_explicit_mapping_and_fragments_preserve_resource_meaning` |
| MB13 | Explicit URL correspondence is unchecked, not independently verified or fetched | CLI: `declared_url_mapping_is_unchecked_and_never_fetched` |
| MB14 | Source attribution, body occurrences, and repository-relative Git sources remain distinct | GRAPH: `catalog_keeps_repository_relative_git_concept_sources_and_body_occurrences`; SKILLS: resumable migration |
| MB15 | Label-only expectations unverified; full commits/refs/digests matched when established; invalid exact identifiers mismatched; missing revisions unavailable | CLI: `snapshots_cover_digest_label_commit_ref_missing_and_mismatch` |
| MB16 | Digests cover raw bytes, including line-ending changes; no canonical-text substitution | GRAPH: `raw_digest_matches_bytes_and_not_canonicalized_text` |
| MB17 | Historical result and mutable ordinary target coexist; changed live bytes retain mismatch and candidate | GRAPH: `local_git_history_remains_separate_from_live_mismatch` |
| MB18 | Body links do not inherit source snapshot expectations; returning a candidate does not mutate metadata | GRAPH: `snapshot_candidates_are_separate_and_body_links_do_not_inherit_expectations`; CLI: snapshot table and byte-preservation assertions |
| MB19 | Target resolution, snapshot satisfaction, and source fingerprint drift are independent | CLI: `source_drift_is_reported_separately_from_successful_target_and_snapshot` |
| MB20 | Graph, links, backlinks, resolve, affected, search, and list support local revisions and report examined versions | CLI: `every_revision_capable_command_reads_available_local_commits` |
| MB21 | Independent repositories may resolve the same ref to different commits; no cross-repository atomicity implied | CLI: `same_catalog_can_span_independent_git_repositories` |
| MB22 | In-repository interpretation uses historical settings; unavailable historical settings diagnosed; external ontology uses current bytes with digest | INTEGRATION: `historical_query_uses_committed_concepts_and_interpretation_settings`; CLI: both historical configuration/ontology tests |
| MB23 | Cycles terminate; rooted/depth-bounded graph and affected analysis report only examined consumers | GRAPH: `cross_bundle_cycles_are_bounded_and_report_only_examined_consumers`; existing graph/affected unit and CLI tests |
| MB24 | Local Git reads disable lazy fetching, including missing blobs in a partial clone | CLI: `historical_git_calls_disable_lazy_fetch_and_never_invoke_fetch`; `partial_clone_missing_blob_is_unavailable_without_invoking_remote_helper` (with positive acquisition control) |
| MB25 | Human graph artifacts preserve scope/reference diagnostics without corrupting the rendered artifact | [Catalog graph artifact test](../crates/okf-cli/tests/catalog_graph_artifact.rs) |
| MB26 | New cross-bundle warnings stay separate from legacy internal broken-link error severity | INTEGRATION: `scoped_lint_does_not_call_registered_cross_bundle_links_broken`; `catalog_lint_preserves_existing_internal_body_link_error_severity` |
| MB27 | Catalog supports local directories only and does not enable general cross-bundle artifact reads | CLI: `deferred_acquisition_and_cross_bundle_artifact_reads_are_not_enabled_by_catalogs`; existing artifact containment tests |

## Structured metadata scenario matrix

| ID | Specified scenario and expected result | Evidence |
| --- | --- | --- |
| SM01 | Each bundle uses its own settings/ontology; explicit sidecar readable; unnamed paths do not inherit named settings | CLI: `explicit_settings_do_not_leak_to_unregistered_paths`; INTEGRATION: `equal_type_spelling_does_not_imply_cross_bundle_type_equivalence` |
| SM02 | Configured ontology has the same effect as root ontology in readers; query/lint/view settings validated | CLI: `configured_sidecar_has_same_effect_on_affected_stats_and_rendering_as_root_ontology`; `query_and_lint_configuration_reject_invalid_limits_and_selectors` |
| SM03 | Selector grammar: explicit list traversal, bracket-quoted literal keys/escaping, case sensitivity; unsupported expression syntax rejected | CORE: `selector_subset_and_case_sensitive_explicit_traversal`; selector unit test |
| SM04 | Reusable nested definitions and inherited fields exposed; field maps replace, not merge | STRUCTURE: `inherited_field_map_replaces_and_closure_checks_replacement`; SKILLS: nested authoring |
| SM05 | Derived enum values and list item definitions replace inherited counterparts | CORE: `derived_enum_values_and_list_item_replace_inherited_declarations` |
| SM06 | Unknown named types, cycles, changing inherited base, conflicting bounds and malformed constraints rejected | CORE: `malformed_constraints_and_named_types_are_configuration_errors`; STRUCTURE: `named_composition_cycles_and_inherited_conflicts_are_configuration_errors` |
| SM07 | Every primitive, enum, pattern, integer/string/list bound, object key type and explicit closure has positive/negative checks | CORE: `every_primitive_and_constraint_has_positive_and_negative_boundaries` |
| SM08 | Unicode scalar lengths, Gregorian dates, explicit-zone datetimes, structural uniqueness ignoring map order | STRUCTURE: `recursive_checks_keep_scalar_types_unicode_lengths_and_structural_uniqueness`; `uri_authorities_percent_escapes_and_components_are_checked` |
| SM09 | Missing optional parents skipped; null, blank required strings, empty required lists and wrong containers diagnosed distinctly | STRUCTURE: `null_required_empty_and_container_failures_are_distinct` |
| SM10 | Unknown metadata stays open; unsupported constraints preserved and reported unenforced | CORE: `unknown_constraints_remain_visible_and_root_metadata_stays_open` |
| SM11 | Depth 64 and finding budget exhaustion explicitly incomplete; severity configurable; conformance remains independent | CORE: `recursive_value_depth_limit_reports_incomplete_checks`; STRUCTURE: `finding_budget_covers_required_references_and_relationships`; CLI: `lint_severity_and_finding_budget_preserve_conformance_separation` |
| SM12 | Published ontology and onboarding example execute successfully; target, kind, attributes and inverse paired per occurrence | CLI: `published_structured_example_lints_and_expands_paired_obligations`; STRUCTURE: `occurrence_pairing_self_edges_and_neighbors` |
| SM13 | Cardinality counts occurrences including duplicates/unresolved targets, not distinct neighbors | CORE: `reference_cardinality_counts_duplicate_and_unresolved_occurrences` |
| SM14 | Ambiguous anchors, contradictory mapping and missing reference rule rejected | CORE: `invalid_relationship_anchors_and_dynamic_vocabularies_are_rejected` |
| SM15 | Dynamic-kind target restrictions narrow declared types; exact qualified target types and concept ancestry respected | STRUCTURE: `concept_subtypes_narrow_relationship_targets`; CORE: `dynamic_kind_widening_and_vocabulary_disagreement_are_configuration_errors`; INTEGRATION: qualified-type test |
| SM16 | Semantic self-edges and distinct occurrences retained; legacy neighbor lists deduplicate | STRUCTURE: `occurrence_pairing_self_edges_and_neighbors`; CLI: `expansion_rule_qualification_and_occurrence_deduplication` |
| SM17 | Unknown kinds remain unresolved; missing, wrong-type, unavailable and out-of-scope references distinguished | CLI: `expansion_reports_missing_wrong_type_unknown_kind_and_byte_bounds`; BOUNDS: unavailable targets; GRAPH: selected-scope tests |
| SM18 | Backlinks retain inverse names/conditions; affected analysis includes nested references; reads do not mutate declarations | CLI: `nested_targets_participate_in_affected_analysis_without_mutation`; SKILLS: nested authoring and migration |
| SM19 | Root/subdirectory index coverage uses actual immediate-child Markdown links; deeper links/prose/images/code/remote links do not count | CLI: `index_coverage_uses_real_immediate_links_not_mentions_or_images`; STRUCTURE: `indexes_require_immediate_markdown_children_and_exclude_subtrees` |
| SM20 | Index link variants include reference style, extensionless, escaped paths, fragments, directories/indexes and duplicates | Same index tests as SM19 |
| SM21 | Missing indexes reported once; unreadable content errors; symlinks ignored; exclusions recompute ancestors but do not hide concepts from queries | CLI: index and unreadable-index tests; STRUCTURE: `index_coverage_does_not_follow_symlink_indexes` |
| SM22 | Hidden/Git-ignored concepts follow the physical walker; .git, artifacts and artifact-only directories excluded from concept coverage | [Walker unit test](../crates/okf-core/src/bundle/walk.rs); CLI/STRUCTURE index tests |
| SM23 | Projections preserve missing/null and literal keys; computed values do not overwrite authored identities | CLI: `projections_distinguish_missing_null_literal_keys_and_authored_identity`; INTEGRATION multi-bundle identity tests |
| SM24 | Human columns escape separators, visibly truncate long cells and show missing markers | CLI: `human_cells_escape_delimiters_and_mark_truncation_and_missing` |
| SM25 | Human views do not implicitly change machine output; explicit extended views opt into projections/expansion; no-expand suppresses configured expansion | CLI: `views_preserve_legacy_json_until_explicit_machine_opt_in` |
| SM26 | Computed target fields work in human expansion | CLI: `human_expansion_accepts_computed_target_fields` |
| SM27 | Expansion is selected, outbound one-hop, page-scoped; target payloads deduplicate by qualified identity while occurrences remain distinct | CLI: qualified expansion test and `expansion_follows_only_one_hop_and_only_the_selected_primary_page`; SKILLS: expansion/pagination workflow; BOUNDS: fragment/query-suffix test |
| SM28 | Per-hit edge, query target and serialized-byte bounds retain warnings/reasons; ranges enforced; truncated target not needlessly parsed | CLI: expansion limits tests; BOUNDS: `expansion_bounds_stop_unrequested_target_parsing` |
| SM29 | Cross-bundle rule names require qualification outside the primary; target metadata need not match primary filters | CLI: `expansion_rule_qualification_and_occurrence_deduplication`; INTEGRATION: `expansion_resolves_scoped_target_outside_primary_scan_prefix` |
| SM30 | Default 1,000-document budget; primary settings supply query-wide budget; explicit limit overrides; full-scan is invocation-only | SKILLS: incomplete inventory; INTEGRATION: both primary-budget tests; CLI: invalid-config tests |
| SM31 | Budget counts examined documents, not hits; exact boundary can be complete; empty incomplete result still warns and has unknown totals/no continuation | CLI: `scan_exact_boundary_is_complete_and_limits_never_hide_empty_scan_warning`; INTEGRATION: prefix parse-budget test |
| SM32 | Scan, facet presentation, result page and expansion completeness remain independent | CLI: facet-cap tests; SKILLS: expansion/pagination workflow |
| SM33 | Facets count matching documents before pagination, deduplicate repeated list values, omit missing/null/empty, retain empty strings and scalar types | INTEGRATION: `facets_count_scoped_matches_before_pagination_and_deduplicate_values`; CLI: facet-value test; STRUCTURE: membership test |
| SM34 | Facet buckets use descending counts then typed value order; nonscalar values produce diagnostics | STRUCTURE: `facet_numeric_ties_use_numeric_order_and_keep_scalar_types_distinct`; CLI: facet-value test |
| SM35 | Exactly 100 values allowed; 101 excluded; explicit facet request does not bypass guard; configured inclusion still capped at 1,000 output values | CLI: `facets_threshold_output_cap_and_explicit_inclusion_are_independent` |
| SM36 | Conditions AND; membership alternatives OR within one condition; not-in checks the whole list; empty differs from missing/null | CLI: `typed_filter_errors_and_literal_key_compatibility`; STRUCTURE: membership and missing-nested-branch tests |
| SM37 | Operands are typed nonnull scalars/nonempty scalar arrays; object/nested-array/type mismatches rejected; literal --field compatible | Same filter tests as SM36 |
| SM38 | Separate nested predicates can match different records; actual condition pairing must be inspected | SKILLS: cross-record conditions; migration preserves two distinct deadlines |
| SM39 | Offset requires positive limit; negative/zero/conflicting controls rejected; qualified global ordering; beyond-end has no continuation | CLI: `pagination_is_globally_ordered_and_beyond_end_has_no_continuation`; INTEGRATION: control rejection test |
| SM40 | Offset invocations rescan independently; page size does not reduce scan work or establish immutable working-tree sessions | SKILLS: expansion/pagination cost scenario; CLI exact-boundary tests |
| SM41 | Nested-reference rewrites/moves remain deferred and preflight refuses affected writes; unaffected moves still work | MOVES: all four tests |
| SM42 | Multi-bundle structured migration preserves provenance, types, opaque extensions and record pairing; interrupted rerun does not duplicate writes | SKILLS: `resumable-structured-migration` |

## Deliberate limits

Remote acquisition/cache management, self-contained export, global consumer inventories, atomic
cross-repository updates, inverse/transitive automatic expansion, correlated predicates, cursor
sessions, persistent query caches, live compliance evaluation, and computation execution are
explicitly deferred. This audit does not claim they work. Negative cases exercise the applicable
CLI boundaries; snapshots and resolution are never treated as proof of factual correctness,
publisher authenticity, target consent, policy fulfillment, or document verification.

Separate-bundle distribution requires the authored relative layout. Working-tree reads cannot
promise an atomic snapshot or stable pages under concurrent changes. These are limitations to
retain in answers, not test failures that an implementation can turn into guarantees.
