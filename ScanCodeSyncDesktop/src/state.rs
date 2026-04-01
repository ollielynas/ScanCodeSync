use std::{ mem, path::PathBuf};
use egui_macroquad::egui::ahash::HashSet;

use crate::{task::Task, val_hold::*};

pub struct State {
    pub input_folder: ValueHolder<PathBuf>,
    pub output_folder: ValueHolder<PathBuf>,

    pub new_files: ValueHolder<Vec<PathBuf>>,
    pub processed_unsorted_files: ValueHolder<Vec<PathBuf>>,

    pub task_list: Vec<Box<dyn Task>>
}


impl Default for State {
    fn default() -> Self {
        State {
            input_folder: ValueHolder::Value(Box::new(PathBuf::new())),
            output_folder: ValueHolder::Value(Box::new(PathBuf::new())),
            new_files: ValueHolder::Value(Box::new(vec![])),
            processed_unsorted_files: ValueHolder::Value(Box::new(vec![])),

            task_list: vec![],
        }
    }
}


impl State {
    pub fn add_task(&mut self, task: Box<dyn Task>) {
        self.task_list.push(task);
    }

    /// If the task that took ownership of the value fails in any way the values will be restored to their defaults
    pub fn restore_dropped_variables(&mut self) {
        let task_ids: HashSet<u64> = self.task_list.iter().map(|x| x.get_id()).collect();

        self.input_folder.restore_if_dropped(&task_ids);
        self.output_folder.restore_if_dropped(&task_ids);
        self.processed_unsorted_files.restore_if_dropped(&task_ids);

    }


    /// Goes through the task list and attempst to start any function that has not already been started
    pub fn update_tasks(&mut self) {

        self.task_list.retain_mut(|x| !x.is_finished());

        let mut tasks:Vec<Box<dyn Task>> = vec![];
        // remove tasks from state
        mem::swap(&mut tasks, &mut self.task_list);
        for task in &mut tasks {
            if task.is_finished() {
            } else if task.is_running() {
                // todo, handle error popup
                let collect = task.attempt_collect(self);
                match collect {
                    Ok(_) => {
                        if task.is_finished() {
                            task.get_progress().finish_with_item("finished");
                        }
                    },
                    Err(e) => {
                        println!("failed to collect: {e}");
                        task.get_progress().finish_with_error(format!("{e:?}"));
                    },
                }

            }else {
                // todo, handle message
                let attempt = task.attempt_run(self);

                match attempt {
                    Ok(_) => {},
                    Err(e) => {
                        task.get_progress().set_item(format!("{e}"));
                    },
                }
            }
        }
        // put tasks back into state
        mem::swap(&mut tasks, &mut self.task_list);
    }
}
