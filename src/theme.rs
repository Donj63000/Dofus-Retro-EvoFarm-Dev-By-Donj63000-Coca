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
}

pub fn palette() -> Palette {
    Palette {
        background: Color32::from_rgba_unmultiplied(13, 16, 20, 0),
        surface: Color32::from_rgba_unmultiplied(21, 25, 30, 228),
        surface_alt: Color32::from_rgba_unmultiplied(29, 34, 40, 214),
        card: Color32::from_rgba_unmultiplied(32, 38, 45, 220),
        card_hover: Color32::from_rgba_unmultiplied(38, 45, 53, 232),
        border: Color32::from_rgb(62, 72, 84),
        border_strong: Color32::from_rgb(100, 112, 126),
        text_primary: Color32::from_rgb(236, 239, 242),
        text_secondary: Color32::from_rgb(154, 163, 174),
        accent: Color32::from_rgb(231, 173, 86),
        accent_soft: Color32::from_rgba_unmultiplied(78, 59, 28, 214),
        positive: Color32::from_rgb(118, 191, 132),
        positive_soft: Color32::from_rgba_unmultiplied(33, 63, 42, 214),
        danger: Color32::from_rgb(215, 108, 98),
        danger_soft: Color32::from_rgba_unmultiplied(77, 39, 36, 214),
        warning: Color32::from_rgb(219, 173, 84),
    }
}

pub fn apply_theme(ctx: &egui::Context) {
    let colors = palette();
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.animation_time = 0.10;
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
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
    style.visuals.window_rounding = 14.0.into();
    style.visuals.menu_rounding = 12.0.into();

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
