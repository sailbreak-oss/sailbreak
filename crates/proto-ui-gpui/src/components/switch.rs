use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::{
    A11ySnapshot, AdapterSnapshot, BridgeDiagnostic, BridgeError, BridgeEvent, ButtonStyle,
    CommitDisposition, InputKind, InputSource, LogicalParentRef, NativeStyle, ProtoAdapter,
    PrototypeKey, PrototypeProfile, Result, SessionId, SessionSnapshot, ShadcnTheme,
    SlotProjection, StartRequest, ViewEpoch,
};

const SWITCH_ROOT_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnSwitchRoot,
    exposed_states: &[
        "checked",
        "disabled",
        "hovered",
        "pressed",
        "focused",
        "focusVisible",
    ],
    signals: &["checkedChange"],
};

const SWITCH_THUMB_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnSwitchThumb,
    exposed_states: &["checked"],
    signals: &[],
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SwitchProps {
    pub checked: Option<bool>,
    pub default_checked: bool,
    pub disabled: bool,
}

impl SwitchProps {
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
        props
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProtoSwitchSnapshot {
    pub id: String,
    pub label: String,
    pub root: SwitchRootSnapshot,
    pub thumb: Option<SwitchThumbSnapshot>,
    pub checked: bool,
    pub disabled: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub focus_visible: bool,
    pub a11y: Option<A11ySnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SwitchRootSnapshot {
    pub id: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub resolved_style: ButtonStyle,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SwitchThumbSnapshot {
    pub id: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub resolved_style: ButtonStyle,
    pub checked: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SwitchDispatchOutcome {
    pub checked_change_count: usize,
    pub checked_changes: Vec<bool>,
    pub events: Vec<BridgeEvent>,
    pub diagnostics: Vec<BridgeDiagnostic>,
}

pub struct ProtoSwitchHost {
    adapter: ProtoAdapter,
    roots: BTreeMap<String, SwitchProps>,
    thumbs: BTreeMap<String, String>,
}

impl ProtoSwitchHost {
    pub fn new() -> Result<Self> {
        Self::with_theme(ShadcnTheme::default())
    }

    pub fn with_theme(theme: ShadcnTheme) -> Result<Self> {
        Ok(Self {
            adapter: ProtoAdapter::with_theme(theme)?,
            roots: BTreeMap::new(),
            thumbs: BTreeMap::new(),
        })
    }

    pub fn register_root(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        props: SwitchProps,
    ) -> Result<()> {
        let id = id.into();
        let label = label.into();
        validate_identity(&id, "switch root")?;
        validate_identity(&label, "switch accessible name")?;
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:switch:{id}"))?,
            crate::InstanceId::new(format!("sailbreak:switch-instance:{id}"))?,
            PrototypeKey::ShadcnSwitchRoot,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), label.clone()),
        );
        self.adapter
            .start(id.clone(), label, SWITCH_ROOT_PROFILE, request)?;
        self.roots.insert(id, props);
        Ok(())
    }

    pub fn register_thumb(&mut self, id: impl Into<String>, root_id: &str) -> Result<()> {
        let parent = self.parent_ref(root_id)?;
        self.register_thumb_with_parent(id, parent)
    }

    pub fn register_thumb_with_parent(
        &mut self,
        id: impl Into<String>,
        parent: LogicalParentRef,
    ) -> Result<()> {
        let id = id.into();
        validate_identity(&id, "switch thumb")?;
        let parent_id = parent
            .session_id
            .as_str()
            .strip_prefix("sailbreak:switch:")
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "thumb parent session id format".to_owned(),
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
            SessionId::new(format!("sailbreak:switch-thumb:{id}"))?,
            crate::InstanceId::new(format!("sailbreak:switch-thumb-instance:{id}"))?,
            PrototypeKey::ShadcnSwitchThumb,
            Map::new(),
            SlotProjection::new(format!("{id}:slot"), "Switch thumb"),
        )
        .with_parent(parent);
        self.adapter
            .start(id.clone(), "Switch thumb", SWITCH_THUMB_PROFILE, request)?;
        self.thumbs.insert(id, parent_id);
        Ok(())
    }

    pub fn replace_thumb(&mut self, root_id: &str, new_thumb_id: impl Into<String>) -> Result<()> {
        let old_ids: Vec<String> = self
            .thumbs
            .iter()
            .filter(|(_, parent)| *parent == root_id)
            .map(|(id, _)| id.clone())
            .collect();
        for id in old_ids {
            self.adapter.dispose(&id)?;
            self.thumbs.remove(&id);
        }
        self.register_thumb(new_thumb_id, root_id)
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
    ) -> Result<SwitchDispatchOutcome> {
        self.require_root(id)?;
        let before = self.adapter.snapshot(id)?;
        let outcome = self.adapter.dispatch(id, kind, source, detail)?;
        let count = outcome.signal_count("checkedChange");
        let checked_changes = if count == 0 {
            Vec::new()
        } else {
            let after = self.adapter.snapshot(id)?;
            vec![
                after
                    .state_bool("checked")
                    .or_else(|| before.state_bool("checked"))
                    .unwrap_or(false),
            ]
        };
        Ok(SwitchDispatchOutcome {
            checked_change_count: count,
            checked_changes,
            events: outcome.events,
            diagnostics: outcome.diagnostics,
        })
    }

    pub fn set_props(&mut self, id: &str, props: SwitchProps) -> Result<CommitDisposition> {
        self.require_root(id)?;
        let disposition = self.adapter.set_props(id, props.to_map())?;
        self.roots.insert(id.to_owned(), props);
        Ok(disposition)
    }

    pub fn remount(&mut self, id: &str) -> Result<ViewEpoch> {
        self.require_root(id)?;
        self.adapter.remount(id)
    }

    pub fn snapshot(&mut self, id: &str) -> Result<ProtoSwitchSnapshot> {
        let props = self.require_root(id)?.clone();
        let theme = self.adapter.theme();
        let root = self.adapter.snapshot(id)?;
        let checked = root
            .state_bool("checked")
            .unwrap_or_else(|| props.checked.unwrap_or(props.default_checked));
        let disabled = root.state_bool("disabled").unwrap_or(props.disabled);
        let hovered = root.state_bool("hovered").unwrap_or(false);
        let pressed = root.state_bool("pressed").unwrap_or(false);
        let focus_visible = root.state_bool("focusVisible").unwrap_or(false);
        let root_resolved_style = ButtonStyle::from_projection(&root.session.style, theme);
        let a11y = root.session.a11y.clone();
        let thumb_id = self
            .thumbs
            .iter()
            .find_map(|(thumb, parent)| (parent == id).then(|| thumb.clone()));
        let thumb = match thumb_id {
            Some(thumb) => Some(self.thumb_snapshot(&thumb)?),
            None => None,
        };
        Ok(ProtoSwitchSnapshot {
            id: root.id.clone(),
            label: root.label.clone(),
            root: root_snapshot(root, root_resolved_style),
            thumb,
            checked,
            disabled,
            hovered,
            pressed,
            focus_visible,
            a11y,
        })
    }

    pub fn thumb_snapshot(&mut self, id: &str) -> Result<SwitchThumbSnapshot> {
        if !self.thumbs.contains_key(id) {
            return Err(unknown_switch(id));
        }
        let theme = self.adapter.theme();
        let snapshot = self.adapter.snapshot(id)?;
        let checked = snapshot.state_bool("checked").unwrap_or(false);
        let resolved_style = ButtonStyle::from_projection(&snapshot.session.style, theme);
        Ok(SwitchThumbSnapshot {
            id: snapshot.id.clone(),
            session: snapshot.session,
            native_style: snapshot.native_style,
            resolved_style,
            checked,
        })
    }

    pub fn dispose(&mut self, id: &str) -> Result<()> {
        self.require_root(id)?;
        let thumb_ids: Vec<String> = self
            .thumbs
            .iter()
            .filter(|(_, parent)| *parent == id)
            .map(|(thumb, _)| thumb.clone())
            .collect();
        for thumb in thumb_ids {
            self.adapter.dispose(&thumb)?;
            self.thumbs.remove(&thumb);
        }
        self.adapter.dispose(id)?;
        self.roots.remove(id);
        Ok(())
    }

    fn require_root(&self, id: &str) -> Result<&SwitchProps> {
        self.roots.get(id).ok_or_else(|| unknown_switch(id))
    }
}

fn root_snapshot(snapshot: AdapterSnapshot, resolved_style: ButtonStyle) -> SwitchRootSnapshot {
    SwitchRootSnapshot {
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

fn unknown_switch(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown switch: {id}"),
    }
}
