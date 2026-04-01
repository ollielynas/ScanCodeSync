use std::time::{Duration, Instant};

use macroquad::prelude::*;
use egui_macroquad::egui;

use crate::{main_ui::render_state, state::State, tasks::task_builders::{build_init_task, build_update_input_files_list_task}};


pub mod val_hold;
pub mod state;
pub mod main_ui;
pub mod task;
pub mod tasks;
pub mod util;

#[macroquad::main("egui with macroquad")]
async fn main() {

    ffmpeg_sidecar::download::auto_download().unwrap();

    let mut state = State::default();

    state.add_task(build_init_task());
    state.add_task(build_update_input_files_list_task());

    state.update_tasks();

    let mut time = Instant::now();

    loop {
        clear_background(WHITE);

        if time.elapsed() > Duration::from_millis(500) {
            time = Instant::now();
            state.restore_dropped_variables();
            state.update_tasks();
        }

        // Process keys, mouse etc.

        render_state(&mut state);

        // Draw things before egui

        egui_macroquad::draw();

        // Draw things after egui

        next_frame().await;
    }
}
