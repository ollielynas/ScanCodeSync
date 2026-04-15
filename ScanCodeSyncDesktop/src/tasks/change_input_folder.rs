use std::{path::PathBuf, thread};

use anyhow::bail;
use atomic_progress::Progress;

use crate::{populate_field, task::Task, tasks::task_builders::build_update_input_files_list_task};



pub struct ChangeInputFolderTask {
    handle: Option<thread::JoinHandle<anyhow::Result<PathBuf>>>,
    progress: Progress,
    id: u64,
    finished: bool,
}

impl Default for ChangeInputFolderTask {
    fn default() -> Self {
        Self {
            handle: None, progress: Progress::new_spinner("Change Input Folder"), id: fastrand::u64(0..u64::MAX), finished: false }
    }
}


impl Task for ChangeInputFolderTask {
    fn get_progress(&self) -> &Progress {
        &self.progress
    }

    fn is_running(&self) -> bool {
        match &self.handle {
            Some(_) => !self.finished,
            None => false,
        }
    }

    fn attempt_run(&mut self, state: &mut crate::state::State) -> anyhow::Result<()> {
        let mut path: PathBuf;
        if [
                state.input_folder.available(),
                ].iter().all(|x| *x) {
                    path =  state.input_folder.depopulate(self.id)?.to_path_buf();
            }else {
                anyhow::bail!("not all of the values are available");
            }

        self.finished = false;
        self.progress.set_total(2);

        let progress = self.progress.clone();
        progress.bump();
        progress.set_item("starting thread");
        self.handle = Some(thread::spawn(move || {
            progress.set_item("opening file dialogue");

            path = rfd::FileDialog::new().set_can_create_directories(true)
                .set_directory(path)
                .set_title("new input folder")
                .pick_folder().ok_or(anyhow::anyhow!("no file was picked to be input"))?.to_path_buf();

            progress.bump();
            progress.set_item("setting value");


            return Ok(path);
        }));
        return Ok(());

    }

    fn get_name(&self) -> &str {
        "change input path"
    }

    fn silent(&self) -> bool {
        false
    }

    /// I think this is unfinished so like; todo: finish
    fn cancel_task(&mut self, _state: &mut crate::state::State) {
        self.handle = None;
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
                populate_field!(state, input_folder, a)?;
                // state.input_folder.populate(Box::new(a))?;
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
}
