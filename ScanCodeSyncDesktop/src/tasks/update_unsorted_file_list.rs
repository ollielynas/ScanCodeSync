use std::{path::PathBuf, thread};

use anyhow::bail;
use atomic_progress::Progress;

use crate::{populate_field, task::Task, util::recurse_files};


pub struct UpdateUnsortedFileListValues {
    pub unsorted_folder: PathBuf,
    pub unsorted_files: Vec<PathBuf>,
}
/// It could be a good idea to keep track of the last change in a folder and only update if something has changed. I dont know how easy that would be
pub struct UpdateUnsortedFileListTask {
    handle: Option<thread::JoinHandle<anyhow::Result<UpdateUnsortedFileListValues>>>,
    progress: Progress,
    id: u64,
    finished: bool,
}

impl Default for UpdateUnsortedFileListTask {
    fn default() -> Self {
        Self {
            handle: None, progress: Progress::new_spinner("Update Unsorted File List"), id: fastrand::u64(0..u64::MAX), finished: false }
    }
}


impl Task for UpdateUnsortedFileListTask {
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
        let values: UpdateUnsortedFileListValues;
        if [
                state.unsorted_folder.available(),
                state.unsorted_files.available(),
                ].iter().all(|x| *x) {
                    let _ = state.unsorted_files.depopulate(self.id)?;
                    values = UpdateUnsortedFileListValues {
                        unsorted_folder: state.unsorted_folder.depopulate(self.id)?.to_path_buf(),
                        unsorted_files: vec![],
                    };
            }else {
                anyhow::bail!("not all of the values are available");
            }

        self.finished = false;

        self.progress = Progress::new_pb("Check For New Unsorted Files", 2_u64);
        let progress = self.progress.clone();
        progress.bump();
        progress.set_item("starting thread");
        self.handle = Some(thread::spawn(move || {
            let mut values = values;

            values.unsorted_files = recurse_files(values.unsorted_folder.clone(), progress.clone())?;
            progress.bump();


            return Ok(values);
        }));
        return Ok(());

    }

    fn get_name(&self) -> &str {
        "discover unsorted files"
    }

    fn silent(&self) -> bool {
        true
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

                populate_field!(state, unsorted_folder, a.unsorted_folder)?;
                populate_field!(state, unsorted_files, a.unsorted_files)?;

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
