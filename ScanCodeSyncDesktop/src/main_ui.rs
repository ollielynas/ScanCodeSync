use crate::{state::State, tasks::task_builders::{build_import_media_task_folders, build_process_new_files_task, build_process_unsorted_files_task, build_reset_data_task, user_accessible_tasks}, themes, util::truncate_front, val_hold::ValueHolder};
use egui_macroquad::egui::{self, Color32, RichText};
use open;
use palette::named::LIGHTYELLOW;



pub fn render_state(state: &mut State) {

    let kill_tasks: Vec<u64> = vec![];

    egui_macroquad::ui(|egui_ctx| {


        egui::TopBottomPanel::bottom("bottom")

            .show(egui_ctx, |ui| {
                ui.horizontal(|ui| {
                ui.checkbox(&mut state.show_detailed_info, "show detailed info");
                if state.show_detailed_info { ui.label(env!("CARGO_PKG_VERSION"));}
                ui.hyperlink("https://sync.ollielynas.com");
                });
            });


        egui::TopBottomPanel::top("top")

            .show(egui_ctx, |ui|{
                let mut add_task_id = 0;

                    if state.show_detailed_info {
                        ui.horizontal(|ui| {
                egui::global_theme_preference_switch(ui);
                ui.menu_button("▼ Tasks", |ui| {
                    for task in &mut state.user_available_tasks {
                        if ui.button(task.get_name()).on_hover_text("task description").clicked() {
                            add_task_id = task.get_id();
                            ui.close_menu();
                        }
                    }
                });
                });}
                ui.horizontal(|ui| {
                if ui.button("Clear Data").clicked() {
                    let _ = state.add_task_without_duplicate(build_reset_data_task());
                }
                if
                    state.new_files.to_string() == "0 files".to_string() &&
                    state.unsorted_files.to_string() == "0 files".to_string() &&
                    state.timeline.to_string() != "0 timeline entries".to_string()

                    {
                        ui.label(RichText::new("<- step 0: clear data from previous project").background_color(Color32::LIGHT_YELLOW));
                }
                });

                ui.horizontal(|ui| {
                if ui.button("Import Media").clicked() {
                    let _ = state.add_task_without_duplicate(build_import_media_task_folders());
                }
                    if
                        !(state.new_files.to_string() == "0 files".to_string() &&
                        state.unsorted_files.to_string() == "0 files".to_string() &&
                        state.timeline.to_string() != "0 timeline entries".to_string())

                        {
                            ui.label(RichText::new("<- step 1: import new files from sd cards").background_color(Color32::LIGHT_YELLOW));
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button(format!("Process {} from input", state.new_files.to_string())).clicked() {
                        let _ = state.add_task(build_process_new_files_task());
                    }
                    if
                        !(state.unsorted_files.to_string() == "0 files".to_string() &&
                        state.timeline.to_string() != "0 timeline entries".to_string()) &&
                        state.new_files.to_string() != "0 files".to_string()


                        {
                            ui.label(RichText::new("<- step 2: start processing imported files").background_color(Color32::LIGHT_YELLOW));
                    }
                });
                ui.horizontal(|ui| {
                match &mut state.save_location_options {
                    crate::val_hold::ValueHolder::Value(slo) => {
                        if ui.button(format!("Sort {} processed files", state.unsorted_files.to_string())).clicked() {
                            slo.popup_open = true;
                        }
                        if slo.render(egui_ctx, state.output_folder.to_string()) {
                            slo.popup_open = false;
                            let _ = state.add_task(build_process_unsorted_files_task());
                        }
                    },
                    crate::val_hold::ValueHolder::BackupValue(_, _) => {
                        if ui.button(format!("Sort {} processed files", state.unsorted_files.to_string())).clicked() {
                                let _ = state.add_task(build_process_unsorted_files_task());
                        }
                    },
                };
                if
                    state.new_files.to_string() == "0 files".to_string() &&
                    state.unsorted_files.to_string() != "0 files".to_string() &&
                    state.timeline.to_string() != "0 timeline entries".to_string()

                    {
                        ui.label(RichText::new("<- step 3: after all project files have been imported and processed, sort them").background_color(Color32::LIGHT_YELLOW));
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
                   if state.show_detailed_info { ui.strong("Task ID");}
                    ui.end_row();
                if state.show_detailed_info {
                ui.label("input folder");
                    if ui.link(truncate_front(state.input_folder.to_string(), 30)).clicked() {
                        let _ = open::that(state.input_folder.to_string());
                    }


                ui.label(state.input_folder.used_by_string());
                ui.end_row();
                }

                if state.show_detailed_info {
                    ui.label("processing folder");
                    if ui.link(truncate_front(state.unsorted_folder.to_string(), 30)).clicked() {
                        let _ = open::that(state.unsorted_folder.to_string());
                    }
                    ui.label(state.unsorted_folder.used_by_string());
                    ui.end_row();
                }

                ui.label("output folder");
                if ui.link(truncate_front(state.output_folder.to_string(),30)).clicked() {
                    let _ = open::that(state.output_folder.to_string());
                };
                ui.label(state.output_folder.used_by_string());

                ui.end_row();

                ui.label("no. unprocessed files:");
                ui.label(state.new_files.to_string());
                if state.show_detailed_info {ui.label(state.new_files.used_by_string());}
                ui.end_row();

                ui.label("no. processed, ready to be sorted files:");
                ui.label(state.unsorted_files.to_string());
                if state.show_detailed_info {ui.label(state.unsorted_files.used_by_string());}
                ui.end_row();

                ui.label("timeline:");
                ui.label(state.timeline.to_string());
                if state.show_detailed_info {ui.label(state.timeline.used_by_string());}
                ui.end_row();



                });
                if state.show_detailed_info {
                    ui.separator();
                    let _ = state.command_pool_ui.render(ui);
                }
//

            });
        egui::CentralPanel::default()
            .show(egui_ctx, |ui| {
                egui::Grid::new("tasks").show(ui, |ui| {
                    ui.strong("Task");
                    ui.strong("Progress");
                    ui.strong("Elapsed");
                    if state.show_detailed_info {ui.label("State");}
                    if state.show_detailed_info {ui.label("ID");}
                    ui.end_row();
                    for t in &mut state.task_list {
                        if t.silent() && !state.show_detailed_info {continue;}
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

                            // todo: for now you cannot kill a running task because of safety, so a co-opretave way of doing it will have to be created
                            // if ui.small_button("cancel").clicked() {
                            //     kill_tasks.push(t.get_id());
                            // }

                        }else {
                            ui.label(format!("Waiting"));
                            ui.label(format!("--:--"));
                            ui.label(format!("{:.1}%", 0));
                            if state.show_detailed_info {
                            ui.label(t.get_progress().get_item().to_string());
                            }
                        }
                        if state.show_detailed_info {ui.label((t.get_id()%999).to_string());}
                        ui.end_row();
                    }
                });
            });
    });

    if kill_tasks.len() > 0 {
        let mut tasks_temp_holder = vec![];
        std::mem::swap(&mut tasks_temp_holder, &mut state.task_list);
        for id in kill_tasks {

            for t in &mut tasks_temp_holder {
                if id == t.get_id() {
                t.cancel_task(state);
                continue;
                }
            }
        }
        std::mem::swap(&mut tasks_temp_holder, &mut state.task_list);
    }

}
