use eframe::egui::{self, Color32, Rounding, Stroke};

pub(super) const BACKGROUND: Color32 = Color32::from_rgb(12, 18, 27);
pub(super) const PANEL: Color32 = Color32::from_rgb(20, 29, 41);
pub(super) const SURFACE: Color32 = Color32::from_rgb(29, 41, 55);
pub(super) const LINE: Color32 = Color32::from_rgb(49, 66, 82);
pub(super) const TEXT: Color32 = Color32::from_rgb(235, 239, 233);
pub(super) const MUTED: Color32 = Color32::from_rgb(158, 177, 190);
pub(super) const ACCENT: Color32 = Color32::from_rgb(116, 232, 192);
pub(super) const CANVAS: Color32 = Color32::from_rgb(9, 15, 23);
pub(super) const WATER: Color32 = Color32::from_rgb(73, 149, 243);
pub(super) const DIRT: Color32 = Color32::from_rgb(95, 116, 133);
pub(super) const SAND: Color32 = Color32::from_rgb(231, 194, 116);
pub(super) const WOOD: Color32 = Color32::from_rgb(171, 116, 82);
pub(super) const FIRE: Color32 = Color32::from_rgb(255, 145, 91);

pub(super) fn install(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    style.spacing.slider_width = 110.0;
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(TEXT);
    visuals.extreme_bg_color = BACKGROUND;
    visuals.faint_bg_color = SURFACE;
    visuals.selection.bg_fill = Color32::from_rgb(34, 82, 73);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    visuals.hyperlink_color = ACCENT;
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.rounding = Rounding::same(6.0);
        widget.bg_stroke = Stroke::new(1.0, LINE);
        widget.fg_stroke = Stroke::new(1.0, TEXT);
    }
    visuals.widgets.noninteractive.bg_fill = PANEL;
    visuals.widgets.inactive.bg_fill = SURFACE;
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(40, 62, 76);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.active.bg_fill = Color32::from_rgb(37, 83, 75);
    style.visuals = visuals;
    ctx.set_style(style);
}

// Presentation-only tint keeps the simulation renderer's coverage contract intact.
pub(super) fn scene_color(color: Color32) -> Color32 {
    match color {
        Color32::BLACK => DIRT,
        Color32::BLUE => WATER,
        Color32::GOLD => SAND,
        Color32::BROWN => WOOD,
        Color32::LIGHT_RED => FIRE,
        Color32::DARK_RED => Color32::from_rgb(218, 82, 63),
        Color32::LIGHT_GRAY => Color32::from_rgb(180, 194, 206),
        Color32::DARK_GRAY => Color32::from_rgb(101, 117, 136),
        other => other,
    }
}
