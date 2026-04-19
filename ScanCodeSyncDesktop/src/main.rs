#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{panic, time::{Duration, Instant}};
use ffmpeg_sidecar::{command::ffmpeg_is_installed};
use macroquad::prelude::*;
use rfd::MessageDialogResult;

use crate::{ main_ui::render_state, state::State, tasks::task_builders::{build_init_task, build_install_exiftools_task, build_install_ffmpeg_task, build_install_magick_task, build_update_input_files_list_task}, util::{get_project_dir, is_magick_installed}};
use crate::util::window_conf;

pub mod val_hold;
pub mod state;
pub mod main_ui;
pub mod task;
pub mod tasks;
pub mod data;
pub mod util;
pub mod macros;
pub mod command_pool;
pub mod updater;


#[macroquad::main(window_conf)]
async fn main() {

    panic::set_hook(Box::new(|a| {
        let panic_text = format!("an error occurred:\n{}", a);

        #[cfg(target_os = "macos")]
        {dispatch::Queue::main().exec_sync(||rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("Program has crashed")
            .set_description(panic_text)
            .show())};

        #[cfg(not(target_os = "macos"))]
        {rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("Program has crashed")
            .set_description(panic_text)
            .show()};
    }));


    std::thread::spawn(|| {
        match updater::check_for_update() {
            Ok(updater::UpdateStatus::UpdateAvailable { version, notes }) => {
                println!("Update available: v{}\n{}", version, notes);
                // In a GUI app: show a dialog asking the user to update
                // In a CLI app: print a notice and optionally auto-update
                #[cfg(target_os = "macos")]
                {if dispatch::Queue::main().exec_sync(||rfd::MessageDialog::new()
                    .set_title("New Version Available")
                    .set_description(format!("Update available\n{}\n Would You like to update?", notes))
                    .set_buttons(rfd::MessageButtons::YesNo)
                    .show()) == MessageDialogResult::Yes
                {
                    if let Err(e) = updater::download_and_install() {
                        dispatch::Queue::main().exec_sync(||rfd::MessageDialog::new()
                            .set_title("Update Failed")
                            .set_description(e.to_string())
                            .show());
                    }
                }}
                #[cfg(not(target_os = "macos"))]
                {if rfd::MessageDialog::new()
                    .set_title("New Version Available")
                    .set_description(format!("Update available\n{}\n Would You like to update?", notes))
                    .set_buttons(rfd::MessageButtons::YesNo)
                    .show() == MessageDialogResult::Yes
                {
                    if let Err(e) = updater::download_and_install() {
                        rfd::MessageDialog::new()
                            .set_title("Update Failed")
                            .set_description(e.to_string())
                            .show();
                    }
                }}
            }
            Ok(updater::UpdateStatus::PlatformNotSupported) => {
                // No update for this platform yet — silently do nothing
            }
            Ok(updater::UpdateStatus::UpToDate) => {}
            Err(e) => {
                // Never crash the app over a failed update check
                eprintln!("Update check failed: {}", e);
            }
        }
    });


    // if !cfg!(debug_assertions){
    // rfd::MessageDialog::new().set_title("Beta Version")
    //     .set_description(format!("Warning, you are on version \n{}\nThis version is not feature complete.", version))
    //     .show();
    // }
    let mut state = State::default();

    if !ffmpeg_is_installed() {
         state.add_task(build_install_ffmpeg_task());
    }

    match exiftool::ExifTool::new() {
        Ok(_) => {crate::dbp!("exiftool is installed")},
        Err(e) => {
            crate::dbp!("{e:?}");
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
