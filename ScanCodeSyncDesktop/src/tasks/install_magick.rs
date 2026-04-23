use std::thread;

use anyhow::{Context, bail};
use atomic_progress::Progress;
use rfd::MessageDialogResult;

use crate::{ dbp, task::Task, tasks::task_builders::build_restart_task};



pub struct InstallMagickTask {
    handle: Option<thread::JoinHandle<anyhow::Result<()>>>,
    progress: Progress,
    id: u64,
    finished: bool,
}

impl Default for InstallMagickTask {
    fn default() -> Self {
        Self {
            handle: None, progress: Progress::new_spinner("Install ImageMagick"), id: fastrand::u64(0..u64::MAX), finished: false }
    }
}


impl Task for InstallMagickTask {
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
        self.progress = Progress::new_spinner("Installing ImageMagick");
        let progress = self.progress.clone();
        self.finished = false;
        self.handle = Some(thread::spawn(move || {
            #[cfg(target_os = "macos")]
            {if matches!(
                dispatch::Queue::main().exec_sync(||rfd::MessageDialog::new().set_title("Install ImageMagick")
                .set_description("This software relies on ImageMagick, would you like to install it?")
                .set_buttons(rfd::MessageButtons::YesNo)
                .show()), MessageDialogResult::Yes)
            {
                    match install_magick(&progress) {
                        Ok(_) => {
                        },
                        Err(e) => {
                            dispatch::Queue::main().exec_sync(|| rfd::MessageDialog::new()
                                .set_title("Failed to Install ImageMagick")
                                .set_description(format!("{e:?}\nPlease attempt to install yourself from:\n https://imagemagick.org"))
                                .set_level(rfd::MessageLevel::Error).show());
                        },
                    }
                }else {
                    bail!("user chose to not install");
                }}
            #[cfg(not(target_os = "macos"))]
            {if matches!(
                rfd::MessageDialog::new().set_title("Install ImageMagick")
                .set_description("This software relies on ImageMagick, would you like to install it?")
                .set_buttons(rfd::MessageButtons::YesNo)
                .show(), MessageDialogResult::Yes)
            {
                    match install_magick(&progress) {
                        Ok(_) => {
                        },
                        Err(e) => {
                            rfd::MessageDialog::new()
                                .set_title("Failed to Install ImageMagick")
                                .set_description(format!("{e:?}\nPlease attempt to install yourself from:\n https://imagemagick.org"))
                                .set_level(rfd::MessageLevel::Error).show();
                        },
                    }
                }else {
                    bail!("user chose to not install");
                }}
            return Ok(());
        }));
        return Ok(());
    }

    fn get_name(&self) -> &str {
        "Install ImageMagick"
    }

    fn silent(&self) -> bool {
        false
    }

    /// I think this is unfinished so like; todo: finish
    fn cancel_task(&mut self, _state: &mut crate::state::State) {
        self.handle = None;
        self.finished = true;
    }

    fn attempt_collect(&mut self, _state: &mut crate::state::State) -> anyhow::Result<()> {

        match &mut self.handle {
            Some(h) if h.is_finished() => {
                self.finished = true;
            },
            _ => {return Ok(())},
        }
        let handle = self.handle.take().ok_or(anyhow::anyhow!("should be unreachable, failed to take handle"))?;

        let resault = handle.join();
        match resault {
            Ok(Ok(_a)) => {
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
        vec![build_restart_task()]
    }
}

/// Returns (program, args) pairs to try in order.
/// On Windows a single winget command is returned.
/// On Linux multiple package managers are tried until one succeeds.
/// On macOS brew is used.
#[cfg(target_os = "windows")]
fn install_candidates() -> Vec<(String, Vec<String>)> {
    vec![
        (
        "cmd".into(),
        vec!["/C".into(), "winget".into(), "install".into(), "--id".into(), "ImageMagick.ImageMagick".into(), "--silent".into()],
        )
    ]
}

#[cfg(target_os = "linux")]
fn install_candidates() -> Vec<(String, Vec<String>)> {
    vec![
        ("sudo".into(), vec!["apt-get".into(), "install".into(), "-y".into(), "imagemagick".into()]), // Debian/Ubuntu/Mint
        ("sudo".into(), vec!["dnf".into(),     "install".into(), "-y".into(), "imagemagick".into()]), // Fedora/RHEL
        ("sudo".into(), vec!["pacman".into(),  "-S".into(), "--noconfirm".into(), "imagemagick".into()]), // Arch
        ("sudo".into(), vec!["zypper".into(),  "install".into(), "-y".into(), "imagemagick".into()]), // openSUSE
    ]
}

#[cfg(target_os = "macos")]
fn install_candidates() -> Vec<(String, Vec<String>)> {
    vec![(
        "brew".into(),
        vec!["install".into(), "imagemagick".into()],
    )]
}


pub fn install_magick(progress: &Progress) -> anyhow::Result<()> {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};

    let candidates = install_candidates();

    for (program, args) in &candidates {
        progress.set_item(format!("Running: {program} {}...", args.join(" ")));
        dbp!("{} {:?}", program, args);
        let mut child = match Command::new(program)
            .args(args)
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(a) => {
                dbg!("{}:?", a);
                continue
            }, // program not found, try next
        };

        let stdout = child.stdout.take().unwrap();
        let mut already_installed = false;
        for line in BufReader::new(stdout).lines() {
            dbp!("{:?}",&line);
            if let Ok(line) = line {
                if line == "No available upgrade found.".to_string() {
                    already_installed = true;
                }
                progress.set_item(line);
            }
        }

        let status = child.wait()
            .with_context(|| format!("Failed to wait on {program}"))?;

        if status.success() || already_installed {
            progress.set_item("ImageMagick installed successfully");
            #[cfg(target_os = "windows")]
            add_magick_to_path(progress);
            return Ok(());
        }else {

        }
    }

    bail!("No valid install command found")
}

/// After a silent winget install, ImageMagick may not be on PATH.
/// Uses `winget show` to find the install location and appends it
/// to the user PATH via setx.
#[cfg(target_os = "windows")]
fn add_magick_to_path(progress: &Progress) {
    use std::process::Command;

    progress.set_item("Locating ImageMagick install directory...");

    let output = match Command::new("winget")
        .args(["show", "--id", "ImageMagick.ImageMagick"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return,
    };

    let text = String::from_utf8_lossy(&output.stdout);

    // `winget show` output contains a line like:
    //   Install Location: C:\Program Files\ImageMagick-7.1.1-Q16-HDRI
    let install_dir = text
        .lines()
        .find(|l| l.trim_start().starts_with("Install Location:"))
        .and_then(|l| l.splitn(2, ':').nth(1))
        .map(|s| s.trim().to_string());

    let Some(dir) = install_dir else {
        progress.set_item("Could not determine ImageMagick install location — PATH not updated");
        return;
    };

    progress.set_item(format!("Adding {dir} to PATH..."));

    let current_path = std::env::var("PATH").unwrap_or_default();
    if current_path.contains(&dir) {
        progress.set_item("ImageMagick already on PATH");
        return;
    }

    let new_path = format!("{};{}", current_path, dir);
    let status = Command::new("setx")
        .args(["PATH", &new_path])
        .status();

    match status {
        Ok(s) if s.success() => progress.set_item("PATH updated successfully"),
        _ => progress.set_item("Failed to update PATH — you may need to add ImageMagick to PATH manually"),
    }
}
