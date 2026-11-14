//! Horizon Jade: original console-inspired colour system for the Android Old School preset.
//! The values are Horizon design decisions, not historical Xbox 360 colours.
//! Unlike Liquid Glass this theme uses opaque surfaces and does not depend on refraction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    pub background: u32,
    pub gradient_start: u32,
    pub gradient_mid: u32,
    pub gradient_end: u32,
    pub surface: u32,
    pub surface_dim: u32,
    pub surface_high: u32,
    pub selected: u32,
    pub header: u32,
    pub sidebar: u32,
    pub border: u32,
    pub focus: u32,
    pub accent: u32,
    pub on_accent: u32,
    pub text: u32,
    pub muted: u32,
    pub success: u32,
    pub danger: u32,
}

impl Palette {
    /// Dark is the main presentation; light keeps the same spatial hierarchy.
    pub const fn for_appearance(light: bool) -> Self {
        if light {
            Self {
                background: 0xF4F6F2,
                gradient_start: 0xEEF5ED,
                gradient_mid: 0xE8EFE7,
                gradient_end: 0xDDE7DF,
                surface: 0xE8ECE7,
                surface_dim: 0xDDE7DF,
                surface_high: 0xFCFDFB,
                selected: 0xCAE7D2,
                header: 0xFCFDFB,
                sidebar: 0xE8ECE7,
                border: 0xB6BDB7,
                focus: 0x125D2D,
                accent: 0x176B35,
                on_accent: 0xFFFFFF,
                text: 0x171B18,
                muted: 0x5C645D,
                success: 0x216A37,
                danger: 0xB3261E,
            }
        } else {
            Self {
                background: 0x141816,
                gradient_start: 0x203028,
                gradient_mid: 0x1B2920,
                gradient_end: 0x101713,
                surface: 0x1D2420,
                surface_dim: 0x19211C,
                surface_high: 0x28302B,
                selected: 0x244E32,
                header: 0x1D2420,
                sidebar: 0x18211B,
                border: 0x4C5850,
                focus: 0x87DB96,
                accent: 0x66C77A,
                on_accent: 0x0E2516,
                text: 0xF4F7F3,
                muted: 0xBBC6BD,
                success: 0x70D28A,
                danger: 0xFF8B82,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Palette;
    fn luminance(rgb: u32) -> f32 {
        let c = |shift| {
            let x = ((rgb >> shift) & 255_u32) as f32 / 255.;
            if x <= 0.04045 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) }
        };
        0.2126 * c(16) + 0.7152 * c(8) + 0.0722 * c(0)
    }
    fn contrast(a: u32, b: u32) -> f32 {
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    #[test]
    fn jade_is_legible_in_both_appearances() {
        for light in [true, false] {
            let p = Palette::for_appearance(light);
            for ground in [p.background, p.surface, p.surface_high, p.header] {
                assert!(contrast(p.text, ground) >= 4.5, "text against {ground:06X}");
                assert!(contrast(p.muted, ground) >= 4.5, "secondary against {ground:06X}");
            }
            assert!(contrast(p.accent, p.on_accent) >= 4.5, "primary action");
            assert!(contrast(p.focus, p.background) >= 3.0, "focus ring");
        }
    }

    #[test]
    fn jade_is_not_the_liquid_glass_colour_family() {
        let dark = Palette::for_appearance(false);
        assert_eq!(dark.background, 0x141816);
        assert_eq!(dark.surface, 0x1D2420);
        assert_eq!(dark.accent, 0x66C77A);
    }
}
