use std::{fs, path::{Path, PathBuf}, str::FromStr, thread};
use anyhow::{anyhow, bail};
use atomic_progress::{Progress, ProgressType};
use crate::{depopulate_all_into_state_init_values, load_field, populate_field, task::Task, tasks::task_builders::build_update_input_files_list_task, util::{get_config, get_project_dir, set_config}, val_hold::ValueHolder};
use directories::ProjectDirs;
pub struct StateInitValues {
    pub input_folder: Box<PathBuf>,
    pub output_folder: Box<PathBuf>,
    pub unsorted_folder: Box<PathBuf>,
}

pub struct InitTask {
    handle: Option<thread::JoinHandle<anyhow::Result<StateInitValues>>>,
    progress: Progress,
    id: u64,
    finished: bool,
}

impl Default for InitTask {
    fn default() -> Self {
        InitTask {
            finished: false,
            handle: None, progress: Progress::new(ProgressType::Spinner, "Init Task", 0_u64), id: fastrand::u64(0..u64::MAX)}
    }
}



impl Task for InitTask {
    fn get_progress(&self) -> &Progress {
        &self.progress
    }



    fn is_running(&self) -> bool {
        match &self.handle {
            Some(_) => !self.finished,
            None => false,
        }
    }
    fn get_name(&self) -> &str {
        "setup"
    }

    fn silent(&self) -> bool {
        false
    }

    /// I think this is unfinished so like; todo: finish
    fn cancel_task(&mut self, state: &mut crate::state::State) {

        if let Some(handle) = self.handle.take() {

        }
        self.finished = true;
    }

    fn attempt_collect(&mut self, state: &mut crate::state::State) -> anyhow::Result<()> {

        match &mut self.handle {
            Some(h) if h.is_finished() => {
                self.finished = true;
            },
            _ => {return Ok(())},
        }
        let handle = self.handle.take().ok_or(anyhow::anyhow!("should be unreachable, failed to take handle"))?;

        let resault = handle.join();
        match resault {
            Ok(Ok(a)) => {

                populate_field!(state, input_folder, *a.input_folder)?;
                populate_field!(state, output_folder, *a.output_folder)?;
                populate_field!(state, unsorted_folder, *a.unsorted_folder)?;

                return Ok(());
            }
            Ok(Err(e)) => {bail!("{}", e)}
            Err(e) => {bail!("{:?}", e)}
        }
    }


    fn get_id(&self) -> u64 {
        self.id
    }

    fn is_finished(&mut self) -> bool {
    self.finished
    }

    fn chain_tasks(&self) -> Vec<Box<dyn Task>> {
        vec![build_update_input_files_list_task()]
    }

    fn attempt_run(&mut self, state: &mut crate::state::State) -> anyhow::Result<()> {
        // Macro to check availability and depopulate in one go


        // 1. Grab current values from state (which were loaded from JSON at startup)
        let mut values = depopulate_all_into_state_init_values!(state, self.id, [input_folder, output_folder, unsorted_folder]);

        self.finished = false;
        self.progress = Progress::new_pb("Init Task", 4_u64);
        let progress = self.progress.clone();

        self.handle = Some(thread::spawn(move || -> anyhow::Result<StateInitValues> {
            let proj_dirs = get_project_dir()?;

            // Helper closure to ensure a path exists or fallback to default
            let ensure_path = |path: &mut Box<PathBuf>, subfolder: &str| -> anyhow::Result<()> {
                if **path == PathBuf::new() {
                    progress.set_item(&format!("Setting default for {}", subfolder));
                    **path = proj_dirs.data_local_dir().join(subfolder);
                }
                fs::create_dir_all(&**path)?;
                progress.bump();
                Ok(())
            };

            // 2. Validate/Create folders. If load_field failed earlier, these become defaults.
            ensure_path(&mut values.input_folder, "INPUT")?;
            ensure_path(&mut values.unsorted_folder, "UNSORTED")?;
            ensure_path(&mut values.output_folder, "PROCESSED")?;




            Ok(values)
        }));
        Ok(())
    }


}
