use std::{ mem, path::PathBuf};
use egui_macroquad::egui::{self, Context, Ui, ahash::HashSet};

use crate::{command_pool::{SharedCommandPool, SharedCommandPoolUiState}, data::timeline::Timeline, load_field, task::Task, tasks::task_builders::user_accessible_tasks, val_hold::*};

pub struct State {
    pub input_folder: ValueHolder<PathBuf>,
    pub unsorted_folder: ValueHolder<PathBuf>,
    pub output_folder: ValueHolder<PathBuf>,

    pub new_files: ValueHolder<Vec<PathBuf>>,
    pub unsorted_files: ValueHolder<Vec<PathBuf>>,


    pub timeline: ValueHolder<Timeline>,

    pub save_location_options: ValueHolder<SaveLocationOptions>,

    pub task_list: Vec<Box<dyn Task>>,
    pub user_available_tasks: Vec<Box<dyn Task>>,

    pub command_pool: SharedCommandPool,
    pub command_pool_ui: SharedCommandPoolUiState,

}


impl Default for State {
    fn default() -> Self {

        let pool = SharedCommandPool::new();

        let mut new = State {
            input_folder: ValueHolder::Value(Box::new(PathBuf::new())),
            unsorted_folder: ValueHolder::Value(Box::new(PathBuf::new())),
            output_folder: ValueHolder::Value(Box::new(PathBuf::new())),
            new_files: ValueHolder::Value(Box::new(vec![])),
            unsorted_files: ValueHolder::Value(Box::new(vec![])),
            timeline: ValueHolder::Value(Box::new(Timeline::new())),
            save_location_options: ValueHolder::Value(Box::new(SaveLocationOptions::new())),


            user_available_tasks: user_accessible_tasks(),
            command_pool: pool.clone(),
            command_pool_ui: SharedCommandPoolUiState::new(pool.clone()),

            task_list: vec![],
        };

        // load values from json
        crate::dbp!("loaded {:?}", load_field!(new, input_folder));
        crate::dbp!("loaded {:?}", load_field!(new, output_folder));
        crate::dbp!("loaded {:?}", load_field!(new, unsorted_files));
        crate::dbp!("loaded {:?}", load_field!(new, new_files));
        crate::dbp!("loaded {:?}", load_field!(new, timeline));

        return new;
    }
}


impl State {
    pub fn add_task(&mut self, task: Box<dyn Task>) {
        self.task_list.push(task);
    }
    pub fn add_task_without_duplicate(&mut self, task: Box<dyn Task>) -> anyhow::Result<()> {
        if self.task_list.iter().find(|x| x.get_name() == task.get_name()).is_some() {
            anyhow::bail!("task already exists")
        }
        self.task_list.push(task);
        Ok(())
    }

    pub fn all_values(&mut self) ->  Vec<&mut dyn ValueHolderExt> {
        return vec![
            &mut self.timeline,

            &mut self.input_folder,
            &mut self.output_folder,
            &mut self.unsorted_folder,

            &mut self.unsorted_files,
            &mut self.new_files,
        ];
    }


    /// If the task that took ownership of the value fails in any way the values will be restored to their defaults
    pub fn restore_dropped_variables(&mut self) {
        let task_ids: HashSet<u64> = self.task_list.iter().map(|x| x.get_id()).collect();
        self.command_pool.prune_commands(&task_ids);
        for v in self.all_values() {
            v.restore_if_dropped(&task_ids);
        }
    }


    /// Goes through the task list and attempts to start any function that has not already been started
    pub fn update_tasks(&mut self) {

        // removes tasks that have finished and adds any taskes included in the chain tasks list
        // potential todo: prevent task looping
        let mut chained_tasks = vec![];
        self.task_list.retain_mut(|x| if x.is_finished() {
            if x.get_progress().get_error().is_none() {
                chained_tasks.append(x.chain_tasks().as_mut());
            }
            false
        } else {true});

        self.task_list.append(&mut chained_tasks);

        let mut tasks:Vec<Box<dyn Task>> = vec![];
        // remove tasks from state
        mem::swap(&mut tasks, &mut self.task_list);
        for task in &mut tasks {
            if task.is_finished() {
            } else if task.is_running() {
                // todo, handle error popup
                let collect = task.attempt_collect(self);
                match collect {
                    Ok(_)  => {
                        if task.is_finished() {
                            task.get_progress().set_error(None::<String>);
                            task.get_progress().finish_with_item("finished");
                        }
                    },

                    Err(e) => {
                        crate::dbp!("failed to collect: {e}");
                        task.get_progress().finish_with_error(format!("{e:?}"));
                    },
                }

            }else {
                // todo, handle message
                let attempt = task.attempt_run(self);

                match attempt {
                    Ok(_) => {

                    },
                    Err(e) => {
                        task.get_progress().set_item(format!("{e}"));
                        // wait this is not corrrect, but i feel like I added it to fix somesort of bug
                        task.cancel_task(self);
                    },
                }
            }
        }
        // put tasks back into state
        mem::swap(&mut tasks, &mut self.task_list);
    }
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct SaveLocationOptions {
    pub popup_open: bool,
    pub file_type: bool,
    pub date: bool,
    pub scene: bool,
    pub take: bool,
    pub device: bool,
}

impl SaveLocationOptions {
    fn new() -> Self {
        Self { popup_open: false,
            file_type: true,
            date: true,
            scene: true,
            take: true,
            device: true
        }
    }

    pub fn render(&mut self, ctx: &Context, mut save_file_path: String) -> bool {
        if !self.popup_open {return false}
        let mut process_files = false;
        let mut close_modal = false;
        let modal = egui_macroquad::egui::Modal::new(egui::Id::new("my_modal")).show(ctx, |ui| {
            save_file_path = save_file_path.replace("\\", "/");
                    ui.heading("Sorting Options");
                    ui.checkbox(&mut self.file_type, "Media Type");
                    if self.file_type {save_file_path += "/MEDIA_TYPE"}
                    ui.checkbox(&mut self.date, "Date");
                    if self.date {save_file_path += "/DD-MM-YYYY"}
                    ui.checkbox(&mut self.scene, "Scene");
                    if self.date {save_file_path += "/SCENE_NAME"}
                    ui.checkbox(&mut self.take, "Take No.");
                    if self.take {save_file_path += "/TAKE_NUMBER"}
                    ui.checkbox(&mut self.device, "Recording Device Name");
                    if self.device {save_file_path += "/TAKE_NUMBER"}
                    save_file_path += "/OG_FILE_NAME.file";
                    ui.label("Files will be saved to:");
                    ui.label(format!("{save_file_path}"));
                    if ui.button("Start Sorting Processed Files").clicked() {
                        process_files = true;
                    }
                    if ui.button("Cancel").clicked() {
                        close_modal = true;
                    }
                });
        if modal.should_close() || close_modal {
            self.popup_open = false;
            return false;
        }else {
            return process_files;
        }
    }

}
