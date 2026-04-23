use catppuccin_egui::Theme;
use egui_macroquad::egui::Color32;
use macroquad::{color::WHITE, window::{clear_background, next_frame}};

use crate::dbp;



pub async fn setup_look() {
    clear_background(WHITE);
    egui_macroquad::ui(|ctx| {

        ctx.send_viewport_cmd(egui_macroquad::egui::ViewportCommand::Focus);
        ctx.set_theme(egui_macroquad::egui::Theme::Light);

        match ctx.system_theme() {
            Some(theme) => {
                ctx.set_theme(theme);
            },
            None => {
                dbp!("no system theme detected")
            },
        };
        ctx.set_pixels_per_point(ctx.native_pixels_per_point().unwrap_or(1.5));
    egui_macroquad::egui::CentralPanel::default()
        .show(ctx, |ui|{
            ui.label("loading...")
        });
    });
    egui_macroquad::draw();
    next_frame().await;
}


pub const DUSK: Theme = Theme {
    rosewater: Color32::from_rgb(210, 105, 85),
    flamingo:  Color32::from_rgb(215,  90, 90),
    pink:      Color32::from_rgb(210,  80, 160),
    mauve:     Color32::from_rgb(145,  65, 200),
    red:       Color32::from_rgb(195,  30,  50),
    maroon:    Color32::from_rgb(185,  55,  70),
    peach:     Color32::from_rgb(220,  95,  20),
    yellow:    Color32::from_rgb(195, 140,  15),
    green:     Color32::from_rgb( 55, 145,  35),
    teal:      Color32::from_rgb( 20, 130, 140),
    sky:       Color32::from_rgb( 10, 150, 210),
    sapphire:  Color32::from_rgb( 25, 140, 170),
    blue:      Color32::from_rgb( 35,  95, 220),
    lavender:  Color32::from_rgb(105, 120, 235),
    text:      Color32::from_rgb( 58,  52,  35),
    subtext1:  Color32::from_rgb( 80,  73,  55),
    subtext0:  Color32::from_rgb(100,  93,  75),
    overlay2:  Color32::from_rgb(125, 118, 100),
    overlay1:  Color32::from_rgb(145, 138, 120),
    overlay0:  Color32::from_rgb(165, 158, 140),
    surface2:  Color32::from_rgb(193, 187, 172),
    surface1:  Color32::from_rgb(207, 201, 187),
    surface0:  Color32::from_rgb(220, 215, 202),
    base:      Color32::from_rgb(250, 247, 240),
    mantle:    Color32::from_rgb(242, 238, 229),
    crust:     Color32::from_rgb(233, 228, 217),
};

pub const VOID: Theme = Theme {
    rosewater: Color32::from_rgb(255, 200, 190),
    flamingo:  Color32::from_rgb(255, 180, 180),
    pink:      Color32::from_rgb(255, 160, 220),
    mauve:     Color32::from_rgb(210, 150, 255),
    red:       Color32::from_rgb(255, 120, 145),
    maroon:    Color32::from_rgb(245, 145, 158),
    peach:     Color32::from_rgb(255, 175, 120),
    yellow:    Color32::from_rgb(255, 220, 140),
    green:     Color32::from_rgb(145, 235, 130),
    teal:      Color32::from_rgb(120, 230, 215),
    sky:       Color32::from_rgb(120, 215, 245),
    sapphire:  Color32::from_rgb(100, 200, 245),
    blue:      Color32::from_rgb(120, 175, 255),
    lavender:  Color32::from_rgb(175, 185, 255),
    text:      Color32::from_rgb(210, 215, 240),
    subtext1:  Color32::from_rgb(178, 185, 215),
    subtext0:  Color32::from_rgb(155, 162, 195),
    overlay2:  Color32::from_rgb(122, 126, 158),
    overlay1:  Color32::from_rgb( 98, 102, 132),
    overlay0:  Color32::from_rgb( 75,  78, 105),
    surface2:  Color32::from_rgb( 48,  48,  76),
    surface1:  Color32::from_rgb( 35,  35,  58),
    surface0:  Color32::from_rgb( 22,  22,  40),
    base:      Color32::from_rgb( 10,  10,  20),
    mantle:    Color32::from_rgb(  8,   8,  16),
    crust:     Color32::from_rgb(  5,   5,  12),
};

pub const PARCHMENT: Theme = Theme {
    rosewater: Color32::from_rgb(214, 122, 100),
    flamingo:  Color32::from_rgb(208, 104, 104),
    pink:      Color32::from_rgb(198,  92, 148),
    mauve:     Color32::from_rgb(135,  68, 152),
    red:       Color32::from_rgb(182,  28,  46),
    maroon:    Color32::from_rgb(158,  50,  60),
    peach:     Color32::from_rgb(208,  98,  22),
    yellow:    Color32::from_rgb(188, 142,  18),
    green:     Color32::from_rgb( 68, 138,  56),
    teal:      Color32::from_rgb( 32, 130, 112),
    sky:       Color32::from_rgb( 72, 150, 120),
    sapphire:  Color32::from_rgb( 48, 128, 108),
    blue:      Color32::from_rgb( 88, 115,  88),
    lavender:  Color32::from_rgb(148, 132, 168),
    text:      Color32::from_rgb( 46,  40,  32),
    subtext1:  Color32::from_rgb( 68,  61,  52),
    subtext0:  Color32::from_rgb( 92,  84,  74),
    overlay2:  Color32::from_rgb(116, 110,  99),
    overlay1:  Color32::from_rgb(140, 133, 122),
    overlay0:  Color32::from_rgb(163, 158, 146),
    surface2:  Color32::from_rgb(190, 186, 175),
    surface1:  Color32::from_rgb(206, 202, 192),
    surface0:  Color32::from_rgb(220, 216, 207),
    base:      Color32::from_rgb(250, 247, 241),
    mantle:    Color32::from_rgb(242, 238, 231),
    crust:     Color32::from_rgb(232, 228, 220),
};

pub const COAL: Theme = Theme {
    rosewater: Color32::from_rgb(242, 188, 168),
    flamingo:  Color32::from_rgb(236, 168, 158),
    pink:      Color32::from_rgb(220, 150, 190),
    mauve:     Color32::from_rgb(195, 142, 228),
    red:       Color32::from_rgb(232,  98, 112),
    maroon:    Color32::from_rgb(225, 125, 136),
    peach:     Color32::from_rgb(242, 162,  95),
    yellow:    Color32::from_rgb(238, 206, 115),
    green:     Color32::from_rgb(138, 208, 116),
    teal:      Color32::from_rgb(112, 200, 175),
    sky:       Color32::from_rgb(125, 210, 185),
    sapphire:  Color32::from_rgb(105, 190, 168),
    blue:      Color32::from_rgb(155, 180, 135),
    lavender:  Color32::from_rgb(192, 165, 215),
    text:      Color32::from_rgb(230, 220, 205),
    subtext1:  Color32::from_rgb(206, 195, 180),
    subtext0:  Color32::from_rgb(180, 170, 154),
    overlay2:  Color32::from_rgb(146, 136, 120),
    overlay1:  Color32::from_rgb(115, 106,  92),
    overlay0:  Color32::from_rgb( 88,  80,  68),
    surface2:  Color32::from_rgb( 62,  56,  46),
    surface1:  Color32::from_rgb( 48,  42,  34),
    surface0:  Color32::from_rgb( 34,  30,  22),
    base:      Color32::from_rgb( 20,  18,  12),
    mantle:    Color32::from_rgb( 16,  14,  10),
    crust:     Color32::from_rgb( 12,  10,   7),
};
