use std::{path::PathBuf, thread};

use anyhow::bail;
use atomic_progress::Progress;

use crate::{data::timeline::Timeline, populate_field, task::Task, tasks::task_builders::build_update_input_files_list_task, util::recurse_files};



pub struct ResetDataTask {
    handle: Option<thread::JoinHandle<anyhow::Result<Timeline>>>,
    progress: Progress,
    id: u64,
    finished: bool,
}

impl Default for ResetDataTask {
    fn default() -> Self {
        Self {
            handle: None, progress: Progress::new_spinner("Reset Data"), id: fastrand::u64(0..u64::MAX), finished: false }
    }
}


impl Task for ResetDataTask {
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
        if [
                state.timeline.available(),
                ].iter().all(|x| *x) {
                    _ =  state.timeline.depopulate(self.id)?;
            }else {
                anyhow::bail!("not all of the values are available");
            }

        self.finished = false;
        self.progress.set_total(2);

        let progress = self.progress.clone();
        progress.bump();
        progress.set_item("starting thread");
        self.handle = Some(thread::spawn(move || {

            progress.set_item("setting value");

            progress.bump();

            return Ok(Timeline::new());
        }));
        return Ok(());

    }

    fn get_name(&self) -> &str {
        "reset data"
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
                populate_field!(state, timeline, a)?;
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
