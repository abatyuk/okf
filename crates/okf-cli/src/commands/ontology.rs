//! `okf ontology list|show|add|update|remove`.
use std::str::FromStr;

use crate::cli::{
    BundleArgs, OntApplyArgs, OntEditArgs, OntFieldTypeCmd, OntRemoveArgs, OntShowArgs,
};
use crate::output;
use crate::structured::{parse_yaml, preview_diff, InputReader};
use okf_core::bundle::resolve::resolve_bundle;
use okf_core::error::{OkfError, Result};
use okf_core::ontology::edit::{
    add_concept_type, apply_changes, merge_concept_type, remove_concept_type, remove_field,
    remove_reference, remove_relationship, render_ontology, save_ontology,
    validate_field_type_name,
};
use okf_core::ontology::field_types::resolve_field;
use okf_core::ontology::load::ontology_path;
use okf_core::ontology::schema::{
    Cardinality, ConceptType, Field, Ontology, ReferenceRule, Target,
};
use serde_json::json;
use serde_yaml::{Mapping, Value};
use std::collections::HashSet;

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
        println!(
            "effective fields:\n{}",
            serde_yaml::to_string(
                &ct.fields
                    .iter()
                    .map(|(k, f)| (k, resolve_field(&ontology, f).ok()))
                    .collect::<std::collections::BTreeMap<_, _>>()
            )
            .unwrap_or_default()
        );
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

/// Add a concept type with shorthand or structured declarations.
pub fn run_add(args: &OntEditArgs, json: bool) -> Result<i32> {
    if !args.remove_field.is_empty()
        || !args.remove_reference.is_empty()
        || !args.remove_relationship.is_empty()
    {
        return Err(OkfError::Usage(
            "removal flags are only valid with `ontology update`".into(),
        ));
    }
    let root = resolve_bundle(args.bundle.as_deref())?;
    let mut ontology = load_or_default(&root)?;
    let ct = edit_definition(&ConceptType::default(), args)?;
    add_concept_type(&mut ontology, &args.name, ct)?;
    finish(&root, &ontology, "add", &args.name, args.dry_run, json)
}

/// Modify named declarations while preserving all omitted entries.
pub fn run_update(args: &OntEditArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let mut ontology = require_ontology(&root)?;
    let existing = ontology
        .concepts
        .get(&args.name)
        .ok_or_else(|| OkfError::Usage(format!("concept type {:?} does not exist", args.name)))?;
    let ct = edit_definition(existing, args)?;
    ontology.concepts.insert(args.name.clone(), ct);
    for key in &args.remove_field {
        remove_field(&mut ontology, &args.name, key)?;
    }
    for key in &args.remove_reference {
        remove_reference(&mut ontology, &args.name, key)?;
    }
    for key in &args.remove_relationship {
        remove_relationship(&mut ontology, &args.name, key)?;
    }
    finish(&root, &ontology, "update", &args.name, args.dry_run, json)
}

pub fn run_remove(args: &OntRemoveArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let mut ontology = require_ontology(&root)?;
    remove_concept_type(&mut ontology, &args.name)?;
    finish(&root, &ontology, "remove", &args.name, args.dry_run, json)
}

pub fn run_field_type(cmd: &OntFieldTypeCmd, json: bool) -> Result<i32> {
    let (name, bundle, dry_run, op, input) = match cmd {
        OntFieldTypeCmd::Add(a) => (
            &a.name,
            &a.bundle,
            a.dry_run,
            "field-type-add",
            Some(&a.from),
        ),
        OntFieldTypeCmd::Update(a) => (
            &a.name,
            &a.bundle,
            a.dry_run,
            "field-type-update",
            Some(&a.from),
        ),
        OntFieldTypeCmd::Remove(a) => (&a.name, &a.bundle, a.dry_run, "field-type-remove", None),
    };
    validate_field_type_name(name)?;
    let root = resolve_bundle(bundle.as_deref())?;
    let mut ontology = if matches!(cmd, OntFieldTypeCmd::Add(_)) {
        load_or_default(&root)?
    } else {
        require_ontology(&root)?
    };
    let exists = ontology.field_types.contains_key(name);
    if matches!(cmd, OntFieldTypeCmd::Add(_)) && exists {
        return Err(OkfError::Usage(format!(
            "field type {name:?} already exists"
        )));
    }
    if !matches!(cmd, OntFieldTypeCmd::Add(_)) && !exists {
        return Err(OkfError::Usage(format!(
            "field type {name:?} does not exist"
        )));
    }
    if let Some(input) = input {
        let value = read_definition(&mut InputReader::default(), input)?;
        require_mapping(&value, "field-type definition")?;
        let definition = serde_yaml::from_value(value)
            .map_err(|e| OkfError::Usage(format!("invalid field-type definition: {e}")))?;
        ontology.field_types.insert(name.clone(), definition);
    } else {
        ontology.field_types.shift_remove(name);
    }
    finish(&root, &ontology, op, name, dry_run, json)
}

pub fn run_apply(args: &OntApplyArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let mut ontology = load_or_default(&root)?;
    let changes = read_definition(&mut InputReader::default(), &args.from)?;
    apply_changes(&mut ontology, &changes)?;
    finish(&root, &ontology, "apply", "ontology", args.dry_run, json)
}

fn edit_definition(existing: &ConceptType, args: &OntEditArgs) -> Result<ConceptType> {
    if args.name.trim().is_empty() {
        return Err(OkfError::Usage("concept-type name must be nonempty".into()));
    }
    let stdin_inputs = usize::from(args.from.as_deref() == Some("-"))
        + args
            .field_yaml
            .iter()
            .chain(&args.ref_yaml)
            .chain(&args.relationship_yaml)
            .filter(|spec| spec.split_once('=').is_some_and(|(_, input)| input == "-"))
            .count();
    if stdin_inputs > 1 {
        return Err(OkfError::Usage(
            "only one structured input may consume stdin".into(),
        ));
    }
    let mut reader = InputReader::default();
    let mut supplied = match &args.from {
        Some(input) => {
            require_mapping(&read_definition(&mut reader, input)?, "concept definition")?.clone()
        }
        None => Mapping::new(),
    };
    let mut touched = HashSet::new();
    for section in ["fields", "references", "relationships"] {
        if let Some(value) = supplied.get(Value::String(section.into())) {
            for key in require_mapping(value, section)?.keys() {
                claim(&mut touched, section, key.as_str().unwrap_or(""))?;
            }
        }
    }
    if let Some(description) = &args.description {
        if supplied.contains_key(Value::String("description".into())) {
            return Err(OkfError::Usage(
                "overlapping --from and --description".into(),
            ));
        }
        supplied.insert(
            Value::String("description".into()),
            Value::String(description.clone()),
        );
    }
    if args.attested {
        if supplied.contains_key(Value::String("attested".into())) {
            return Err(OkfError::Usage("overlapping --from and --attested".into()));
        }
        supplied.insert(Value::String("attested".into()), Value::Bool(true));
    }
    for spec in &args.field {
        let (key, declaration) = parse_field(spec)?;
        put_declaration(
            &mut supplied,
            &mut touched,
            "fields",
            &key,
            serde_yaml::to_value(declaration).map_err(|e| OkfError::Yaml(e.to_string()))?,
        )?;
    }
    for spec in &args.reference {
        let (key, declaration) = parse_reference(spec)?;
        put_declaration(
            &mut supplied,
            &mut touched,
            "references",
            &key,
            serde_yaml::to_value(declaration).map_err(|e| OkfError::Yaml(e.to_string()))?,
        )?;
    }
    for (section, specs) in [
        ("fields", &args.field_yaml),
        ("references", &args.ref_yaml),
        ("relationships", &args.relationship_yaml),
    ] {
        for spec in specs {
            let (key, input) = spec
                .split_once('=')
                .ok_or_else(|| OkfError::Usage(format!("expected key=YAML for {section}")))?;
            let value = parse_yaml(&reader.read(input)?)?;
            require_mapping(&value, section)?;
            put_declaration(&mut supplied, &mut touched, section, key, value)?;
        }
    }
    for (section, keys) in [
        ("fields", &args.remove_field),
        ("references", &args.remove_reference),
        ("relationships", &args.remove_relationship),
    ] {
        for key in keys {
            claim(&mut touched, section, key)?;
        }
    }
    let result = merge_concept_type(existing, &Value::Mapping(supplied))?;
    if result.attested && args.name != "Attested Computation" {
        return Err(OkfError::Usage(
            "attested is reserved for the exact OKF type `Attested Computation`".into(),
        ));
    }
    Ok(result)
}

fn claim(touched: &mut HashSet<(String, String)>, section: &str, key: &str) -> Result<()> {
    if key.trim().is_empty() {
        return Err(OkfError::Usage("declaration keys must be nonempty".into()));
    }
    if !touched.insert((section.into(), key.into())) {
        return Err(OkfError::Usage(format!(
            "overlapping operations for {section}.{key}"
        )));
    }
    Ok(())
}
fn put_declaration(
    map: &mut Mapping,
    touched: &mut HashSet<(String, String)>,
    section: &str,
    key: &str,
    value: Value,
) -> Result<()> {
    claim(touched, section, key)?;
    map.entry(Value::String(section.into()))
        .or_insert_with(|| Value::Mapping(Mapping::new()))
        .as_mapping_mut()
        .unwrap()
        .insert(Value::String(key.into()), value);
    Ok(())
}
fn require_mapping<'a>(value: &'a Value, context: &str) -> Result<&'a Mapping> {
    value
        .as_mapping()
        .ok_or_else(|| OkfError::Usage(format!("{context} must be a mapping")))
}
fn read_definition(reader: &mut InputReader, input: &str) -> Result<Value> {
    let normalized =
        if input != "-" && !input.starts_with('@') && std::path::Path::new(input).is_file() {
            format!("@{input}")
        } else {
            input.to_owned()
        };
    parse_yaml(&reader.read(&normalized)?)
}

fn finish(
    root: &std::path::Path,
    ontology: &Ontology,
    op: &str,
    name: &str,
    dry_run: bool,
    json: bool,
) -> Result<i32> {
    if !dry_run {
        persist(root, ontology)?;
        return report_change(op, name, json);
    }
    let (settings, _) = okf_core::bundle::settings::load_for(root)?;
    let path = settings
        .ontology_path
        .unwrap_or_else(|| ontology_path(root));
    let generated = render_ontology(&path, ontology)?;
    let before = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => {
            return Err(OkfError::Environment(format!(
                "cannot read {}: {e}",
                path.display()
            )))
        }
    };
    let diff = preview_diff(&path, &before, &generated);
    if json {
        output::print_line(
            &json!({"kind":"change", "op":format!("ontology-{op}"), "name":name, "dry_run":true, "diff":diff, "before":before, "after":generated}),
        )?;
    } else {
        print!("{diff}");
    }
    Ok(0)
}

// ---- helpers ----

fn require_ontology(root: &std::path::Path) -> Result<Ontology> {
    okf_core::bundle::settings::load_for(root)?
        .1
        .ok_or_else(|| {
            OkfError::Environment(format!(
                "no ontology.yaml in bundle {} (run `okf ontology add ...` or `okf init`)",
                root.display()
            ))
        })
}

fn load_or_default(root: &std::path::Path) -> Result<Ontology> {
    Ok(okf_core::bundle::settings::load_for(root)?
        .1
        .unwrap_or_else(|| Ontology {
            okf_ontology: ONTOLOGY_VERSION.to_string(),
            ..Ontology::default()
        }))
}

fn persist(root: &std::path::Path, ontology: &Ontology) -> Result<()> {
    let (settings, _) = okf_core::bundle::settings::load_for(root)?;
    save_ontology(
        &settings
            .ontology_path
            .unwrap_or_else(|| ontology_path(root)),
        ontology,
    )
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
            let resolved = resolve_field(ontology,f).ok();
            json!({"key": k, "type": f.type_name, "required": f.required, "values": resolved.as_ref().map(|r|&r.ty.values), "effective":resolved})
        })
        .collect();
    let references: Vec<_> = ct
        .references
        .iter()
        .map(|(k, r)| json!({"key": k, "target": r.target.types(), "cardinality": r.cardinality.as_str(),"selector":r.selector}))
        .collect();
    json!({
        "kind": "ontology_type",
        "name": name,
        "description": ct.description,
        "attested": ct.attested,
        "requires": ct.requires,
        "fields": fields,
        "references": references,
        "relationships": ct.relationships,
        "field_types": ontology.field_types,
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
            selector: None,
            target,
            cardinality,
            extra: Default::default(),
        },
    ))
}
