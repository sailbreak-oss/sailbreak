use std::collections::BTreeMap;

use crate::{BridgeError, Result, ViewEpoch};

/// Opaque identity of one native focus target, stable within one host adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FocusTarget {
    pub session_id: String,
    pub instance_id: String,
    pub view_epoch: ViewEpoch,
    pub route_ref: String,
    pub role: String,
}

/// Result of one native focus operation. Allocating a `FocusHandle` does not
/// make a target ready; readiness is decided by the host adapter once the
/// native surface is present.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FocusOperationResult {
    Accepted,
    NotReady,
    Rejected,
}

/// Host-owned registry of opaque focus targets for one component family.
///
/// Each entry tracks the last projected native view epoch. A focus request
/// against a stale target is `Rejected`; a request against a registered target
/// that has not yet become natively present is `NotReady`.
#[derive(Clone, Debug, Default)]
pub struct FocusRegistry {
    targets: BTreeMap<String, FocusTarget>,
}

impl FocusRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, id: impl Into<String>, target: FocusTarget) -> Result<()> {
        let id = id.into();
        validate_identity(&id, "focus target")?;
        validate_identity(&target.session_id, "focus session")?;
        validate_identity(&target.instance_id, "focus instance")?;
        validate_identity(&target.route_ref, "focus route")?;
        self.targets.insert(id, target);
        Ok(())
    }

    pub fn remove(&mut self, id: &str) {
        self.targets.remove(id);
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<&FocusTarget> {
        self.targets.get(id)
    }
}

fn validate_identity(value: &str, kind: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(BridgeError::InvalidIdentity {
            kind: kind.to_owned(),
        });
    }
    Ok(())
}
