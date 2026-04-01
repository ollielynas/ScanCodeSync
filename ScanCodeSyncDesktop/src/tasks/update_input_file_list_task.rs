use std::{path::PathBuf, thread};

use anyhow::bail;
use atomic_progress::Progress;

use crate::{task::Task, util::recurse_files};


pub struct UpdateNewFileListValues {
    pub input_folder: PathBuf,
    pub new_files: Vec<PathBuf>,
}

pub struct UpdateInputFileListTask {
    handle: Option<thread::JoinHandle<anyhow::Result<UpdateNewFileListValues>>>,
    progress: Progress,
    id: u64,
    finished: bool,
}

impl Default for UpdateInputFileListTask {
    fn default() -> Self {
        Self {
            handle: None, progress: Progress::new_spinner("Update Input File List"), id: fastrand::u64(0..u64::MAX), finished: false }
    }
}


impl Task for UpdateInputFileListTask {
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
        let values: UpdateNewFileListValues;
        if [
                state.input_folder.available(),
                state.new_files.available(),
                ].iter().all(|x| *x) {
                    let _ = state.new_files.depopulate(self.id)?;
                    values = UpdateNewFileListValues {
                        input_folder: state.input_folder.depopulate(self.id)?.to_path_buf(),
                        new_files: vec![],
                    };
            }else {
                anyhow::bail!("not all of the values are available");
            }

        self.finished = false;

        self.progress = Progress::new_pb("Check For New Inputs", 2_u64);
        let progress = self.progress.clone();
        progress.bump();
        progress.set_item("starting thread");
        self.handle = Some(thread::spawn(move || {
            let mut values = values;

            values.new_files = recurse_files(values.input_folder.clone(), progress.clone())?;
            progress.bump();


            return Ok(values);
        }));
        return Ok(());

    }

    fn get_name(&self) -> &str {
        "discover input files"
    }

    fn silent(&self) -> bool {
        false
    }

    /// I think this is unfinished so like; todo: finish
    fn cancel_task(&mut self, state: &mut crate::state::State) {
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

                state.input_folder.populate(Box::new(a.input_folder))?;
                state.new_files.populate(Box::new(a.new_files))?;

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
}
