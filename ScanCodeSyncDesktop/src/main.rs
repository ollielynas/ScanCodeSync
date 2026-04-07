use std::time::{Duration, Instant};

use macroquad::{prelude::*, ui::widgets::Window, window};
use egui_macroquad::egui;
use rfd::MessageDialogResult;

use crate::{ main_ui::render_state, state::State, tasks::task_builders::{build_init_task, build_install_exiftools_task, build_install_magick_task, build_update_input_files_list_task}, util::{get_project_dir, is_magick_installed}};
use crate::util::window_conf;

pub mod val_hold;
pub mod state;
pub mod main_ui;
pub mod task;
pub mod tasks;
pub mod data;
pub mod util;
pub mod macros;

#[macroquad::main(window_conf)]
async fn main() {

    let version = env!("CARGO_PKG_VERSION");

    rfd::MessageDialog::new().set_title("Beta Version")
        .set_description(format!("Warning, you are on version \n{}\nThis version is not feature complete.", version))
        .show();



    let mut state = State::default();
    // let a = ffmpeg_sidecar::download::download_ffmpeg_package("https://www.gyan.dev/ffmpeg/builds/ffmpeg-git-full.7z", get_project_dir().unwrap().cache_dir());

    ffmpeg_sidecar::download::auto_download().unwrap();
    match exiftool::ExifTool::new() {
        Ok(_) => {println!("exiftool is installed")},
        Err(e) => {
            println!("{e:?}");
            state.add_task(build_install_exiftools_task());
        },
    }
    if !is_magick_installed() {
        state.add_task(build_install_magick_task());
    }


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
