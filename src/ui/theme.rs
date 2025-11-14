use eframe::egui::{self, Color32, FontFamily, FontId, TextStyle, Visuals};

pub fn apply_theme(ctx: &egui::Context, dark_mode: bool) {
    let accent = if dark_mode {
        Color32::from_rgb(96, 170, 255)
    } else {
        Color32::from_rgb(45, 110, 230)
    };

    let mut visuals = if dark_mode {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    visuals.hyperlink_color = accent;
    visuals.selection.bg_fill = accent.linear_multiply(if dark_mode { 0.65 } else { 0.8 });
    visuals.selection.stroke.color = accent;
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(12.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    style.spacing.tooltip_width = 360.0;
    style.interaction.tooltip_delay = 0.15;

    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(24.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Name("Title".into()),
        FontId::new(20.0, FontFamily::Proportional),
    );
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(16.0, FontFamily::Proportional));
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(15.0, FontFamily::Monospace),
    );
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(16.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(13.0, FontFamily::Proportional),
    );

    ctx.set_style(style);
}
