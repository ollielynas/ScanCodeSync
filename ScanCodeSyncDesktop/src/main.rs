use std::time::{Duration, Instant};

use macroquad::prelude::*;
use egui_macroquad::egui;

use crate::{main_ui::render_state, state::State, tasks::task_builders::{build_init_task, build_update_input_files_list_task}};
use crate::util::window_conf;

pub mod val_hold;
pub mod state;
pub mod main_ui;
pub mod task;
pub mod tasks;
pub mod data;
pub mod util;

#[macroquad::main(window_conf)]
async fn main() {



    ffmpeg_sidecar::download::auto_download().unwrap();

    let mut state = State::default();

    state.add_task(build_init_task());

    state.update_tasks();

    let mut time_500ms = Instant::now();
    let mut time_10000ms = Instant::now();

    loop {
        clear_background(WHITE);

        if time_500ms.elapsed() > Duration::from_millis(500) {
            time_500ms = Instant::now();
            state.restore_dropped_variables();
            state.update_tasks();
        }
        if time_10000ms.elapsed() > Duration::from_millis(10000) {
            time_10000ms = Instant::now();
            let _ = state.add_task_without_duplicate(build_update_input_files_list_task());
        }





        // Process keys, mouse etc.

        render_state(&mut state);

        // Draw things before egui

        egui_macroquad::draw();

        // Draw things after egui

        next_frame().await;
    }
}
