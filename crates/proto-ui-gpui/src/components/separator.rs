use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::{
    AdapterSnapshot, BridgeError, ColorValue, CommitDisposition, NativeStyle, ProtoAdapter,
    PrototypeKey, PrototypeProfile, Result, SessionId, SessionSnapshot, ShadcnTheme,
    SlotProjection, StartRequest, ViewEpoch,
};

const SEPARATOR_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnSeparatorRoot,
    exposed_states: &["orientation", "decorative"],
    signals: &[],
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SeparatorOrientation {
    #[default]
    Horizontal,
    Vertical,
}

impl SeparatorOrientation {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeparatorProps {
    pub orientation: SeparatorOrientation,
    pub decorative: bool,
}

impl Default for SeparatorProps {
    fn default() -> Self {
        Self {
            orientation: SeparatorOrientation::Horizontal,
            decorative: true,
        }
    }
}

impl SeparatorProps {
    fn to_map(&self) -> Map<String, Value> {
        Map::from_iter([
            (
                "orientation".to_owned(),
                Value::String(self.orientation.as_str().to_owned()),
            ),
            ("decorative".to_owned(), Value::Bool(self.decorative)),
        ])
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProtoSeparatorSnapshot {
    pub id: String,
    pub profile: PrototypeProfile,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub color: ColorValue,
    pub orientation: SeparatorOrientation,
    pub decorative: bool,
}

pub struct ProtoSeparatorHost {
    adapter: ProtoAdapter,
    props: BTreeMap<String, SeparatorProps>,
}

impl ProtoSeparatorHost {
    pub fn new() -> Result<Self> {
        Self::with_theme(ShadcnTheme::default())
    }

    pub fn with_theme(theme: ShadcnTheme) -> Result<Self> {
        Ok(Self {
            adapter: ProtoAdapter::with_theme(theme)?,
            props: BTreeMap::new(),
        })
    }

    pub fn register(&mut self, id: impl Into<String>, props: SeparatorProps) -> Result<()> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(BridgeError::InvalidIdentity {
                kind: "separator".to_owned(),
            });
        }
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:separator:{id}"))?,
            crate::InstanceId::new(format!("sailbreak:separator-instance:{id}"))?,
            PrototypeKey::ShadcnSeparatorRoot,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), "Separator"),
        );
        self.adapter
            .start(id.clone(), "", SEPARATOR_PROFILE, request)?;
        self.props.insert(id, props);
        Ok(())
    }

    pub fn set_props(&mut self, id: &str, props: SeparatorProps) -> Result<CommitDisposition> {
        self.require_props(id)?;
        let disposition = self.adapter.set_props(id, props.to_map())?;
        self.props.insert(id.to_owned(), props);
        Ok(disposition)
    }

    pub fn snapshot(&self, id: &str) -> Result<ProtoSeparatorSnapshot> {
        let props = self.require_props(id)?.clone();
        let theme = self.adapter.theme();
        let snapshot = self.adapter.snapshot_current(id)?;
        Ok(build_snapshot(snapshot, props, theme))
    }

    pub fn remount(&mut self, id: &str) -> Result<ViewEpoch> {
        self.require_props(id)?;
        self.adapter.remount(id)
    }

    pub fn dispose(&mut self, id: &str) -> Result<()> {
        self.require_props(id)?;
        self.adapter.dispose(id)?;
        self.props.remove(id);
        Ok(())
    }

    fn require_props(&self, id: &str) -> Result<&SeparatorProps> {
        self.props
            .get(id)
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: format!("unknown separator: {id}"),
            })
    }
}

fn build_snapshot(
    snapshot: AdapterSnapshot,
    props: SeparatorProps,
    theme: ShadcnTheme,
) -> ProtoSeparatorSnapshot {
    let orientation = snapshot
        .session
        .state_values
        .get("orientation")
        .and_then(Value::as_str)
        .and_then(|orientation| match orientation {
            "horizontal" => Some(SeparatorOrientation::Horizontal),
            "vertical" => Some(SeparatorOrientation::Vertical),
            _ => None,
        })
        .unwrap_or(props.orientation);
    let decorative = snapshot
        .state_bool("decorative")
        .unwrap_or(props.decorative);
    ProtoSeparatorSnapshot {
        id: snapshot.id,
        profile: snapshot.profile,
        session: snapshot.session,
        native_style: snapshot.native_style,
        color: theme
            .color("border")
            .unwrap_or_else(|| ColorValue::opaque(theme.border)),
        orientation,
        decorative,
    }
}
