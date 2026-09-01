use std::collections::BTreeSet;
use std::fmt;

use crate::protocol::{BridgeError, Result, TemplateNode};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticId(String);

impl SemanticId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(BridgeError::InvalidIdentity {
                kind: "semantic".to_owned(),
            });
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SemanticId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateSnapshot {
    pub nodes: Vec<TemplateNode>,
    pub semantic_ids: BTreeSet<SemanticId>,
}

impl TemplateSnapshot {
    pub fn try_new(
        nodes: Vec<TemplateNode>,
        semantic_ids: impl IntoIterator<Item = SemanticId>,
    ) -> Result<Self> {
        validate_nodes(&nodes)?;
        let mut ids = BTreeSet::new();
        for id in semantic_ids {
            if !ids.insert(id.clone()) {
                return Err(BridgeError::DuplicateSemanticId { id: id.to_string() });
            }
        }
        validate_slot_ids(&nodes, &mut BTreeSet::new())?;
        Ok(Self {
            nodes,
            semantic_ids: ids,
        })
    }

    pub fn from_nodes(nodes: Vec<TemplateNode>) -> Result<Self> {
        let mut ids = BTreeSet::new();
        collect_slot_ids(&nodes, &mut ids)?;
        Self::try_new(nodes, ids.into_iter().collect::<Vec<_>>())
    }

    pub fn new(nodes: Vec<TemplateNode>) -> Result<Self> {
        Self::from_nodes(nodes)
    }
}

#[must_use]
pub fn prune_replaced_tree(
    previous: &TemplateSnapshot,
    next: &TemplateSnapshot,
) -> Vec<SemanticId> {
    previous
        .semantic_ids
        .difference(&next.semantic_ids)
        .cloned()
        .collect()
}

fn validate_nodes(nodes: &[TemplateNode]) -> Result<()> {
    for node in nodes {
        match node {
            TemplateNode::Container { tag, children, .. } => {
                require_non_empty(tag, "template tag")?;
                validate_nodes(children)?;
            }
            TemplateNode::Text { .. } => {}
            TemplateNode::Slot { slot_id } => require_non_empty(slot_id, "slot")?,
            TemplateNode::Svg {
                tag,
                attributes,
                children,
            } => {
                require_non_empty(tag, "SVG tag")?;
                for (name, value) in attributes {
                    require_non_empty(name, "SVG attribute name")?;
                    if value.contains('\0') {
                        return Err(BridgeError::InvalidIdentity {
                            kind: "SVG attribute value".to_owned(),
                        });
                    }
                }
                validate_nodes(children)?;
            }
        }
    }
    Ok(())
}

fn validate_slot_ids(nodes: &[TemplateNode], seen: &mut BTreeSet<String>) -> Result<()> {
    for node in nodes {
        match node {
            TemplateNode::Slot { slot_id } => {
                if !seen.insert(slot_id.clone()) {
                    return Err(BridgeError::DuplicateSemanticId {
                        id: slot_id.clone(),
                    });
                }
            }
            TemplateNode::Container { children, .. } | TemplateNode::Svg { children, .. } => {
                validate_slot_ids(children, seen)?;
            }
            TemplateNode::Text { .. } => {}
        }
    }
    Ok(())
}

fn collect_slot_ids(nodes: &[TemplateNode], ids: &mut BTreeSet<SemanticId>) -> Result<()> {
    for node in nodes {
        match node {
            TemplateNode::Slot { slot_id } => {
                ids.insert(SemanticId::new(slot_id.clone())?);
            }
            TemplateNode::Container { children, .. } | TemplateNode::Svg { children, .. } => {
                collect_slot_ids(children, ids)?;
            }
            TemplateNode::Text { .. } => {}
        }
    }
    Ok(())
}

fn require_non_empty(value: &str, kind: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(BridgeError::InvalidIdentity {
            kind: kind.to_owned(),
        });
    }
    Ok(())
}
