use eframe::egui::{self, Color32, FontFamily, FontId, Stroke, TextStyle};

#[derive(Clone, Copy)]
pub struct Palette {
    pub background: Color32,
    pub surface: Color32,
    pub surface_alt: Color32,
    pub card: Color32,
    pub card_hover: Color32,
    pub border: Color32,
    pub border_strong: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub positive: Color32,
    pub positive_soft: Color32,
    pub danger: Color32,
    pub danger_soft: Color32,
    pub warning: Color32,
    pub gold: Color32,
    pub silver: Color32,
}

pub fn palette() -> Palette {
    Palette {
        background: Color32::TRANSPARENT,
        surface: Color32::from_rgba_unmultiplied(12, 26, 43, 245),
        surface_alt: Color32::from_rgb(18, 37, 57),
        card: Color32::from_rgb(12, 29, 47),
        card_hover: Color32::from_rgb(30, 60, 82),
        border: Color32::from_rgb(51, 79, 101),
        border_strong: Color32::from_rgb(123, 168, 191),
        text_primary: Color32::from_rgb(235, 246, 251),
        text_secondary: Color32::from_rgb(169, 193, 209),
        accent: Color32::from_rgb(116, 222, 242),
        accent_soft: Color32::from_rgb(21, 60, 79),
        positive: Color32::from_rgb(136, 225, 180),
        positive_soft: Color32::from_rgb(21, 54, 47),
        danger: Color32::from_rgb(255, 167, 156),
        danger_soft: Color32::from_rgb(65, 34, 46),
        warning: Color32::from_rgb(240, 205, 132),
        gold: Color32::from_rgb(240, 205, 132),
        silver: Color32::from_rgb(197, 220, 233),
    }
}

pub fn apply_theme(ctx: &egui::Context) {
    let colors = palette();
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.animation_time = 0.14;
    style.spacing.item_spacing = egui::vec2(10.0, 9.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    style.spacing.interact_size = egui::vec2(38.0, 30.0);
    style.spacing.text_edit_width = 240.0;

    style.visuals.override_text_color = Some(colors.text_primary);
    style.visuals.panel_fill = colors.background;
    style.visuals.window_fill = colors.surface;
    style.visuals.window_stroke = Stroke::new(1.0, colors.border);
    style.visuals.faint_bg_color = colors.background;
    style.visuals.extreme_bg_color = colors.surface_alt;
    style.visuals.code_bg_color = colors.surface_alt;
    style.visuals.hyperlink_color = colors.accent;
    style.visuals.warn_fg_color = colors.warning;
    style.visuals.error_fg_color = colors.danger;
    style.visuals.selection.bg_fill = colors.accent_soft;
    style.visuals.selection.stroke = Stroke::new(1.0, colors.accent);
    style.visuals.window_rounding = 10.0.into();
    style.visuals.menu_rounding = 8.0.into();
    style.visuals.window_shadow = egui::epaint::Shadow {
        offset: egui::vec2(0.0, 8.0),
        blur: 22.0,
        spread: 2.0,
        color: Color32::from_black_alpha(85),
    };

    style.visuals.widgets.noninteractive.rounding = 12.0.into();
    style.visuals.widgets.noninteractive.bg_fill = colors.surface;
    style.visuals.widgets.noninteractive.weak_bg_fill = colors.surface;
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, colors.border);
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors.text_primary);

    style.visuals.widgets.inactive.rounding = 12.0.into();
    style.visuals.widgets.inactive.bg_fill = colors.card;
    style.visuals.widgets.inactive.weak_bg_fill = colors.card;
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, colors.border);
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors.text_primary);

    style.visuals.widgets.hovered.rounding = 12.0.into();
    style.visuals.widgets.hovered.bg_fill = colors.card_hover;
    style.visuals.widgets.hovered.weak_bg_fill = colors.card_hover;
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, colors.border_strong);
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, colors.text_primary);

    style.visuals.widgets.active.rounding = 12.0.into();
    style.visuals.widgets.active.bg_fill = colors.accent_soft;
    style.visuals.widgets.active.weak_bg_fill = colors.accent_soft;
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, colors.accent);
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, colors.text_primary);

    style.visuals.widgets.open.rounding = 12.0.into();
    style.visuals.widgets.open.bg_fill = colors.surface_alt;
    style.visuals.widgets.open.weak_bg_fill = colors.surface_alt;
    style.visuals.widgets.open.bg_stroke = Stroke::new(1.0, colors.border_strong);
    style.visuals.widgets.open.fg_stroke = Stroke::new(1.0, colors.text_primary);

    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(24.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Name("Title".into()),
        FontId::new(18.0, FontFamily::Proportional),
    );
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(15.0, FontFamily::Proportional));
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(14.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(12.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(15.0, FontFamily::Monospace),
    );

    ctx.set_style(style);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luminance(color: [f32; 3]) -> f32 {
        let linear = color.map(|value| {
            let channel = value / 255.0;
            if channel <= 0.04045 {
                channel / 12.92
            } else {
                ((channel + 0.055) / 1.055).powf(2.4)
            }
        });
        linear[0] * 0.2126 + linear[1] * 0.7152 + linear[2] * 0.0722
    }

    fn contrast(text: Color32, fill: Color32, background: f32) -> f32 {
        let [r, g, b, a] = fill.to_srgba_unmultiplied();
        let alpha = f32::from(a) / 255.0;
        let surface =
            [r, g, b].map(|channel| f32::from(channel) * alpha + background * (1.0 - alpha));
        let [r, g, b, _] = text.to_srgba_unmultiplied();
        let foreground = luminance([r, g, b].map(f32::from));
        let background = luminance(surface);
        (foreground.max(background) + 0.05) / (foreground.min(background) + 0.05)
    }

    #[test]
    fn theme_text_remains_readable_over_light_and_dark_scenery() {
        let colors = palette();
        // Je contrôle les deux extrêmes du décor derrière les panneaux translucides.
        for backdrop in [0.0, 255.0] {
            for fill in [
                colors.surface,
                colors.surface_alt,
                colors.card,
                colors.card_hover,
            ] {
                for text in [colors.text_primary, colors.text_secondary, colors.silver] {
                    assert!(
                        contrast(text, fill, backdrop) >= 4.5,
                        "Contraste insuffisant : {text:?} / {fill:?}"
                    );
                }
            }
            for (text, fill) in [
                (colors.accent, colors.accent_soft),
                (colors.positive, colors.positive_soft),
                (colors.danger, colors.danger_soft),
                (colors.gold, colors.surface_alt),
                (colors.card, colors.accent),
            ] {
                assert!(contrast(text, fill, backdrop) >= 4.5);
            }
        }
    }

    #[test]
    fn theme_applies_selection_and_feedback_consistently() {
        let ctx = egui::Context::default();
        apply_theme(&ctx);
        let colors = palette();
        let style = ctx.style();
        assert_eq!(style.visuals.selection.stroke.color, colors.accent);
        assert_eq!(style.visuals.error_fg_color, colors.danger);
        assert_eq!(style.visuals.widgets.hovered.bg_fill, colors.card_hover);
        assert!(style.spacing.interact_size.y >= 30.0);
    }
}
