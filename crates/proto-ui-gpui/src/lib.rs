#![forbid(unsafe_code)]

mod button_host;
mod protocol;
mod quickjs;
mod runtime;
mod theme;

pub use button_host::{ProtoButtonHost, ProtoButtonState, ShadcnButtonSize, ShadcnButtonVariant};
pub use protocol::*;
pub use quickjs::QuickJsBridge;
pub use runtime::{
    CommitDisposition, InputRequest, PropsRequest, ProtoSessionHost, SessionSnapshot, StartRequest,
};
pub use theme::{ButtonStyle, ColorScheme, ColorValue, ShadcnTheme};
