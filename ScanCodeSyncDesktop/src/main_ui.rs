use crate::state::State;
use egui_macroquad::egui;
use macroquad::window::screen_width;
use open;



pub fn render_state(state: &mut State) {
    egui_macroquad::ui(|egui_ctx| {

        egui::TopBottomPanel::top("top")
            .show(egui_ctx, |ui|{
                ui.label("thingy");
            });

        egui::SidePanel::left("left panel (state)")

            .show(egui_ctx, |ui| {
                egui::Grid::new("state").show(ui, |ui| {
                ui.label("file input");
                if ui.link(state.input_folder.to_string()).clicked() {
                    let _ = open::that(state.input_folder.to_string());
                }
                ui.end_row();
                ui.label("output Path");
                if ui.link(state.output_folder.to_string()).clicked() {
                    let _ = open::that(state.output_folder.to_string());
                };

                ui.end_row();

                ui.label("no. unprocessed files:");
                ui.label(state.processed_unsorted_files.to_string());

                });
            });
        egui::CentralPanel::default()
            .show(egui_ctx, |ui| {
                egui::Grid::new("tasks").show(ui, |ui| {
                    ui.label("Task");
                    ui.label("Progress");
                    ui.label("Elapsed");
                    ui.label("State");
                    ui.end_row();
                    for t in &mut state.task_list {
                        if t.silent() {continue;}
                        ui.label(t.get_name());
                        if t.is_finished() {
                            ui.label("Finished");

                        }else if t.is_running() {
                            let time_text = t.get_progress().get_elapsed().map_or("--:--".to_string(), |x|
                                {format!("{:02}:{:02}", x.as_secs() / 60, x.as_secs() % 60)});
                            ui.label(format!("{}%", t.get_progress().get_percent()));
                            ui.label(time_text);
                            ui.label(t.get_progress().get_item().to_string());
                        }else {
                            ui.label(format!("Waiting"));
                            ui.label(format!("--:--"));
                            ui.label(t.get_progress().get_item().to_string());
                        }
                        ui.end_row();
                    }
                });
            });
    });

}
