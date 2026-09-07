//! Trust tiers and the trust frontmatter families.
use super::frontmatter::Frontmatter;
use serde_yaml::Value;

/// Derived trust tier (lowest to highest), computed from the `verified` field's actors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustTier {
    Unverified,
    MachineConfirmed,
    HumanReviewed,
}

impl TrustTier {
    /// Stable machine string used in NDJSON records and human output.
    pub fn as_str(&self) -> &'static str {
        match self {
            TrustTier::Unverified => "unverified",
            TrustTier::MachineConfirmed => "machine-confirmed",
            TrustTier::HumanReviewed => "human-reviewed",
        }
    }
}

/// Extract the actor string from a single `verified` entry.
///
/// An entry is normally a mapping with a `by` (or `actor`) key, but we also tolerate a
/// bare string entry (`verified: ["human:andrey"]`).
fn entry_actor(entry: &Value) -> Option<&str> {
    if let Some(s) = entry.as_str() {
        return Some(s);
    }
    entry
        .get("by")
        .or_else(|| entry.get("actor"))
        .and_then(Value::as_str)
}

/// Derive the trust tier from a concept's frontmatter per the decided rule:
/// - no `verified`            → `unverified`
/// - `verified` by non-`human:` actors only → `machine-confirmed`
/// - any `verified` entry whose actor starts with `human:` → `human-reviewed`
pub fn derive_trust_tier(fm: &Frontmatter) -> TrustTier {
    let verified = match fm.get("verified") {
        Some(v) => v,
        None => return TrustTier::Unverified,
    };

    // Normalize to a list of entries: a sequence, or a single mapping/scalar.
    let entries: Vec<&Value> = match verified {
        Value::Sequence(seq) => seq.iter().collect(),
        Value::Null => Vec::new(),
        other => vec![other],
    };

    if entries.is_empty() {
        return TrustTier::Unverified;
    }

    let mut any = false;
    let mut human = false;
    for entry in entries {
        any = true;
        if let Some(actor) = entry_actor(entry) {
            if actor.starts_with("human:") {
                human = true;
            }
        }
    }

    if !any {
        TrustTier::Unverified
    } else if human {
        TrustTier::HumanReviewed
    } else {
        TrustTier::MachineConfirmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    fn fm(yaml: &str) -> Frontmatter {
        let map: IndexMap<String, Value> = serde_yaml::from_str(yaml).unwrap();
        Frontmatter::from_map(map)
    }

    #[test]
    fn no_verified_is_unverified() {
        assert_eq!(derive_trust_tier(&fm("type: table")), TrustTier::Unverified);
    }

    #[test]
    fn machine_only() {
        let f = fm("verified:\n- by: process:ci\n  at: 2026-01-01");
        assert_eq!(derive_trust_tier(&f), TrustTier::MachineConfirmed);
    }

    #[test]
    fn any_human_is_human_reviewed() {
        let f = fm("verified:\n- by: process:ci\n- by: human:andrey");
        assert_eq!(derive_trust_tier(&f), TrustTier::HumanReviewed);
    }

    #[test]
    fn empty_verified_is_unverified() {
        assert_eq!(derive_trust_tier(&fm("verified: []")), TrustTier::Unverified);
    }
}
