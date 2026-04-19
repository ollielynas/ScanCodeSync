use std::{path::{PathBuf}, thread};
use anyhow::{Context, bail};
use atomic_progress::Progress;
use rfd::MessageDialogResult;
use ffmpeg_sidecar::download::{download_ffmpeg_package, ffmpeg_download_url, unpack_ffmpeg};

use crate::{task::Task, tasks::task_builders::build_restart_task, util::get_project_dir};

pub struct InstallFfmpegTask {
    handle: Option<thread::JoinHandle<anyhow::Result<()>>>,
    progress: Progress,
    id: u64,
    finished: bool,
}

impl InstallFfmpegTask {
    pub fn new() -> Self {
        Self {
            handle: None,
            progress: Progress::new_spinner("Install FFmpeg"),
            id: fastrand::u64(0..u64::MAX),
            finished: false,
        }
    }
}

impl Task for InstallFfmpegTask {
    fn get_progress(&self) -> &Progress {
        &self.progress
    }

    fn is_running(&self) -> bool {
        match &self.handle {
            Some(_) => !self.finished,
            None => false,
        }
    }

    fn attempt_run(&mut self, _state: &mut crate::state::State) -> anyhow::Result<()> {
        self.progress = Progress::new_spinner("Installing FFmpeg");
        let progress = self.progress.clone();
        self.finished = false;

        self.handle = Some(thread::spawn(move || {
            #[cfg(target_os = "macos")]
            {
                let confirmed = dispatch::Queue::main().exec_sync(|| {
                    rfd::MessageDialog::new()
                        .set_title("Install FFmpeg")
                        .set_description("This software requires FFmpeg. Would you like to install it via Homebrew now?")
                        .set_buttons(rfd::MessageButtons::YesNo)
                        .show()
                });

                if !matches!(confirmed, MessageDialogResult::Yes) {
                    bail!("User chose not to install");
                }

                match install_ffmpeg_via_brew(&progress) {
                    Ok(_) => {}
                    Err(e) => {
                        dispatch::Queue::main().exec_sync(|| {
                            rfd::MessageDialog::new()
                                .set_title("Failed to Install FFmpeg")
                                .set_description(format!("{e:?}\nPlease install FFmpeg manually via `brew install ffmpeg`."))
                                .set_level(rfd::MessageLevel::Error)
                                .show()
                        });
                        bail!(e);
                    }
                }
            }

            #[cfg(not(target_os = "macos"))]
            {
                if matches!(
                    rfd::MessageDialog::new()
                        .set_title("Install FFmpeg")
                        .set_description("This software requires FFmpeg. Would you like to download it now?")
                        .set_buttons(rfd::MessageButtons::YesNo)
                        .show(),
                    MessageDialogResult::Yes
                ) {
                    let dir = get_project_dir()?;
                    let cdir = dir.cache_dir();
                    match download_and_unpack_ffmpeg(&progress, cdir.to_owned()) {
                        Ok(_) => {}
                        Err(e) => {
                            rfd::MessageDialog::new()
                                .set_title("Failed to Install FFmpeg")
                                .set_description(format!("{e:?}\nPlease install FFmpeg manually."))
                                .set_level(rfd::MessageLevel::Error)
                                .show();
                            bail!(e);
                        }
                    }
                } else {
                    bail!("User chose not to install");
                }
            }

            Ok(())
        }));
        Ok(())
    }

    fn get_name(&self) -> &str {
        "Install FFmpeg"
    }

    fn silent(&self) -> bool {
        false
    }

    fn cancel_task(&mut self, _state: &mut crate::state::State) {
        self.handle = None;
        self.finished = true;
    }

    fn attempt_collect(&mut self, _state: &mut crate::state::State) -> anyhow::Result<()> {
        match &mut self.handle {
            Some(h) if h.is_finished() => {
                self.finished = true;
            }
            _ => return Ok(()),
        }
        let handle = self.handle.take().ok_or(anyhow::anyhow!("Failed to take handle"))?;
        match handle.join() {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => bail!("{}", e),
            Err(e) => bail!("{:?}", e),
        }
    }

    fn get_id(&self) -> u64 {
        self.id
    }

    fn is_finished(&mut self) -> bool {
        self.finished
    }

    fn chain_tasks(&self) -> Vec<Box<dyn Task>> {
        vec![build_restart_task()]
    }
}

#[cfg(target_os = "macos")]
pub fn install_ffmpeg_via_brew(progress: &Progress) -> anyhow::Result<()> {
    progress.set_item("Installing FFmpeg via Homebrew...");

    let status = std::process::Command::new("brew")
        .args(["install", "ffmpeg"])
        .status()
        .context("Failed to run brew — is Homebrew installed?")?;

    if !status.success() {
        bail!("brew install ffmpeg failed with status: {status}");
    }

    progress.set_item("FFmpeg installed successfully");
    Ok(())
}

/// The core logic using ffmpeg-sidecar's manual download methods (non-macOS)
pub fn download_and_unpack_ffmpeg(progress: &Progress, dest_dir: PathBuf) -> anyhow::Result<()> {
    progress.set_item("Locating FFmpeg release...");
    let url = ffmpeg_download_url()?;

    if !dest_dir.exists() {
        std::fs::create_dir_all(&dest_dir).context("Failed to create install directory")?;
    }

    progress.set_item("Downloading FFmpeg archive...");
    let dir = get_project_dir()?;
    let cdir = dir.cache_dir();

    let archive_path = download_ffmpeg_package(url, &cdir)
        .map_err(|e| anyhow::anyhow!(e))?;

    progress.set_item("Unpacking binaries...");
    unpack_ffmpeg(&archive_path, &dest_dir)
        .map_err(|e| anyhow::anyhow!(e))?;

    let _ = std::fs::remove_file(archive_path);

    progress.set_item("FFmpeg installed successfully");
    Ok(())
}
