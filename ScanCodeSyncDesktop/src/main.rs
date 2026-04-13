#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{panic, time::{Duration, Instant}};
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

    panic::set_hook(Box::new(|a| {
        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("Program has crashed")
            .set_description(format!("an error occurred:\n{}", a))
            .show();
    }));

    let version = env!("CARGO_PKG_VERSION");

    if !cfg!(debug_assertions){
    rfd::MessageDialog::new().set_title("Beta Version")
        .set_description(format!("Warning, you are on version \n{}\nThis version is not feature complete.", version))
        .show();
    }


    let mut state = State::default();
    // let a = ffmpeg_sidecar::download::download_ffmpeg_package("https://www.gyan.dev/ffmpeg/builds/ffmpeg-git-full.7z", get_project_dir().unwrap().cache_dir());

    match ffmpeg_sidecar::download::auto_download() {
        Ok(_) => {},
        Err(a) => {
            println!("{:?}", a);
            rfd::MessageDialog::new().set_title("FFMPEG failed to download").
                set_description("please consider downloading ffmpeg yourself:\nhttps://www.google.com/search?q=how+to+install+ffmped+and+add+to+path")
                .show();
        },
    };
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
        if time_10000ms.elapsed() > Duration::from_millis(5000) {
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
