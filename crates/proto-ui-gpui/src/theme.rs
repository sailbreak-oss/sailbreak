use crate::protocol::StyleProjection;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorScheme {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorValue {
    pub rgb: u32,
    pub alpha: f32,
}

impl ColorValue {
    #[must_use]
    pub const fn opaque(rgb: u32) -> Self {
        Self { rgb, alpha: 1.0 }
    }

    #[must_use]
    pub const fn transparent() -> Self {
        Self { rgb: 0, alpha: 0.0 }
    }

    #[must_use]
    pub const fn with_alpha(self, alpha: f32) -> Self {
        Self {
            rgb: self.rgb,
            alpha,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShadcnTheme {
    pub scheme: ColorScheme,
    pub background: u32,
    pub foreground: u32,
    pub primary: u32,
    pub primary_foreground: u32,
    pub secondary: u32,
    pub secondary_foreground: u32,
    pub destructive: u32,
    pub border: u32,
    pub input: u32,
    pub muted: u32,
    pub ring: u32,
}

impl ShadcnTheme {
    #[must_use]
    pub const fn light() -> Self {
        Self {
            scheme: ColorScheme::Light,
            background: 0xfafafa,
            foreground: 0x18181b,
            primary: 0x18181b,
            primary_foreground: 0xfafafa,
            secondary: 0xf4f4f5,
            secondary_foreground: 0x18181b,
            destructive: 0xdc2626,
            border: 0xe4e4e7,
            input: 0xe4e4e7,
            muted: 0xf4f4f5,
            ring: 0x18181b,
        }
    }

    #[must_use]
    pub const fn dark() -> Self {
        Self {
            scheme: ColorScheme::Dark,
            background: 0x09090b,
            foreground: 0xfafafa,
            primary: 0xf4f4f5,
            primary_foreground: 0x18181b,
            secondary: 0x27272a,
            secondary_foreground: 0xfafafa,
            destructive: 0x7f1d1d,
            border: 0x27272a,
            input: 0x27272a,
            muted: 0x27272a,
            ring: 0xd4d4d8,
        }
    }

    #[must_use]
    pub fn color(self, role: &str) -> Option<ColorValue> {
        match role {
            "background" => Some(ColorValue::opaque(self.background)),
            "foreground" => Some(ColorValue::opaque(self.foreground)),
            "primary" => Some(ColorValue::opaque(self.primary)),
            "primary-foreground" => Some(ColorValue::opaque(self.primary_foreground)),
            "secondary" => Some(ColorValue::opaque(self.secondary)),
            "secondary-foreground" => Some(ColorValue::opaque(self.secondary_foreground)),
            "destructive" => Some(ColorValue::opaque(self.destructive)),
            "border" => Some(ColorValue::opaque(self.border)),
            "input" => Some(ColorValue::opaque(self.input)),
            "muted" => Some(ColorValue::opaque(self.muted)),
            "ring" => Some(ColorValue::opaque(self.ring)),
            _ => None,
        }
    }
}

impl Default for ShadcnTheme {
    fn default() -> Self {
        Self::dark()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ButtonStyle {
    pub background: ColorValue,
    pub foreground: ColorValue,
    pub border: ColorValue,
    pub height: f32,
    pub padding_x: f32,
    pub gap: f32,
    pub radius: f32,
    pub opacity: f32,
    pub translate_y: f32,
    pub ring: Option<ColorValue>,
    pub ring_width: f32,
    pub underline: bool,
    pub pointer_events_none: bool,
    pub unsupported: Vec<String>,
}

impl ButtonStyle {
    #[must_use]
    pub fn from_projection(projection: &StyleProjection, theme: ShadcnTheme) -> Self {
        Self::from_tokens(&projection.tokens, theme)
    }

    #[must_use]
    pub fn from_tokens(tokens: &[String], theme: ShadcnTheme) -> Self {
        let mut style = Self {
            background: ColorValue::transparent(),
            foreground: theme
                .color("foreground")
                .unwrap_or(ColorValue::opaque(theme.foreground)),
            border: ColorValue::transparent(),
            height: 32.0,
            padding_x: 10.0,
            gap: 6.0,
            radius: 8.0,
            opacity: 1.0,
            translate_y: 0.0,
            ring: None,
            ring_width: 0.0,
            underline: false,
            pointer_events_none: false,
            unsupported: Vec::new(),
        };

        for token in tokens {
            if !apply_token(&mut style, token, theme) {
                style.unsupported.push(token.clone());
            }
        }
        style
    }
}

fn apply_token(style: &mut ButtonStyle, token: &str, theme: ShadcnTheme) -> bool {
    match token {
        "group/button"
        | "inline-flex"
        | "shrink-0"
        | "items-center"
        | "justify-center"
        | "border"
        | "bg-clip-padding"
        | "text-sm"
        | "font-medium"
        | "whitespace-nowrap"
        | "transition-all"
        | "transition-colors"
        | "outline-none"
        | "select-none"
        | "relative"
        | "cursor-default"
        | "w-full"
        | "z-50"
        | "overflow-x-hidden"
        | "overflow-y-auto"
        | "p-1"
        | "shadow-xs"
        | "shadow-md"
        | "duration-150"
        | "pointer-events-none" => {
            if token == "pointer-events-none" {
                style.pointer_events_none = true;
            }
            true
        }
        "rounded-sm" | "rounded-lg" | "rounded-md" | "rounded-[min(var(--radius-md),12px)]" => {
            style.radius = if token == "rounded-sm" { 4.0 } else { 8.0 };
            true
        }
        "h-7" => {
            style.height = 28.0;
            style.gap = 4.0;
            true
        }
        "h-8" => {
            style.height = 32.0;
            true
        }
        "size-8" => {
            style.height = 32.0;
            style.padding_x = 0.0;
            style.gap = 0.0;
            true
        }
        "h-9" => {
            style.height = 36.0;
            true
        }
        "gap-1" => {
            style.gap = 4.0;
            true
        }
        "gap-1.5" => {
            style.gap = 6.0;
            true
        }
        "px-2" => {
            style.padding_x = 8.0;
            true
        }
        "px-2.5" => {
            style.padding_x = 10.0;
            true
        }
        "text-[0.8rem]" | "text-xs" => true,
        "translate-y-px" => {
            style.translate_y = 1.0;
            true
        }
        "opacity-50" => {
            style.opacity = 0.5;
            true
        }
        "underline" => {
            style.underline = true;
            true
        }
        "underline-offset-4" => true,
        "border-transparent" => {
            style.border = ColorValue::transparent();
            true
        }
        "bg-transparent" => {
            style.background = ColorValue::transparent();
            true
        }
        "bg-background" => set_color(&mut style.background, theme, "background", 1.0),
        "border-border" => set_color(&mut style.border, theme, "border", 1.0),
        "border-input" => set_color(&mut style.border, theme, "input", 1.0),
        "border-ring" => set_color(&mut style.border, theme, "ring", 1.0),
        "border-destructive/40" => set_color(&mut style.border, theme, "destructive", 0.4),
        "bg-primary" => set_color(&mut style.background, theme, "primary", 1.0),
        "bg-primary/80" => set_color(&mut style.background, theme, "primary", 0.8),
        "bg-secondary" => set_color(&mut style.background, theme, "secondary", 1.0),
        "bg-secondary/80" => set_color(&mut style.background, theme, "secondary", 0.8),
        "bg-destructive/10" => set_color(&mut style.background, theme, "destructive", 0.1),
        "bg-destructive/20" => set_color(&mut style.background, theme, "destructive", 0.2),
        "bg-destructive/30" => set_color(&mut style.background, theme, "destructive", 0.3),
        "bg-muted" => set_color(&mut style.background, theme, "muted", 1.0),
        "bg-input/30" => set_color(&mut style.background, theme, "input", 0.3),
        "bg-input/50" => set_color(&mut style.background, theme, "input", 0.5),
        "bg-input/70" => set_color(&mut style.background, theme, "input", 0.7),
        "text-primary-foreground" => {
            set_color(&mut style.foreground, theme, "primary-foreground", 1.0)
        }
        "text-secondary-foreground" => {
            set_color(&mut style.foreground, theme, "secondary-foreground", 1.0)
        }
        "text-foreground" => set_color(&mut style.foreground, theme, "foreground", 1.0),
        "text-destructive" => set_color(&mut style.foreground, theme, "destructive", 1.0),
        "text-primary" => set_color(&mut style.foreground, theme, "primary", 1.0),
        "ring-3" => {
            style.ring_width = 3.0;
            true
        }
        "ring-ring/50" => set_ring(style, theme, "ring", 0.5),
        "ring-destructive/20" => set_ring(style, theme, "destructive", 0.2),
        "ring-destructive/40" => set_ring(style, theme, "destructive", 0.4),
        _ => false,
    }
}

fn set_color(target: &mut ColorValue, theme: ShadcnTheme, role: &str, alpha: f32) -> bool {
    let Some(color) = theme.color(role) else {
        return false;
    };
    *target = color.with_alpha(alpha);
    true
}

fn set_ring(style: &mut ButtonStyle, theme: ShadcnTheme, role: &str, alpha: f32) -> bool {
    let Some(color) = theme.color(role) else {
        return false;
    };
    style.ring = Some(color.with_alpha(alpha));
    true
}
