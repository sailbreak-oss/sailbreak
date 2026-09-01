use crate::{ShadcnTheme, StyleProjection};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeStyle {
    pub tokens: Vec<String>,
    pub unsupported: Vec<String>,
}

impl NativeStyle {
    #[must_use]
    pub fn is_fully_supported(&self) -> bool {
        self.unsupported.is_empty()
    }
}

#[must_use]
pub fn translate_style(tokens: Vec<String>, _theme: ShadcnTheme) -> NativeStyle {
    let unsupported = tokens
        .iter()
        .filter(|token| !is_supported_token(token))
        .cloned()
        .collect();
    NativeStyle {
        tokens,
        unsupported,
    }
}

#[must_use]
pub fn translate_projection(projection: &StyleProjection, theme: ShadcnTheme) -> NativeStyle {
    translate_style(projection.tokens.clone(), theme)
}

fn is_supported_token(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    if let Some((prefix, value)) = token.split_once(':') {
        if matches!(
            prefix,
            "hover"
                | "focus"
                | "focus-visible"
                | "active"
                | "disabled"
                | "enabled"
                | "checked"
                | "selected"
                | "open"
                | "closed"
                | "group-hover"
                | "group-focus"
                | "peer-checked"
        ) || prefix.starts_with("data-")
            || prefix.starts_with("aria-")
        {
            return is_supported_token(value);
        }
        return false;
    }

    matches!(
        token,
        "block"
            | "inline"
            | "inline-block"
            | "inline-flex"
            | "flex"
            | "grid"
            | "hidden"
            | "contents"
            | "table"
            | "relative"
            | "absolute"
            | "fixed"
            | "sticky"
            | "static"
            | "inset-0"
            | "inset-auto"
            | "shrink"
            | "shrink-0"
            | "grow"
            | "grow-0"
            | "items-start"
            | "items-center"
            | "items-end"
            | "items-baseline"
            | "items-stretch"
            | "justify-start"
            | "justify-center"
            | "justify-end"
            | "justify-between"
            | "justify-around"
            | "justify-evenly"
            | "justify-stretch"
            | "content-start"
            | "content-center"
            | "content-end"
            | "self-start"
            | "self-center"
            | "self-end"
            | "self-stretch"
            | "flex-row"
            | "flex-col"
            | "flex-wrap"
            | "flex-nowrap"
            | "border"
            | "border-0"
            | "border-2"
            | "border-4"
            | "border-8"
            | "bg-transparent"
            | "bg-clip-padding"
            | "text-left"
            | "text-center"
            | "text-right"
            | "text-sm"
            | "text-xs"
            | "text-base"
            | "font-normal"
            | "font-medium"
            | "font-semibold"
            | "font-bold"
            | "whitespace-nowrap"
            | "whitespace-normal"
            | "truncate"
            | "overflow-hidden"
            | "overflow-auto"
            | "overflow-x-hidden"
            | "overflow-y-auto"
            | "select-none"
            | "select-text"
            | "outline-none"
            | "ring"
            | "ring-0"
            | "ring-1"
            | "ring-2"
            | "ring-3"
            | "ring-4"
            | "shadow-none"
            | "shadow-xs"
            | "shadow-sm"
            | "shadow-md"
            | "shadow-lg"
            | "transition"
            | "transition-all"
            | "transition-colors"
            | "transition-opacity"
            | "transition-transform"
            | "duration-75"
            | "duration-100"
            | "duration-150"
            | "duration-200"
            | "duration-300"
            | "ease-in"
            | "ease-out"
            | "ease-in-out"
            | "opacity-0"
            | "opacity-50"
            | "opacity-75"
            | "opacity-100"
            | "pointer-events-none"
            | "pointer-events-auto"
            | "cursor-pointer"
            | "cursor-default"
            | "cursor-not-allowed"
            | "sr-only"
            | "not-sr-only"
            | "appearance-none"
            | "transform"
            | "origin-center"
            | "underline"
            | "underline-offset-4"
            | "z-50"
            | "z-[100]"
            | "group/button"
            | "group/toggle"
            | "bg-background"
            | "bg-primary"
            | "bg-primary/80"
            | "bg-secondary"
            | "bg-secondary/80"
            | "bg-destructive/10"
            | "bg-destructive/20"
            | "bg-destructive/30"
            | "bg-muted"
            | "bg-input/30"
            | "bg-input/50"
            | "bg-input/70"
            | "text-primary-foreground"
            | "text-secondary-foreground"
            | "text-foreground"
            | "text-muted-foreground"
            | "text-destructive"
            | "text-primary"
            | "border-transparent"
            | "border-border"
            | "border-input"
            | "border-ring"
            | "border-destructive/40"
            | "ring-ring/50"
            | "ring-destructive/20"
            | "ring-destructive/40"
            | "bg-accent"
            | "text-accent-foreground"
    ) || has_supported_prefix(token)
}

fn has_supported_prefix(token: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "gap-",
        "space-x-",
        "space-y-",
        "p-",
        "px-",
        "py-",
        "pt-",
        "pr-",
        "pb-",
        "pl-",
        "m-",
        "mx-",
        "my-",
        "mt-",
        "mr-",
        "mb-",
        "ml-",
        "inset-",
        "top-",
        "right-",
        "bottom-",
        "left-",
        "w-",
        "min-w-",
        "max-w-",
        "h-",
        "min-h-",
        "max-h-",
        "size-",
        "basis-",
        "rounded-",
        "border-",
        "bg-",
        "text-",
        "font-",
        "leading-",
        "tracking-",
        "shadow-",
        "opacity-",
        "duration-",
        "delay-",
        "ease-",
        "ring-",
        "translate-",
        "scale-",
        "rotate-",
        "fill-",
        "stroke-",
        "overflow-",
        "cursor-",
        "z-",
    ];
    PREFIXES
        .iter()
        .any(|prefix| token.starts_with(prefix) && token.len() > prefix.len())
}
