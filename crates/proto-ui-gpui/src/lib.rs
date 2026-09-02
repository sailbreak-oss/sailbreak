#![forbid(unsafe_code)]

mod button_host;
mod components;
mod focus;
mod overlay;
mod protocol;
mod quickjs;
mod runtime;
mod style;
mod template;
mod theme;

pub use button_host::{ProtoButtonHost, ProtoButtonState, ShadcnButtonSize, ShadcnButtonVariant};
pub use components::{
    AdapterDispatchOutcome, AdapterSnapshot, CheckboxDispatchOutcome, CheckboxIndicatorSnapshot,
    CheckboxProps, CheckboxRootSnapshot, ProtoAdapter, ProtoCheckboxHost, ProtoCheckboxSnapshot,
    ProtoSelectHost, ProtoSelectSnapshot, ProtoSeparatorHost, ProtoSeparatorSnapshot,
    ProtoSwitchHost, ProtoSwitchSnapshot, ProtoTabsHost, ProtoTextareaHost, ProtoTextareaSnapshot,
    ProtoToggleHost, ProtoToggleSnapshot, PrototypeProfile, SelectContentPosition,
    SelectContentProps, SelectContentSnapshot, SelectDispatchOutcome, SelectItemProps,
    SelectItemSnapshot, SelectPosition, SelectRootProps, SelectRootSnapshot, SelectSnapshot,
    SelectTriggerProps, SelectTriggerSnapshot, SelectValueProps, SelectValueSnapshot,
    SeparatorOrientation, SeparatorProps, SwitchDispatchOutcome, SwitchProps, SwitchRootSnapshot,
    SwitchThumbSnapshot, TabsActivationMode, TabsContentProps, TabsContentSnapshot,
    TabsDispatchOutcome, TabsListProps, TabsListSnapshot, TabsOrientation, TabsRootProps,
    TabsRootSnapshot, TabsSnapshot, TabsTriggerProps, TabsTriggerSnapshot, TextareaDispatchOutcome,
    TextareaProps, TextareaWrap, ToggleDispatchOutcome, ToggleProps, ToggleSize, ToggleVariant,
};
pub use focus::{FocusOperationResult, FocusRegistry, FocusTarget};
pub use overlay::*;
pub use protocol::*;
pub use quickjs::QuickJsBridge;
pub use runtime::{
    CommitDisposition, InputRequest, PropsRequest, ProtoSessionHost, SessionSnapshot, StartRequest,
};
pub use style::{NativeStyle, translate_projection, translate_style};
pub use template::{SemanticId, TemplateSnapshot, prune_replaced_tree};
pub use theme::{ButtonStyle, ColorScheme, ColorValue, ShadcnTheme};
