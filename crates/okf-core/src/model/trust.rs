//! Trust tiers and the trust frontmatter families.
use super::frontmatter::Frontmatter;
use super::standard::{parse_timestamp, valid_actor};
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
/// Only a standard `{ by, at }` mapping is a verification event. Malformed optional metadata
/// remains consumable but does not elevate trust.
fn entry_actor(entry: &Value) -> Option<&str> {
    let by = entry.get("by")?.as_str()?;
    let at = entry.get("at")?.as_str()?;
    (valid_actor(by) && parse_timestamp(at).is_some()).then_some(by)
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
        if let Some(actor) = entry_actor(entry) {
            any = true;
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
        let f = fm("verified:\n- by: process:ci\n  at: 2026-01-01T00:00:00Z");
        assert_eq!(derive_trust_tier(&f), TrustTier::MachineConfirmed);
    }

    #[test]
    fn any_human_is_human_reviewed() {
        let f = fm("verified:\n- by: process:ci\n  at: 2026-01-01T00:00:00Z\n- by: human:andrey\n  at: 2026-01-02T00:00:00Z");
        assert_eq!(derive_trust_tier(&f), TrustTier::HumanReviewed);
    }

    #[test]
    fn empty_verified_is_unverified() {
        assert_eq!(
            derive_trust_tier(&fm("verified: []")),
            TrustTier::Unverified
        );
    }

    #[test]
    fn malformed_verified_does_not_elevate_trust() {
        assert_eq!(
            derive_trust_tier(&fm("verified: {by: human:x}")),
            TrustTier::Unverified
        );
        assert_eq!(
            derive_trust_tier(&fm("verified: nonsense")),
            TrustTier::Unverified
        );
    }
}
