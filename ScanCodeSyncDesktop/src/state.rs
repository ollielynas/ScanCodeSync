use std::{ mem, path::PathBuf};
use egui_macroquad::egui::ahash::HashSet;

use crate::{data::timeline::Timeline, load_field, task::Task, tasks::task_builders::user_accessible_tasks, val_hold::*};

pub struct State {
    pub input_folder: ValueHolder<PathBuf>,
    pub unsorted_folder: ValueHolder<PathBuf>,
    pub output_folder: ValueHolder<PathBuf>,

    pub new_files: ValueHolder<Vec<PathBuf>>,
    pub unsorted_files: ValueHolder<Vec<PathBuf>>,


    pub timeline: ValueHolder<Timeline>,

    pub task_list: Vec<Box<dyn Task>>,
    pub user_available_tasks: Vec<Box<dyn Task>>,
}


impl Default for State {
    fn default() -> Self {
        let mut new = State {
            input_folder: ValueHolder::Value(Box::new(PathBuf::new())),
            unsorted_folder: ValueHolder::Value(Box::new(PathBuf::new())),
            output_folder: ValueHolder::Value(Box::new(PathBuf::new())),
            new_files: ValueHolder::Value(Box::new(vec![])),
            unsorted_files: ValueHolder::Value(Box::new(vec![])),
            timeline: ValueHolder::Value(Box::new(Timeline::new())),

            user_available_tasks: user_accessible_tasks(),
            task_list: vec![],
        };

        let _ = load_field!(new, input_folder);
        let _ = load_field!(new, output_folder);
        let tl = load_field!(new, timeline);

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

        for v in self.all_values() {
            v.restore_if_dropped(&task_ids);
        }
    }


    /// Goes through the task list and attempts to start any function that has not already been started
    pub fn update_tasks(&mut self) {

        // removes tasks that have finished and adds any taskes included in the chain tasks list
        // potential todo: prevent task looping
        let mut chained_tasks = vec![];
        self.task_list.retain_mut(|x| if x.is_finished() {chained_tasks.append(x.chain_tasks().as_mut());false} else {true});
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
                    Ok(_) => {

                    },
                    Err(e) => {
                        task.get_progress().set_item(format!("{e}"));
                        task.cancel_task(self);
                    },
                }
            }
        }
        // put tasks back into state
        mem::swap(&mut tasks, &mut self.task_list);
    }
}
