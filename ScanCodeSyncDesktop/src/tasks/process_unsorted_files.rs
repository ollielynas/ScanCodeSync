use std::{collections::HashSet, ffi::OsStr, fs, path::PathBuf, thread};

use anyhow::bail;
use atomic_progress::{Progress, ProgressBuilder};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{data::{data_entry::{DataValue, DeviceTime, TimelineEntry}, file_metadata::{get_creation_time_ms, get_device_id}, process_files::{self, attempt_process_file}, timeline::{self, Timeline}}, populate_field, task::Task, tasks::task_builders::{build_update_input_files_list_task, build_update_unsorted_files_list_task}, util::recurse_files};

// thread_local! {
//     static EXIFTOOL: RefCell<Option<exiftool::ExifTool>> = RefCell::new(None);
// }


pub struct ProcessUnsortedFilesTask {
    handle: Option<thread::JoinHandle<anyhow::Result<(Vec<PathBuf>, Timeline, PathBuf)>>>,
    progress: Progress,
    id: u64,
    finished: bool,
}

impl Default for ProcessUnsortedFilesTask {
    fn default() -> Self {
        Self {
            handle: None, progress: Progress::new_spinner("Sorting Files Files"), id: fastrand::u64(0..u64::MAX), finished: false }
    }
}


impl Task for ProcessUnsortedFilesTask {
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
        let files: Vec<PathBuf>;
        let mut timeline: Timeline;
        let output_folder: PathBuf;
        if [
                state.unsorted_files.available(),
                state.timeline.available(),
                state.output_folder.available(),
                ].iter().all(|x| *x) {
                    files =  state.unsorted_files.depopulate(self.id)?.to_vec();
                    output_folder =  *state.output_folder.depopulate(self.id)?;
                    timeline =  *state.timeline.depopulate(self.id)?;
            }else {
                anyhow::bail!("list of new files is being used");
            }

        self.finished = false;

        self.progress = ProgressBuilder::new_spinner("processing file").with_start_time_now().build();

        let progress = self.progress.clone();
        progress.bump();
        progress.set_item("starting thread");



        self.handle = Some(thread::spawn(move || {


            timeline.sort_files(&output_folder, progress)?;

            return Ok((vec![], timeline, output_folder))
        }));
        return Ok(());

    }

    fn get_name(&self) -> &str {
        "sort processed files files"
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

                populate_field!(state, unsorted_files, a.0)?;
                populate_field!(state, timeline, a.1)?;
                populate_field!(state, output_folder, a.2)?;

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
        vec![build_update_unsorted_files_list_task()]
    }
}
