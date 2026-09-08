//! `okf ontology list|show|add|update|remove`.
use std::str::FromStr;

use crate::cli::{BundleArgs, OntEditArgs, OntRemoveArgs, OntShowArgs};
use crate::output;
use okf_core::bundle::resolve::resolve_bundle;
use okf_core::error::{OkfError, Result};
use okf_core::ontology::edit::{
    add_concept_type, remove_concept_type, remove_field, remove_reference, save_ontology,
    set_field, set_reference,
};
use okf_core::ontology::field_types::resolve_field;
use okf_core::ontology::load::{ontology_path, try_load};
use okf_core::ontology::schema::{
    Cardinality, ConceptType, Field, Ontology, ReferenceRule, Target,
};
use serde_json::json;

/// The default schema version stamped into a freshly-created `ontology.yaml`.
const ONTOLOGY_VERSION: &str = "0.1";

/// `okf ontology list [bundle]`.
pub fn run_list(args: &BundleArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let ontology = require_ontology(&root)?;
    for (name, ct) in &ontology.concepts {
        if json {
            output::print_line(&concept_type_record(name, ct, &ontology))?;
        } else {
            let desc = ct.description.as_deref().unwrap_or("");
            println!("{name}\t{desc}");
        }
    }
    Ok(0)
}

/// `okf ontology show <name> [bundle]`.
pub fn run_show(args: &OntShowArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let ontology = require_ontology(&root)?;
    let ct = ontology
        .concepts
        .get(&args.name)
        .ok_or_else(|| OkfError::Usage(format!("no concept type {:?}", args.name)))?;
    if json {
        output::print_line(&concept_type_record(&args.name, ct, &ontology))?;
    } else {
        println!("# {}", args.name);
        if let Some(d) = &ct.description {
            println!("description: {d}");
        }
        if ct.attested {
            println!("attested: true");
        }
        if !ct.requires.is_empty() {
            println!("requires: {}", ct.requires.join(", "));
        }
        if !ct.fields.is_empty() {
            println!("fields:");
            for (k, f) in &ct.fields {
                let values = resolve_field(&ontology, f)
                    .ok()
                    .map(|r| r.ty.values)
                    .filter(|v| !v.is_empty())
                    .map(|v| format!(" [{}]", v.join("|")))
                    .unwrap_or_default();
                println!(
                    "  {k}: {}{}{}",
                    f.type_name,
                    values,
                    if f.required { " (required)" } else { "" }
                );
            }
        }
        if !ct.references.is_empty() {
            println!("references:");
            for (k, r) in &ct.references {
                println!(
                    "  {k}: {} [{}]",
                    r.target.types().join("|"),
                    r.cardinality.as_str()
                );
            }
        }
    }
    Ok(0)
}

/// `okf ontology add <name> [bundle] [--description] [--field ...] [--ref ...] [--attested]`.
pub fn run_add(args: &OntEditArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let mut ontology = load_or_default(&root)?;
    if !args.remove_field.is_empty() || !args.remove_reference.is_empty() {
        return Err(OkfError::Usage(
            "--remove-field/--remove-ref are only valid with `ontology update`".to_string(),
        ));
    }

    let mut ct = ConceptType {
        description: args.description.clone(),
        attested: args.attested,
        ..ConceptType::default()
    };
    for f in &args.field {
        let (key, field) = parse_field(f)?;
        ct.fields.insert(key, field);
    }
    for r in &args.reference {
        let (key, rule) = parse_reference(r)?;
        ct.references.insert(key, rule);
    }
    add_concept_type(&mut ontology, &args.name, ct)?;
    persist(&root, &ontology)?;
    report_change("add", &args.name, json)
}

/// `okf ontology update <name> [bundle] ...` — modify an existing concept type in place.
pub fn run_update(args: &OntEditArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let mut ontology = require_ontology(&root)?;
    if !ontology.concepts.contains_key(&args.name) {
        return Err(OkfError::Usage(format!(
            "concept type {:?} does not exist",
            args.name
        )));
    }
    if let Some(d) = &args.description {
        if let Some(ct) = ontology.concepts.get_mut(&args.name) {
            ct.description = Some(d.clone());
        }
    }
    if args.attested {
        if let Some(ct) = ontology.concepts.get_mut(&args.name) {
            ct.attested = true;
        }
    }
    for f in &args.field {
        let (key, field) = parse_field(f)?;
        set_field(&mut ontology, &args.name, &key, field)?;
    }
    for r in &args.reference {
        let (key, rule) = parse_reference(r)?;
        set_reference(&mut ontology, &args.name, &key, rule)?;
    }
    for key in &args.remove_field {
        remove_field(&mut ontology, &args.name, key)?;
    }
    for key in &args.remove_reference {
        remove_reference(&mut ontology, &args.name, key)?;
    }
    persist(&root, &ontology)?;
    report_change("update", &args.name, json)
}

/// `okf ontology remove <name> [bundle]`.
pub fn run_remove(args: &OntRemoveArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let mut ontology = require_ontology(&root)?;
    remove_concept_type(&mut ontology, &args.name)?;
    persist(&root, &ontology)?;
    report_change("remove", &args.name, json)
}

// ---- helpers ----

fn require_ontology(root: &std::path::Path) -> Result<Ontology> {
    try_load(root)?.ok_or_else(|| {
        OkfError::Environment(format!(
            "no ontology.yaml in bundle {} (run `okf ontology add ...` or `okf init`)",
            root.display()
        ))
    })
}

fn load_or_default(root: &std::path::Path) -> Result<Ontology> {
    Ok(try_load(root)?.unwrap_or_else(|| Ontology {
        okf_ontology: ONTOLOGY_VERSION.to_string(),
        ..Ontology::default()
    }))
}

fn persist(root: &std::path::Path, ontology: &Ontology) -> Result<()> {
    save_ontology(&ontology_path(root), ontology)
}

fn report_change(op: &str, name: &str, json: bool) -> Result<i32> {
    if json {
        output::print_line(
            &json!({"kind": "change", "op": format!("ontology-{op}"), "name": name}),
        )?;
    } else {
        println!("ontology {op}: {name}");
    }
    Ok(0)
}

/// Build an NDJSON record describing a concept type.
fn concept_type_record(name: &str, ct: &ConceptType, ontology: &Ontology) -> serde_json::Value {
    let fields: Vec<_> = ct
        .fields
        .iter()
        .map(|(k, f)| {
            let values = resolve_field(ontology, f).ok().map(|r| r.ty.values);
            json!({"key": k, "type": f.type_name, "required": f.required, "values": values})
        })
        .collect();
    let references: Vec<_> = ct
        .references
        .iter()
        .map(|(k, r)| json!({"key": k, "target": r.target.types(), "cardinality": r.cardinality.as_str()}))
        .collect();
    json!({
        "kind": "ontology_type",
        "name": name,
        "description": ct.description,
        "attested": ct.attested,
        "requires": ct.requires,
        "fields": fields,
        "references": references,
    })
}

/// Parse a `--field key:type[:required][:v1|v2|...]` spec.
fn parse_field(spec: &str) -> Result<(String, Field)> {
    let mut segs = spec.split(':');
    let key = segs
        .next()
        .filter(|k| !k.trim().is_empty())
        .ok_or_else(|| OkfError::Usage(format!("invalid --field {spec:?}: expected key:type")))?
        .trim()
        .to_string();
    let type_name = segs
        .next()
        .filter(|t| !t.trim().is_empty())
        .ok_or_else(|| OkfError::Usage(format!("invalid --field {spec:?}: missing type")))?
        .trim()
        .to_string();

    let mut field = Field {
        type_name,
        ..Field::default()
    };
    for seg in segs {
        let seg = seg.trim();
        if seg == "required" {
            field.required = true;
        } else if !seg.is_empty() {
            field.values = Some(seg.split('|').map(|v| v.trim().to_string()).collect());
        }
    }
    Ok((key, field))
}

/// Parse a `--ref key:Target[|Target2]:cardinality` spec.
fn parse_reference(spec: &str) -> Result<(String, ReferenceRule)> {
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.len() != 3 || parts[0].trim().is_empty() {
        return Err(OkfError::Usage(format!(
            "invalid --ref {spec:?}: expected key:Target[|Target2]:cardinality"
        )));
    }
    let key = parts[0].trim().to_string();
    let targets: Vec<String> = parts[1].split('|').map(|t| t.trim().to_string()).collect();
    let target = if targets.len() == 1 {
        Target::One(targets.into_iter().next().unwrap())
    } else {
        Target::Many(targets)
    };
    let cardinality = Cardinality::from_str(parts[2].trim())?;
    Ok((
        key,
        ReferenceRule {
            target,
            cardinality,
            extra: Default::default(),
        },
    ))
}
