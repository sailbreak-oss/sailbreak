mod adapter;
mod switch;
mod toggle;

pub use adapter::{AdapterDispatchOutcome, AdapterSnapshot, ProtoAdapter, PrototypeProfile};

pub use switch::{
    ProtoSwitchHost, ProtoSwitchSnapshot, SwitchDispatchOutcome, SwitchProps, SwitchRootSnapshot,
    SwitchThumbSnapshot,
};

pub use toggle::{
    ProtoToggleHost, ProtoToggleSnapshot, ToggleDispatchOutcome, ToggleProps, ToggleSize,
    ToggleVariant,
};
