//! Integration tests for the mutate layer: init / add / edit / mv (link rewrite!) / rm-guard /
//! verify / refresh. Each test runs in a self-cleaning temp bundle. Network/git effects use the
//! in-memory `FakeFs`/`FakeGit` ports; timestamps use `FixedClock`.
use std::path::{Path, PathBuf};

use okf_core::fingerprint::Engine;
use okf_core::model::concept::ConceptId;
use okf_core::model::source::{parse_sources, Source, SourceKind};
use okf_core::mutate::{add, edit, init, mv, refresh, rm, verify};
use okf_core::ontology::load::parse_ontology;
use okf_core::ontology::schema::Ontology;
use okf_core::parse::parse_concept;
use okf_core::ports::clock::FixedClock;
use okf_core::ports::fs::FakeFs;
use okf_core::ports::git::FakeGit;

/// A temp bundle directory that removes itself on drop (even on test panic).
struct Tmp(PathBuf);
impl Tmp {
    fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn temp_bundle() -> Tmp {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::SeqCst);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("okf-mutate-{}-{nanos}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    Tmp(dir)
}

fn write_file(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn read_file(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap()
}

fn sample_ontology() -> Ontology {
    parse_ontology(
        "okf_ontology: \"0.1\"\n\
         concepts:\n\
        \x20 Policy:\n\
        \x20   requires: [description]\n\
        \x20   fields:\n\
        \x20     owner: { type: string, required: true }\n\
        \x20     review_cycle: { type: enum, values: [monthly, quarterly, annual] }\n\
        \x20   references:\n\
        \x20     computations: { target: Computation, cardinality: 1..n }\n\
        \x20 Computation:\n\
        \x20   attested: true\n\
        \x20   fields:\n\
        \x20     runtime: { type: enum, values: [bigquery, dbt, python], required: true }\n",
    )
    .unwrap()
}

// ---------------------------------------------------------------------------- init

#[test]
fn init_scaffolds_index_and_ontology_and_is_idempotent() {
    let tmp = temp_bundle();
    let root = tmp.path().join("newbundle");

    let res = init::init(&root, &init::InitOptions::default()).unwrap();
    assert!(res.created_dir);
    assert!(res.index_path.is_some());
    assert!(res.ontology_path.is_some());
    assert!(root.join("index.md").is_file());
    assert!(root.join("ontology.yaml").is_file());
    assert!(read_file(&root, "index.md").starts_with("---\nokf_version: \"0.2\"\n---\n"));
    assert!(read_file(&root, "ontology.yaml").contains("okf_ontology"));

    // Re-init never clobbers existing structural files.
    std::fs::write(root.join("index.md"), "# custom\n").unwrap();
    let res2 = init::init(&root, &init::InitOptions::default()).unwrap();
    assert!(!res2.created_dir);
    assert!(res2.index_path.is_none(), "existing index.md preserved");
    assert_eq!(read_file(&root, "index.md"), "# custom\n");
}

// ---------------------------------------------------------------------------- add

#[test]
fn add_scaffolds_from_ontology_required_fields() {
    let tmp = temp_bundle();
    let root = tmp.path();
    let ont = sample_ontology();

    let res = add::add(
        root,
        "policies/travel",
        Some(&ont),
        &add::AddOptions {
            concept_type: Some("Policy".to_string()),
            title: Some("Travel policy".to_string()),
            description: Some("Rules for travel.".to_string()),
            attested: false,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(res.concept_type, "Policy");
    assert_eq!(res.id, ConceptId::from_relative("policies/travel"));

    let c = parse_concept(res.id.clone(), &read_file(root, "policies/travel.md")).unwrap();
    let keys: Vec<&str> = c.frontmatter.map.keys().map(String::as_str).collect();
    // type/title/description first, then the required custom field + required reference key.
    assert_eq!(
        keys,
        vec!["type", "title", "description", "owner", "computations"]
    );
    assert_eq!(c.title(), Some("Travel policy"));
    assert_eq!(c.frontmatter.get_str("owner"), Some(""));
    assert!(c.frontmatter.get("computations").unwrap().is_sequence());

    // Refuses to overwrite an existing file.
    let again = add::add(
        root,
        "policies/travel",
        Some(&ont),
        &add::AddOptions {
            concept_type: Some("Policy".to_string()),
            ..Default::default()
        },
    );
    assert!(again.is_err());
}

#[test]
fn add_attested_defaults_type_and_scaffolds_block() {
    let tmp = temp_bundle();
    let root = tmp.path();
    let ont = sample_ontology();

    let res = add::add(
        root,
        "computations/mileage_calc",
        Some(&ont),
        &add::AddOptions {
            concept_type: None,
            title: Some("Mileage calc".to_string()),
            description: None,
            attested: true,
            ..Default::default()
        },
    )
    .unwrap();
    // No --type + --attested resolves to the ontology's attested concept type.
    assert_eq!(res.concept_type, "Computation");
    assert!(res.attested);

    let c = parse_concept(
        res.id.clone(),
        &read_file(root, "computations/mileage_calc.md"),
    )
    .unwrap();
    // Required enums stay unset rather than being silently guessed.
    assert_eq!(c.frontmatter.get_str("runtime"), None);
    for key in ["computation", "executor", "attester"] {
        assert!(c.frontmatter.get(key).is_some(), "missing {key}");
    }
}

// ---------------------------------------------------------------------------- edit

#[test]
fn edit_sets_fields_losslessly_preserving_order_and_unknown_keys() {
    let tmp = temp_bundle();
    let root = tmp.path();
    write_file(
        root,
        "notes/n.md",
        "---\ntype: Note\ntitle: Old\ncustom_key: keep-me\nversion: 3\n---\nBody stays.\n",
    );

    let res = edit::edit(
        root,
        "notes/n",
        &edit::EditSpec {
            sets: vec![
                ("title".to_string(), "New title".to_string()),
                ("count".to_string(), "42".to_string()),
                ("draft".to_string(), "true".to_string()),
            ],
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(res.changes.len(), 3);

    let c = parse_concept(res.id.clone(), &read_file(root, "notes/n.md")).unwrap();
    let keys: Vec<&str> = c.frontmatter.map.keys().map(String::as_str).collect();
    // Existing keys keep position; new keys appended in order.
    assert_eq!(
        keys,
        vec!["type", "title", "custom_key", "version", "count", "draft"]
    );
    assert_eq!(c.title(), Some("New title"));
    assert_eq!(c.frontmatter.get_str("custom_key"), Some("keep-me"));
    assert_eq!(c.frontmatter.get("count").unwrap().as_i64(), Some(42));
    assert_eq!(c.frontmatter.get("draft").unwrap().as_bool(), Some(true));
    assert_eq!(c.body, "Body stays.\n");
}

#[test]
fn edit_unset_add_remove_manage_fields_and_lists() {
    let tmp = temp_bundle();
    let root = tmp.path();
    write_file(
        root,
        "c/x.md",
        "---\ntype: Component\ntitle: X\nlayer: core\ndeps:\n- /a\n- /b\n---\nbody\n",
    );

    let res = edit::edit(
        root,
        "c/x",
        &edit::EditSpec {
            unsets: vec!["layer".to_string()],
            adds: vec![
                ("deps".to_string(), "/c".to_string()),
                ("deps".to_string(), "/a".to_string()), // idempotent: already present
                ("tags".to_string(), "core".to_string()), // creates a new list
            ],
            removes: vec![("deps".to_string(), "/b".to_string())],
            ..Default::default()
        },
    )
    .unwrap();
    assert!(matches!(
        res.changes[0],
        edit::EditChange::Unset { existed: true, .. }
    ));

    let c = parse_concept(res.id.clone(), &read_file(root, "c/x.md")).unwrap();
    // layer removed; remaining keys keep order; new `tags` appended.
    let keys: Vec<&str> = c.frontmatter.map.keys().map(String::as_str).collect();
    assert_eq!(keys, vec!["type", "title", "deps", "tags"]);
    let deps: Vec<&str> = c
        .frontmatter
        .get("deps")
        .unwrap()
        .as_sequence()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(deps, vec!["/a", "/c"]); // /b removed, /c added, /a not duplicated
    assert_eq!(c.frontmatter.get_tags(), vec!["core"]);
    assert_eq!(c.body, "body\n");
}

#[test]
fn edit_removes_sources_by_resource_and_optional_kind() {
    let tmp = temp_bundle();
    let root = tmp.path();
    write_file(
        root,
        "c/sources.md",
        "---\ntype: Component\nsources:\n- resource: shared\n  kind: file\n  fingerprint: {content_sha256: old}\n- resource: shared\n  kind: url\n- resource: remove-all\n  kind: file\n- malformed-entry\n---\nbody\n",
    );

    let res = edit::edit(
        root,
        "c/sources",
        &edit::EditSpec {
            remove_sources: vec![
                edit::SourceSelector {
                    resource: "shared".to_string(),
                    kind: Some(SourceKind::File),
                },
                edit::SourceSelector {
                    resource: "remove-all".to_string(),
                    kind: None,
                },
            ],
            ..Default::default()
        },
    )
    .unwrap();

    assert!(matches!(
        &res.changes[0],
        edit::EditChange::Remove { key, removed: 1 } if key == "sources"
    ));
    assert!(matches!(
        &res.changes[1],
        edit::EditChange::Remove { key, removed: 1 } if key == "sources"
    ));
    let after = read_file(root, "c/sources.md");
    let concept = parse_concept(res.id, &after).unwrap();
    let sources = concept
        .frontmatter
        .get("sources")
        .unwrap()
        .as_sequence()
        .unwrap();
    assert_eq!(sources.len(), 2);
    assert_eq!(
        SourceKind::Url,
        Source::from_value(&sources[0]).unwrap().kind
    );
    assert_eq!(sources[1].as_str(), Some("malformed-entry"));
}

#[test]
fn edit_add_to_scalar_field_is_a_usage_error() {
    let tmp = temp_bundle();
    let root = tmp.path();
    write_file(root, "c/y.md", "---\ntype: Component\ntitle: Y\n---\nb\n");
    let err = edit::edit(
        root,
        "c/y",
        &edit::EditSpec {
            adds: vec![("title".to_string(), "z".to_string())],
            ..Default::default()
        },
    );
    assert!(err.is_err());
}

#[test]
fn edit_body_and_section_ops_preserve_frontmatter() {
    let tmp = temp_bundle();
    let root = tmp.path();
    write_file(
        root,
        "c/z.md",
        "---\ntype: Component\ntitle: Z\ncustom: keep\n---\n# Z\n\nintro\n\n## Rates\n\nold\n\n## Next\n\ntail\n",
    );

    // Replace a section, append to the body, all in one pass.
    edit::edit(
        root,
        "c/z",
        &edit::EditSpec {
            set_sections: vec![("rates".to_string(), "fresh content".to_string())],
            append_body: Some("appended tail".to_string()),
            ..Default::default()
        },
    )
    .unwrap();

    let c = parse_concept(ConceptId::from_relative("c/z"), &read_file(root, "c/z.md")).unwrap();
    // Frontmatter untouched (including the unknown key and its order).
    let keys: Vec<&str> = c.frontmatter.map.keys().map(String::as_str).collect();
    assert_eq!(keys, vec!["type", "title", "custom"]);
    assert!(c.body.contains("## Rates\n\nfresh content\n\n## Next"));
    assert!(!c.body.contains("old"));
    assert!(c.body.trim_end().ends_with("appended tail"));

    // Now remove the section entirely.
    edit::edit(
        root,
        "c/z",
        &edit::EditSpec {
            remove_sections: vec!["Rates".to_string()],
            ..Default::default()
        },
    )
    .unwrap();
    let c2 = parse_concept(ConceptId::from_relative("c/z"), &read_file(root, "c/z.md")).unwrap();
    assert!(!c2.body.contains("## Rates"));
    assert!(c2.body.contains("## Next"));
}

#[test]
fn edit_invalidates_verification_and_trust_tier() {
    let tmp = temp_bundle();
    let root = tmp.path();
    write_file(
        root,
        "policies/p.md",
        "---\ntype: Policy\ntitle: Old\nverified:\n- by: human:reviewer\n  at: 2026-09-01\n- by: process:ci\n  at: 2026-09-02\n---\n# Policy\n",
    );

    let res = edit::edit(
        root,
        "policies/p",
        &edit::EditSpec {
            sets: vec![("title".to_string(), "New".to_string())],
            ..Default::default()
        },
    )
    .unwrap();

    assert!(matches!(
        res.changes.last(),
        Some(edit::EditChange::InvalidateVerification { removed: 2 })
    ));
    let concept =
        parse_concept(res.id, &read_file(root, "policies/p.md")).expect("edited concept parses");
    assert!(concept.frontmatter.get("verified").is_none());
    assert_eq!(
        concept.trust_tier(),
        okf_core::model::trust::TrustTier::Unverified
    );
}

#[test]
fn edit_with_no_operations_errors() {
    let tmp = temp_bundle();
    let root = tmp.path();
    write_file(root, "c/e.md", "---\ntype: Component\ntitle: E\n---\nb\n");
    assert!(edit::edit(root, "c/e", &edit::EditSpec::default()).is_err());
}

// ---------------------------------------------------------------------------- mv

#[test]
fn mv_rewrites_every_inbound_link_and_rebases_moved_relative_links() {
    let tmp = temp_bundle();
    let root = tmp.path();
    write_file(root, "ontology.yaml", "okf_ontology: '0.1'\nconcepts:\n  Table:\n    references:\n      see: {target: Metric, cardinality: 0..n}\n      rel: {target: Table, cardinality: 0..n}\n  Policy:\n    references:\n      refs: {target: Table, cardinality: 0..n}\n  Metric:\n    references:\n      inputs: {target: Table, cardinality: 0..n}\n  T: {}\n");

    // The concept being moved: has an outbound RELATIVE link that must be rebased.
    write_file(
        root,
        "tables/customers.md",
        "---\ntype: Table\ntitle: Customers\nsee:\n- ../metrics/revenue\n---\n# Customers\n\nRelated [rev](../metrics/revenue.md).\n",
    );
    // Referrer via bundle-relative link (frontmatter + body).
    write_file(
        root,
        "policies/travel.md",
        "---\ntype: Policy\nrefs:\n- /tables/customers\n---\nSee [c](/tables/customers.md).\n",
    );
    // Referrer via relative link `../tables/...`.
    write_file(
        root,
        "metrics/revenue.md",
        "---\ntype: Metric\ninputs:\n- ../tables/customers.md\n---\nUses [c](../tables/customers.md).\n",
    );
    // Referrer via sibling relative link `./customers`.
    write_file(
        root,
        "tables/orders.md",
        "---\ntype: Table\nrel:\n- ./customers\n---\nLink [c](./customers.md).\n",
    );

    let res = mv::mv(root, "tables/customers", "warehouse/east/customers").unwrap();
    let rewritten: Vec<&str> = res.rewritten.iter().map(|c| c.0.as_str()).collect();
    assert_eq!(
        rewritten,
        vec!["/metrics/revenue", "/policies/travel", "/tables/orders"]
    );

    // Old file gone, new file present.
    assert!(!root.join("tables/customers.md").exists());
    assert!(root.join("warehouse/east/customers.md").is_file());

    // Bundle-relative referrer: style preserved, id updated.
    let travel = read_file(root, "policies/travel.md");
    assert!(travel.contains("- /warehouse/east/customers\n"), "{travel}");
    assert!(
        travel.contains("[c](/warehouse/east/customers.md)"),
        "{travel}"
    );

    // Relative referrer recomputed from its own directory.
    let revenue = read_file(root, "metrics/revenue.md");
    assert!(
        revenue.contains("- ../warehouse/east/customers.md\n"),
        "{revenue}"
    );
    assert!(
        revenue.contains("[c](../warehouse/east/customers.md)"),
        "{revenue}"
    );

    // Sibling relative referrer recomputed.
    let orders = read_file(root, "tables/orders.md");
    assert!(
        orders.contains("- ../warehouse/east/customers\n"),
        "{orders}"
    );
    assert!(
        orders.contains("[c](../warehouse/east/customers.md)"),
        "{orders}"
    );

    // Moved file's OWN relative links rebased to the new depth.
    let moved = read_file(root, "warehouse/east/customers.md");
    assert!(moved.contains("- ../../metrics/revenue\n"), "{moved}");
    assert!(moved.contains("[rev](../../metrics/revenue.md)"), "{moved}");

    // Refuses when target already exists.
    write_file(root, "x/y.md", "---\ntype: T\n---\n");
    assert!(mv::mv(root, "policies/travel", "x/y").is_err());
}

// ---------------------------------------------------------------------------- rm

#[test]
fn rm_guards_dangling_backlinks_unless_forced() {
    let tmp = temp_bundle();
    let root = tmp.path();
    write_file(root, "ontology.yaml", "okf_ontology: '0.1'\nconcepts:\n  Table: {}\n  Policy:\n    references:\n      refs: {target: Table, cardinality: 0..n}\n");
    write_file(root, "tables/customers.md", "---\ntype: Table\n---\n# C\n");
    write_file(
        root,
        "policies/travel.md",
        "---\ntype: Policy\nrefs:\n- /tables/customers\n---\n",
    );

    // Refuses while a backlink exists.
    let refused = rm::rm(root, "tables/customers", false);
    assert!(refused.is_err());
    assert!(root.join("tables/customers.md").exists());

    // Force removes and reports the now-dangling referrer.
    let forced = rm::rm(root, "tables/customers", true).unwrap();
    assert!(forced.removed);
    let dangling: Vec<&str> = forced
        .dangling_referrers
        .iter()
        .map(|c| c.0.as_str())
        .collect();
    assert_eq!(dangling, vec!["/policies/travel"]);
    assert!(!root.join("tables/customers.md").exists());

    // An unreferenced concept removes without force.
    let plain = rm::rm(root, "policies/travel", false).unwrap();
    assert!(plain.removed);
    assert!(plain.dangling_referrers.is_empty());
}

// ---------------------------------------------------------------------------- verify

#[test]
fn verify_appends_entry_and_promotes_bare_mapping_to_list() {
    let tmp = temp_bundle();
    let root = tmp.path();
    let clock = FixedClock("2026-09-07T00:00:00Z".to_string());

    // No prior `verified`: a one-element list is created.
    write_file(root, "a.md", "---\ntype: T\ntitle: A\n---\n# A\n");
    verify::verify(root, "a", "human:andrey", &clock).unwrap();
    let a = parse_concept(ConceptId::from_relative("a"), &read_file(root, "a.md")).unwrap();
    let seq = a
        .frontmatter
        .get("verified")
        .unwrap()
        .as_sequence()
        .unwrap();
    assert_eq!(seq.len(), 1);
    assert_eq!(a.trust_tier().as_str(), "human-reviewed");

    // A bare mapping is promoted to a two-element list, keeping the original entry.
    write_file(
        root,
        "b.md",
        "---\ntype: T\nverified:\n  by: process:ci\n  at: 2026-01-01T00:00:00Z\n---\n",
    );
    verify::verify(root, "b", "human:andrey", &clock).unwrap();
    let b = parse_concept(ConceptId::from_relative("b"), &read_file(root, "b.md")).unwrap();
    let seq = b
        .frontmatter
        .get("verified")
        .unwrap()
        .as_sequence()
        .unwrap();
    assert_eq!(seq.len(), 2, "bare mapping became a 2-element list");
    assert_eq!(
        seq[0].get("by").and_then(|v| v.as_str()),
        Some("process:ci")
    );
    assert_eq!(
        seq[1].get("by").and_then(|v| v.as_str()),
        Some("human:andrey")
    );
    assert_eq!(
        seq[1].get("at").and_then(|v| v.as_str()),
        Some("2026-09-07T00:00:00Z")
    );
}

// ---------------------------------------------------------------------------- refresh

#[test]
fn refresh_recomputes_fingerprints_preserving_extras_and_updating_last_modified() {
    let tmp = temp_bundle();
    let root = tmp.path();
    write_file(
        root,
        "computations/mileage.md",
        "---\n\
         type: Computation\n\
         title: Mileage\n\
         last_modified: 2020-01-01T00:00:00Z\n\
         sources:\n\
         - resource: src/x.py\n\
        \x20 kind: git-path\n\
        \x20 fingerprint:\n\
        \x20   blob_sha: oldsha\n\
        \x20 id: src-1\n\
        \x20 author: alice\n\
         - resource: weird\n\
        \x20 kind: sql-query\n\
         ---\n# Mileage\n",
    );

    let fs = FakeFs::new();
    let git = FakeGit::new().with_hash_object(root.join("src/x.py"), "newsha");
    let engine = Engine::new(root, &fs, &git, None);
    let clock = FixedClock("2026-09-07T12:00:00Z".to_string());

    let res = refresh::refresh(root, "computations/mileage", &engine, &clock).unwrap();
    assert_eq!(res.updated, vec!["src/x.py".to_string()]);
    assert_eq!(res.skipped.len(), 1, "unknown kind is skipped, not fatal");
    assert_eq!(res.skipped[0].resource, "weird");
    assert_eq!(res.last_modified, "2026-09-07T12:00:00Z");

    let c = parse_concept(res.id.clone(), &read_file(root, "computations/mileage.md")).unwrap();
    assert_eq!(
        c.frontmatter.get_str("last_modified"),
        Some("2026-09-07T12:00:00Z")
    );
    let sources = parse_sources(c.frontmatter.get("sources").unwrap());
    assert_eq!(sources[0].fingerprint.get("blob_sha"), Some("newsha"));
    // Extra keys (id/author) preserved verbatim.
    let extras: Vec<&str> = sources[0].extra.keys().map(String::as_str).collect();
    assert_eq!(extras, vec!["id", "author"]);
    // The skipped source is untouched.
    assert_eq!(sources[1].resource, "weird");
}

#[test]
fn refresh_preserves_and_reports_malformed_source_entries() {
    let tmp = temp_bundle();
    let root = tmp.path();
    write_file(
        root,
        "x.md",
        "---\ntype: T\nsources:\n- accidentally-a-string\n- resource: ''\n  kind: file\n---\n",
    );
    let fs = FakeFs::new();
    let git = FakeGit::new();
    let engine = Engine::new(root, &fs, &git, None);
    let clock = FixedClock("2026-09-08T00:00:00Z".to_string());
    let result = refresh::refresh(root, "x", &engine, &clock).unwrap();
    assert_eq!(result.skipped.len(), 2);
    let after = read_file(root, "x.md");
    assert!(after.contains("- accidentally-a-string"), "{after}");
    assert!(after.contains("resource: ''"), "{after}");
}
