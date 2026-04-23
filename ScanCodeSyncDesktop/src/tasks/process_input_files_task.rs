use std::{collections::HashSet, ffi::OsStr, fs, path::PathBuf, thread};

use anyhow::bail;
use atomic_progress::{Progress, ProgressBuilder};
use exiftool::ExifTool;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{data::{data_entry::{DataValue, DeviceTime, TimelineEntry}, file_metadata::{get_creation_time_ms, get_device_id}, process_files::attempt_process_file, timeline::{self, Timeline}}, populate_field, task::Task, tasks::task_builders::{build_update_input_files_list_task, build_update_unsorted_files_list_task}, util::recurse_files};

// thread_local! {
//     static EXIFTOOL: RefCell<Option<exiftool::ExifTool>> = RefCell::new(None);
// }


pub struct ProcessInputFilesTask {
    handle: Option<thread::JoinHandle<anyhow::Result<(Vec<PathBuf>, Timeline, PathBuf)>>>,
    progress: Progress,
    id: u64,
    finished: bool,
}

impl Default for ProcessInputFilesTask {
    fn default() -> Self {
        Self {
            handle: None, progress: Progress::new_spinner("Process Input New Files"), id: fastrand::u64(0..u64::MAX), finished: false }
    }
}


impl Task for ProcessInputFilesTask {
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
        let mut files: Vec<PathBuf>;
        let mut timeline: Timeline;
        let mut unsorted_folder: PathBuf;
        if [
                state.new_files.available(),
                state.timeline.available(),
                state.unsorted_folder.available(),
                ].iter().all(|x| *x) {
                    files =  state.new_files.depopulate(self.id)?.to_vec();
                    unsorted_folder =  *state.unsorted_folder.depopulate(self.id)?;
                    timeline =  *state.timeline.depopulate(self.id)?;
            }else {
                anyhow::bail!("list of new files is being used");
            }

        self.finished = false;

        self.progress = ProgressBuilder::new_spinner("processing new file").with_start_time_now().build();

        let progress = self.progress.clone();
        progress.bump();
        progress.set_item("starting thread");


        let _ex = match exiftool::ExifTool::new() {
            Ok(a) => a,
            Err(_) => {

                #[cfg(target_os = "macos")]
                {dispatch::Queue::main().exec_sync(||rfd::MessageDialog::new().set_buttons(rfd::MessageButtons::Ok)
                    .set_title("ExifTool Is Not Installed")
                    .set_description("This function relies on ExifTool.\nPlease Install it from https://exiftool.org/")
                    .show());}

                #[cfg(not(target_os = "macos"))]
                {rfd::MessageDialog::new().set_buttons(rfd::MessageButtons::Ok)
                    .set_title("ExifTool Is Not Installed")
                    .set_description("This function relies on ExifTool.\nPlease Install it from https://exiftool.org/")
                    .show();}

                crate::dbp!("showed message, should bail next");
                anyhow::bail!("exif tool is not installed");
            },
        };

        let pool = state.command_pool.with_id(self.id.clone()).clone();

        self.handle = Some(thread::spawn(move || {

            progress.set_total(files.len() as u64 + 1);
            let tl_entries: HashSet<TimelineEntry> = files.par_iter().map(|x| {
                attempt_process_file(x, progress.clone(), pool.clone())

            }).filter(|x| x.is_ok()).flat_map(|x| x.unwrap()).collect();


            timeline.entries.extend(tl_entries);

            // dont forget to remove this after you have finished debugging

            progress.set_item("finished processing files");
            progress.set_total(files.len() as u64);

            let ex = pool.get_exif_command()?;

            for file in files {
                let filename = file.file_stem().unwrap_or(OsStr::new("undefined")).to_string_lossy().to_string();
                let filetype = file.extension().unwrap_or(OsStr::new(".file")).to_string_lossy().to_string();

                let creation_time = match get_creation_time_ms(&file, &ex) {
                    Ok(a) => a,
                    Err(a) => {
                        crate::dbp!("creation time not found {}",a);
                        continue;},
                };
                let camera_id = match get_device_id(&file, &ex) {
                    Ok(a) =>   a,
                    Err(a) => {
                        crate::dbp!("cam id not found {}",a);
                        continue;},
                };

                let new_file_name = format!("DEVICE_ID{}TIMESTAMP{}-{}.{}", camera_id.to_string(), creation_time, filename, filetype);
                let new_path = unsorted_folder.join(&new_file_name);

                progress.set_item(format!("moving file {}", filename));
                if fs::copy(&file,&new_path).is_ok() {
                    if matches!(new_path.try_exists(), Ok(true)) {
                        let _ = fs::remove_file(&file);
                    }
                };
                timeline.entries.insert(
                    TimelineEntry { time: DeviceTime {
                        internal_clock: creation_time,
                        device_id: camera_id,
                    }
                        ,
                        val: DataValue::MediaCreated(new_path) }
                );
            }
            pool.return_exif_command(ex);
            return Ok((vec![], timeline, unsorted_folder))
        }));
        return Ok(());

    }

    fn get_name(&self) -> &str {
        "process new files"
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

                populate_field!(state, new_files, a.0)?;
                populate_field!(state, timeline, a.1)?;
                populate_field!(state, unsorted_folder, a.2)?;

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
        vec![build_update_input_files_list_task(), build_update_unsorted_files_list_task()]
    }
}
