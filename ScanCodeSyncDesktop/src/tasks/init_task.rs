use std::{path::{Path, PathBuf}, str::FromStr, thread};
use anyhow::{anyhow, bail};
use atomic_progress::{Progress, ProgressType};
use crate::{task::Task, util::{get_config, set_config}, val_hold::ValueHolder};
use directories::ProjectDirs;
pub struct StateInitValues {
    pub input_folder: PathBuf,
    pub output_folder: PathBuf,

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

    fn attempt_run(&mut self, state: &mut crate::state::State) -> anyhow::Result<()> {
        let values: StateInitValues;
        if [
                state.input_folder.available(),
                state.output_folder.available(),
                ].iter().all(|x| *x) {
                    values = StateInitValues {
                        input_folder: state.input_folder.depopulate(self.id)?.to_path_buf(),
                        output_folder: state.output_folder.depopulate(self.id)?.to_path_buf(),
                    };
            }else {
                anyhow::bail!("not all of the values are available");
            }

        self.finished = false;

        self.progress = Progress::new_pb("Init Task", 5_u64);
        let progress = self.progress.clone();

        progress.set_item("starting thread");
        self.handle = Some(thread::spawn(move || {
            progress.bump();
            let mut values = values;
            if let Some(proj_dirs) = ProjectDirs::from("com", "OllieLyans",  "Sync") {
                if let Some(path) = get_config("input_folder") {
                    progress.set_item("loading INPUT from config");
                    match PathBuf::from_str(&path) {
                        Ok(path_buf) => {
                            values.input_folder = path_buf;
                        }
                        _ => (),
                    }
                }
                progress.bump();
                if values.input_folder == PathBuf::new() {
                    progress.set_item("no input value found saved, loading default");
                    values.input_folder = proj_dirs.data_local_dir().to_path_buf().join("INPUT");
                    set_config("input_folder", values.input_folder.to_string_lossy().into_owned())?;
                }

                progress.bump();

                if let Some(path) = get_config("output_folder") {
                    match PathBuf::from_str(&path) {
                        Ok(path_buf) => {
                            values.output_folder = path_buf;
                        }
                        _ => (),
                    }
                }

                progress.bump();
                if values.output_folder == PathBuf::new() {
                    progress.set_item("no output folder found saved, loading default");
                    values.output_folder = proj_dirs.data_local_dir().to_path_buf().join("PROCESSED");
                    set_config("output_folder", values.output_folder.to_string_lossy().into_owned())?;
                }

                progress.bump();

            }

            return Ok(values);
        }));
        return Ok(());

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

                state.input_folder.populate(Box::new(a.input_folder))?;
                state.output_folder.populate(Box::new(a.output_folder))?;

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
