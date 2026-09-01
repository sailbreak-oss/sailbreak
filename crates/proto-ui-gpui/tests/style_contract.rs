use proto_ui_gpui::{ShadcnTheme, translate_style};

#[test]
fn semantic_tokens_translate_and_unsupported_tokens_are_named() {
    let native = translate_style(
        vec![
            "inline-flex".to_owned(),
            "bg-primary".to_owned(),
            "h-9".to_owned(),
            "rounded-md".to_owned(),
            "ring-ring/50".to_owned(),
            "shadow-md".to_owned(),
            "opacity-50".to_owned(),
            "unknown-token".to_owned(),
        ],
        ShadcnTheme::dark(),
    );
    assert_eq!(native.tokens[0], "inline-flex");
    assert!(native.unsupported.contains(&"unknown-token".to_owned()));
    assert!(!native.unsupported.contains(&"bg-primary".to_owned()));
}

#[test]
fn current_component_token_families_are_covered_or_reported() {
    let tokens = [
        "flex",
        "grid",
        "items-center",
        "justify-between",
        "gap-2",
        "space-y-2",
        "w-full",
        "min-w-0",
        "max-w-sm",
        "h-4",
        "size-4",
        "p-1",
        "px-3",
        "py-2",
        "rounded-sm",
        "border",
        "border-input",
        "bg-background",
        "bg-muted",
        "text-foreground",
        "text-muted-foreground",
        "text-xs",
        "font-medium",
        "shadow-xs",
        "shadow-md",
        "opacity-50",
        "transition-all",
        "duration-150",
        "outline-none",
        "focus-visible:ring-3",
        "translate-y-px",
        "pointer-events-none",
    ];
    let native = translate_style(
        tokens.iter().map(|token| (*token).to_owned()).collect(),
        ShadcnTheme::dark(),
    );
    assert!(
        native.unsupported.is_empty(),
        "unsupported component tokens: {:?}",
        native.unsupported
    );
}
