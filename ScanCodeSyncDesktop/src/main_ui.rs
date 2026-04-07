use crate::{state::State, tasks::task_builders::{build_import_media_task_folders, build_process_new_files_task, user_accessible_tasks}, util::truncate_front};
use egui_macroquad::egui;
use macroquad::window::screen_width;
use open;



pub fn render_state(state: &mut State) {
    egui_macroquad::ui(|egui_ctx| {
        egui_ctx.set_pixels_per_point(1.3);
        catppuccin_egui::set_theme(egui_ctx, catppuccin_egui::LATTE);
        // println!("{}",screen_width());
        egui::TopBottomPanel::top("top")

            .show(egui_ctx, |ui|{

                let mut add_task_id = 0;
                ui.horizontal(|ui| {
                ui.menu_button("> Tasks", |ui| {
                    for task in &mut state.user_available_tasks {
                        if ui.button(task.get_name()).on_hover_text("task description").clicked() {
                            add_task_id = task.get_id();
                            ui.close_menu();
                        }
                    }
                });

                if ui.button("Import Media").clicked() {
                    let _ = state.add_task_without_duplicate(build_import_media_task_folders());
                }
                if ui.button(format!("Process {} from input", state.new_files.to_string())).clicked() {
                    let _ = state.add_task_without_duplicate(build_process_new_files_task());
                }
                });

                if add_task_id != 0 {
                    let task = state.user_available_tasks.drain(..).find(|x| x.get_id() == add_task_id);
                    state.user_available_tasks = user_accessible_tasks();
                    if let Some(t) = task {
                        state.add_task(t);
                    }

                }
            });

        egui::SidePanel::left("left panel (state)")

            .show(egui_ctx, |ui| {
                egui::Grid::new("state").show(ui, |ui| {

                    ui.strong("Value");
                    ui.strong("State");
                    ui.strong("Task ID");
                    ui.end_row();

                ui.label("input folder");
                if ui.link(truncate_front(state.input_folder.to_string(), 30)).clicked() {
                    let _ = open::that(state.input_folder.to_string());
                }
                ui.label(state.input_folder.used_by_string());
                ui.end_row();

                ui.label("processing folder");
                if ui.link(truncate_front(state.unsorted_folder.to_string(), 30)).clicked() {
                    let _ = open::that(state.unsorted_folder.to_string());
                }
                ui.label(state.unsorted_folder.used_by_string());
                ui.end_row();

                ui.label("output folder");
                if ui.link(truncate_front(state.output_folder.to_string(),30)).clicked() {
                    let _ = open::that(state.output_folder.to_string());
                };
                ui.label(state.output_folder.used_by_string());

                ui.end_row();

                ui.label("no. unprocessed files:");
                ui.label(state.new_files.to_string());
                ui.label(state.new_files.used_by_string());
                ui.end_row();

                ui.label("no. processed, ready to be sorted files:");
                ui.label(state.unsorted_files.to_string());
                ui.label(state.unsorted_files.used_by_string());
                ui.end_row();

                ui.label("timeline:");
                ui.label(state.timeline.to_string());
                ui.label(state.timeline.used_by_string());
                ui.end_row();

                });
            });
        egui::CentralPanel::default()
            .show(egui_ctx, |ui| {
                egui::Grid::new("tasks").show(ui, |ui| {
                    ui.label("Task");
                    ui.label("Progress");
                    ui.label("Elapsed");
                    ui.label("State");
                    ui.label("ID");
                    ui.end_row();
                    for t in &mut state.task_list {
                        if t.silent() {continue;}
                        ui.label(t.get_name());
                        if t.is_finished() {
                            ui.label("Finished");
                            ui.label("100%");
                            ui.label("");

                        }else if t.is_running() {
                            let time_text = t.get_progress().get_elapsed().map_or("--:--".to_string(), |x|
                                {format!("{:02}:{:02}", x.as_secs() / 60, x.as_secs() % 60)});
                            ui.label(format!("{:.1}%", t.get_progress().get_percent()));
                            ui.label(time_text);
                            ui.label(t.get_progress().get_item().to_string());
                        }else {
                            ui.label(format!("Waiting"));
                            ui.label(format!("--:--"));
                            ui.label(format!("{:.1}%", 0));

                            ui.label(t.get_progress().get_item().to_string());
                        }
                        ui.label((t.get_id()%999).to_string());
                        ui.end_row();
                    }
                });
            });
    });

}
