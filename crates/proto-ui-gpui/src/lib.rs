#![forbid(unsafe_code)]

mod button_host;
mod components;
mod protocol;
mod quickjs;
mod runtime;
mod style;
mod template;
mod theme;

pub use button_host::{ProtoButtonHost, ProtoButtonState, ShadcnButtonSize, ShadcnButtonVariant};
pub use components::{
    ProtoToggleHost, ProtoToggleSnapshot, ToggleDispatchOutcome, ToggleProps, ToggleSize,
    ToggleVariant,
};
pub use protocol::*;
pub use quickjs::QuickJsBridge;
pub use runtime::{
    CommitDisposition, InputRequest, PropsRequest, ProtoSessionHost, SessionSnapshot, StartRequest,
};
pub use style::{NativeStyle, translate_projection, translate_style};
pub use template::{SemanticId, TemplateSnapshot, prune_replaced_tree};
pub use theme::{ButtonStyle, ColorScheme, ColorValue, ShadcnTheme};
