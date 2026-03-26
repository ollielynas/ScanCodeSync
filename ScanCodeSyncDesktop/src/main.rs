use macroquad::prelude::*;
use egui_macroquad::egui;


pub mod val_hold;
pub mod state;
pub mod main_ui;
pub mod task;

#[macroquad::main("egui with macroquad")]
async fn main() {

    ffmpeg_sidecar::download::auto_download().unwrap();

    loop {
        clear_background(WHITE);

        // Process keys, mouse etc.

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("egui ❤ macroquad")
                .show(egui_ctx, |ui| {
                    ui.label("Test");
                });
        });

        // Draw things before egui

        egui_macroquad::draw();

        // Draw things after egui

        next_frame().await;
    }
}
