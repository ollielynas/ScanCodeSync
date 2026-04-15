use std::{ffi::OsStr, fs, path::{Path, PathBuf}, thread};

use anyhow::bail;
use atomic_progress::Progress;
use egui_macroquad::egui::TextBuffer;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{data::file_metadata::write_custom_metadata, populate_field, task::Task, tasks::task_builders::build_update_input_files_list_task, util::{format_filesize_human_readable, get_total_size_of_files, recurse_files}};



pub struct ImportMediaTask {
    handle: Option<thread::JoinHandle<anyhow::Result<PathBuf>>>,
    progress: Progress,
    id: u64,
    finished: bool,
    files: bool,
}

impl ImportMediaTask {
    pub fn new(files: bool)-> Self {
        ImportMediaTask { handle: None, progress: Progress::new_spinner("importing media"), id: fastrand::u64(0..u64::MAX), finished: false, files }
    }
}



impl Task for ImportMediaTask {
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
        let path: PathBuf;
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
        let is_files = self.files.clone();
        progress.bump();
        progress.set_item("starting thread");
        self.handle = Some(thread::spawn(move || {
            progress.set_item("opening file dialogue");

            let paths: Vec<PathBuf>;
            let mut root_folders: Vec<PathBuf> = vec![];

            if is_files {
                paths = rfd::FileDialog::new()
                    .set_title("import files")
                    .set_can_create_directories(false)
                    .pick_files()
                    .ok_or(anyhow::anyhow!("user did not pick any files"))?;
            } else {
                root_folders = rfd::FileDialog::new()
                    .set_title("import from folders")
                    .set_can_create_directories(false)
                    .pick_folders()
                    .ok_or(anyhow::anyhow!("user did not pick any folders"))?;

                paths = root_folders
                    .iter()
                    .flat_map(|x| recurse_files(x, progress.clone()).unwrap_or_default())
                    .collect();
            }

            let delete_og = match rfd::MessageDialog::new()
                .set_buttons(rfd::MessageButtons::YesNoCancel)
                .set_title("Import Files")
                .set_description(format!(
                    "Ready to import {} files ({})\nKeep originals after import?",
                    paths.len(),
                    format_filesize_human_readable(get_total_size_of_files(&paths))
                ))
                .show()
            {
                rfd::MessageDialogResult::Yes | rfd::MessageDialogResult::Ok => false,
                rfd::MessageDialogResult::No => true,
                _ => bail!("user cancelled import"),
            };

            progress.set_total(paths.len() as u64);
            progress.set_pos(0);
            let ex = exiftool::ExifTool::new()?;
            paths.par_iter().for_each(|file| {

                let _ = write_custom_metadata(&file, "OriginalFilePath", file.to_string_lossy().as_str(), &ex);

                let name = file.file_name().unwrap_or(OsStr::new("unknown"));
                progress.set_item(format!("copying {}", name.to_string_lossy()));

                // Find the most specific root folder this file belongs to,
                // then strip it to get the relative path to preserve.
                let relative = root_folders
                    .iter()
                    .filter_map(|root| file.strip_prefix(root.parent().unwrap_or(Path::new(""))).ok())
                    .min_by_key(|rel| rel.as_os_str().len())  // shortest relative = deepest root match
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from(name));   // fallback: just filename

                let new_location = path.join(&relative);

                if let Some(parent) = new_location.parent() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        progress.set_item(format!("failed to create dirs for {}: {e}", name.to_string_lossy()));
                        return;
                    }
                }

                match fs::copy(file, &new_location) {
                    Ok(_) => progress.set_item(format!("copied {}", name.to_string_lossy())),
                    Err(e) => progress.set_item(format!("failed to copy {}: {e}", name.to_string_lossy())),
                }
                progress.bump();
            });

            if delete_og {
                progress.set_total(paths.len() as u64);
                progress.set_pos(0);
                paths.par_iter().for_each(|file| {
                    progress.bump();
                    let name = file.file_name().unwrap_or(OsStr::new("unknown")).to_string_lossy();
                    progress.set_item(format!("deleting {name}"));
                    if let Err(e) = fs::remove_file(file) {
                        progress.set_item(format!("failed to delete {name}: {e}"));
                    }
                });
            }

            Ok(path)
        }));
        return Ok(());

    }

    fn get_name(&self) -> &str {
        if self.files {"import media/metdata files"} else {"import media/metadata folders"}
    }

    fn silent(&self) -> bool {
        false
    }

    /// I think this is unfinished so like; todo: finish
    fn cancel_task(&mut self, state: &mut crate::state::State) {
        let _ = state;
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
