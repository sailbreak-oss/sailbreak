use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::{
    A11ySnapshot, AdapterSnapshot, BridgeDiagnostic, BridgeError, BridgeEvent, ButtonStyle,
    CommitDisposition, InputKind, InputSource, LogicalParentRef, NativeStyle, ProtoAdapter,
    PrototypeKey, PrototypeProfile, Result, SessionId, SessionSnapshot, ShadcnTheme,
    SlotProjection, StartRequest, ViewEpoch,
};

const CHECKBOX_ROOT_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnCheckboxRoot,
    exposed_states: &[
        "checked",
        "indeterminate",
        "disabled",
        "hovered",
        "pressed",
        "focused",
        "focusVisible",
    ],
    signals: &["checkedChange", "indeterminateChange"],
};

const CHECKBOX_INDICATOR_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnCheckboxIndicator,
    exposed_states: &["checked", "indeterminate"],
    signals: &[],
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CheckboxProps {
    pub checked: Option<bool>,
    pub default_checked: bool,
    pub disabled: bool,
    pub indeterminate: Option<bool>,
    pub default_indeterminate: bool,
}

impl CheckboxProps {
    fn to_map(&self) -> Map<String, Value> {
        let mut props = Map::new();
        if let Some(checked) = self.checked {
            props.insert("checked".to_owned(), Value::Bool(checked));
        }
        props.insert(
            "defaultChecked".to_owned(),
            Value::Bool(self.default_checked),
        );
        props.insert("disabled".to_owned(), Value::Bool(self.disabled));
        if let Some(indeterminate) = self.indeterminate {
            props.insert("indeterminate".to_owned(), Value::Bool(indeterminate));
        }
        props.insert(
            "defaultIndeterminate".to_owned(),
            Value::Bool(self.default_indeterminate),
        );
        props
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProtoCheckboxSnapshot {
    pub id: String,
    pub label: String,
    pub root: CheckboxRootSnapshot,
    pub indicator: Option<CheckboxIndicatorSnapshot>,
    pub checked: bool,
    pub indeterminate: bool,
    pub disabled: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub focus_visible: bool,
    pub a11y: Option<A11ySnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CheckboxRootSnapshot {
    pub id: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub resolved_style: ButtonStyle,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CheckboxIndicatorSnapshot {
    pub id: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub resolved_style: ButtonStyle,
    pub checked: bool,
    pub indeterminate: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CheckboxDispatchOutcome {
    pub checked_change_count: usize,
    pub checked_changes: Vec<bool>,
    pub indeterminate_change_count: usize,
    pub indeterminate_changes: Vec<bool>,
    pub events: Vec<BridgeEvent>,
    pub diagnostics: Vec<BridgeDiagnostic>,
}

pub struct ProtoCheckboxHost {
    adapter: ProtoAdapter,
    roots: BTreeMap<String, CheckboxProps>,
    indicators: BTreeMap<String, String>,
}

impl ProtoCheckboxHost {
    pub fn new() -> Result<Self> {
        Self::with_theme(ShadcnTheme::default())
    }

    pub fn with_theme(theme: ShadcnTheme) -> Result<Self> {
        Ok(Self {
            adapter: ProtoAdapter::with_theme(theme)?,
            roots: BTreeMap::new(),
            indicators: BTreeMap::new(),
        })
    }

    pub fn register_root(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        props: CheckboxProps,
    ) -> Result<()> {
        let id = id.into();
        let label = label.into();
        validate_identity(&id, "checkbox root")?;
        validate_identity(&label, "checkbox accessible name")?;
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:checkbox:{id}"))?,
            crate::InstanceId::new(format!("sailbreak:checkbox-instance:{id}"))?,
            PrototypeKey::ShadcnCheckboxRoot,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), label.clone()),
        );
        self.adapter
            .start(id.clone(), label, CHECKBOX_ROOT_PROFILE, request)?;
        self.roots.insert(id, props);
        Ok(())
    }

    pub fn register_indicator(&mut self, id: impl Into<String>, root_id: &str) -> Result<()> {
        let parent = self.parent_ref(root_id)?;
        self.register_indicator_with_parent(id, parent)
    }

    pub fn register_indicator_with_parent(
        &mut self,
        id: impl Into<String>,
        parent: LogicalParentRef,
    ) -> Result<()> {
        let id = id.into();
        validate_identity(&id, "checkbox indicator")?;
        let parent_id = parent
            .session_id
            .as_str()
            .strip_prefix("sailbreak:checkbox:")
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "indicator parent session id format".to_owned(),
            })?
            .to_owned();
        let current = self.adapter.parent_ref(&parent_id)?;
        if current.session_id != parent.session_id {
            return Err(BridgeError::SessionMismatch {
                expected: current.session_id,
                received: parent.session_id,
            });
        }
        if current.instance_id != parent.instance_id {
            return Err(BridgeError::InstanceMismatch {
                expected: current.instance_id,
                received: parent.instance_id,
            });
        }
        if current.route_ref != parent.route_ref {
            return Err(BridgeError::ParentRouteMismatch {
                expected: current.route_ref,
                received: parent.route_ref,
            });
        }
        if current.view_epoch != parent.view_epoch {
            return Err(BridgeError::StaleParent {
                expected: current.view_epoch,
                received: parent.view_epoch,
            });
        }
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:checkbox-indicator:{id}"))?,
            crate::InstanceId::new(format!("sailbreak:checkbox-indicator-instance:{id}"))?,
            PrototypeKey::ShadcnCheckboxIndicator,
            Map::new(),
            SlotProjection::new(format!("{id}:slot"), "Checkbox indicator"),
        )
        .with_parent(parent);
        self.adapter.start(
            id.clone(),
            "Checkbox indicator",
            CHECKBOX_INDICATOR_PROFILE,
            request,
        )?;
        self.indicators.insert(id, parent_id);
        Ok(())
    }

    pub fn replace_indicator(
        &mut self,
        root_id: &str,
        new_indicator_id: impl Into<String>,
    ) -> Result<()> {
        let old_ids: Vec<String> = self
            .indicators
            .iter()
            .filter(|(_, parent)| *parent == root_id)
            .map(|(id, _)| id.clone())
            .collect();
        for id in old_ids {
            self.adapter.dispose(&id)?;
            self.indicators.remove(&id);
        }
        self.register_indicator(new_indicator_id, root_id)
    }

    pub fn parent_ref(&mut self, id: &str) -> Result<LogicalParentRef> {
        self.require_root(id)?;
        self.adapter.parent_ref(id)
    }

    pub fn dispatch(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<CheckboxDispatchOutcome> {
        self.require_root(id)?;
        let before = self.adapter.snapshot(id)?;
        let outcome = self.adapter.dispatch(id, kind, source, detail)?;
        let checked_change_count = outcome.signal_count("checkedChange");
        let indeterminate_change_count = outcome.signal_count("indeterminateChange");
        let checked_changes = if checked_change_count == 0 {
            Vec::new()
        } else {
            vec![!before.state_bool("checked").unwrap_or(false)]
        };
        let indeterminate_changes = if indeterminate_change_count == 0 {
            Vec::new()
        } else {
            vec![false]
        };
        Ok(CheckboxDispatchOutcome {
            checked_change_count,
            checked_changes,
            indeterminate_change_count,
            indeterminate_changes,
            events: outcome.events,
            diagnostics: outcome.diagnostics,
        })
    }

    pub fn set_props(&mut self, id: &str, props: CheckboxProps) -> Result<CommitDisposition> {
        self.require_root(id)?;
        let disposition = self.adapter.set_props(id, props.to_map())?;
        self.roots.insert(id.to_owned(), props);
        Ok(disposition)
    }

    pub fn remount(&mut self, id: &str) -> Result<ViewEpoch> {
        self.require_root(id)?;
        self.adapter.remount(id)
    }

    pub fn snapshot(&mut self, id: &str) -> Result<ProtoCheckboxSnapshot> {
        let props = self.require_root(id)?.clone();
        let theme = self.adapter.theme();
        let root = self.adapter.snapshot(id)?;
        let checked = root
            .state_bool("checked")
            .unwrap_or_else(|| props.checked.unwrap_or(props.default_checked));
        let indeterminate = root
            .state_bool("indeterminate")
            .unwrap_or_else(|| props.indeterminate.unwrap_or(props.default_indeterminate));
        let disabled = root.state_bool("disabled").unwrap_or(props.disabled);
        let hovered = root.state_bool("hovered").unwrap_or(false);
        let pressed = root.state_bool("pressed").unwrap_or(false);
        let focus_visible = root.state_bool("focusVisible").unwrap_or(false);
        let root_resolved_style = ButtonStyle::from_projection(&root.session.style, theme);
        let a11y = root.session.a11y.clone();
        let indicator_id = self
            .indicators
            .iter()
            .find_map(|(indicator, parent)| (parent == id).then(|| indicator.clone()));
        let indicator = match indicator_id {
            Some(indicator) => Some(self.indicator_snapshot(&indicator)?),
            None => None,
        };
        Ok(ProtoCheckboxSnapshot {
            id: root.id.clone(),
            label: root.label.clone(),
            root: root_snapshot(root, root_resolved_style),
            indicator,
            checked,
            indeterminate,
            disabled,
            hovered,
            pressed,
            focus_visible,
            a11y,
        })
    }

    pub fn indicator_snapshot(&mut self, id: &str) -> Result<CheckboxIndicatorSnapshot> {
        if !self.indicators.contains_key(id) {
            return Err(unknown_checkbox(id));
        }
        let theme = self.adapter.theme();
        let snapshot = self.adapter.snapshot(id)?;
        let checked = snapshot.state_bool("checked").unwrap_or(false);
        let indeterminate = snapshot.state_bool("indeterminate").unwrap_or(false);
        let resolved_style = ButtonStyle::from_projection(&snapshot.session.style, theme);
        Ok(CheckboxIndicatorSnapshot {
            id: snapshot.id.clone(),
            session: snapshot.session,
            native_style: snapshot.native_style,
            resolved_style,
            checked,
            indeterminate,
        })
    }

    pub fn dispose(&mut self, id: &str) -> Result<()> {
        self.require_root(id)?;
        let indicator_ids: Vec<String> = self
            .indicators
            .iter()
            .filter(|(_, parent)| *parent == id)
            .map(|(indicator, _)| indicator.clone())
            .collect();
        for indicator in indicator_ids {
            self.adapter.dispose(&indicator)?;
            self.indicators.remove(&indicator);
        }
        self.adapter.dispose(id)?;
        self.roots.remove(id);
        Ok(())
    }

    fn require_root(&self, id: &str) -> Result<&CheckboxProps> {
        self.roots.get(id).ok_or_else(|| unknown_checkbox(id))
    }
}

fn root_snapshot(snapshot: AdapterSnapshot, resolved_style: ButtonStyle) -> CheckboxRootSnapshot {
    CheckboxRootSnapshot {
        id: snapshot.id,
        session: snapshot.session,
        native_style: snapshot.native_style,
        resolved_style,
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

fn unknown_checkbox(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown checkbox: {id}"),
    }
}
