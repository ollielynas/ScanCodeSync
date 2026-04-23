use std::{path::{PathBuf, Prefix}, process::{Command, exit}, thread};

use anyhow::{self, Context, bail};
use atomic_progress::Progress;
use rfd::MessageDialogResult;

use crate::{ task::Task};



pub struct RestartTask {
    handle: Option<thread::JoinHandle<anyhow::Result<()>>>,
    progress: Progress,
    id: u64,
    finished: bool,
}

impl Default for RestartTask {
    fn default() -> Self {
        Self {
            handle: None, progress: Progress::new_spinner("Restart"), id: fastrand::u64(0..u64::MAX), finished: false }
    }
}


impl Task for RestartTask {
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
        self.progress = Progress::new_spinner("Restarting");
        if !state.all_values().iter().all(|x| x.available()) {
            bail!("waiting for all processes to be finished");
        }
        let progress = self.progress.clone();
        self.finished = false;
        self.handle = Some(thread::spawn(move || {
            progress.set_item("shutting down...");
            let current_exe = std::env::current_exe().expect("Failed to get current exe path");


            // macos
            #[cfg(target_os = "macos")]
            Command::new(&current_exe)
                .args(std::env::args().skip(1))
                .spawn()
                .expect("Failed to spawn new process");

            // linux
            #[cfg(target_os = "linux")]
            Command::new("bash")
                .arg("-l") // "Login" flag: tells bash to reload the user's full profile/PATH
                .arg("-c")
                .arg(format!("{} {}", current_exe.display(), std::env::args().skip(1).collect::<Vec<_>>().join(" ")))
                .spawn()
                .expect("Failed to restart");

            // windows
            #[cfg(target_os = "windows")]
            let restart_cmd = format!("Start-Process '{}'", current_exe.display());

            #[cfg(target_os = "windows")]
            Command::new("powershell")
                .args(["-Command", &restart_cmd])
                .spawn()
                .expect("Failed to restart");

            exit(0);
        }));
        return Ok(());

    }

    fn get_name(&self) -> &str {
        "Restart"
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
        vec![]
    }
}
