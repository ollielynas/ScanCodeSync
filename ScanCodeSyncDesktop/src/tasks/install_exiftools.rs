use std::thread;

use anyhow::bail;
use atomic_progress::Progress;
use rfd::MessageDialogResult;

use crate::{ task::Task, tasks::task_builders::{build_restart_task}};



pub struct InstallExifToolsTask {
    handle: Option<thread::JoinHandle<anyhow::Result<()>>>,
    progress: Progress,
    id: u64,
    finished: bool,
}

impl Default for InstallExifToolsTask {
    fn default() -> Self {
        Self {
            handle: None, progress: Progress::new_spinner("Install Exiftools"), id: fastrand::u64(0..u64::MAX), finished: false }
    }
}


impl Task for InstallExifToolsTask {
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
        self.progress = Progress::new_spinner("Installing ExifTools");
        let progress = self.progress.clone();
        self.finished = false;
        self.handle = Some(thread::spawn(move || {
            #[cfg(target_os = "macos")]
            {if matches!(dispatch::Queue::main().exec_sync(||rfd::MessageDialog::new().set_title("Install ExifTools")
                .set_description("This software relies on ExifTools, would you like to install it?")
                .set_buttons(rfd::MessageButtons::YesNo)
                .show()), MessageDialogResult::Yes) {
                    match install_exiftool(&progress) {
                        Ok(_) => {
                        },
                        Err(e) => {
                            dispatch::Queue::main().exec_sync(||rfd::MessageDialog::new()
                                .set_title("Failed to Install ExifTools")
                                .set_description(format!("{e:?}\nPlease attempt to install yourself from:\nhttps://exiftool.org"))
                                .set_level(rfd::MessageLevel::Error).show());
                        },
                    }
                }else {
                    bail!("user chose not to install");
                }}

            #[cfg(not(target_os = "macos"))]
            {if matches!(rfd::MessageDialog::new().set_title("Install ExifTools")
                .set_description("This software relies on ExifTools, would you like to install it?")
                .set_buttons(rfd::MessageButtons::YesNo)
                .show(), MessageDialogResult::Yes) {
                    match install_exiftool(&progress) {
                        Ok(_) => {
                        },
                        Err(e) => {
                            rfd::MessageDialog::new()
                                .set_title("Failed to Install ExifTools")
                                .set_description(format!("{e:?}\nPlease attempt to install yourself from:\nhttps://exiftool.org"))
                                .set_level(rfd::MessageLevel::Error).show();
                        },
                    }
                }else {
                    bail!("user chose not to install");
                }}
            return Ok(());
        }));
        return Ok(());

    }

    fn get_name(&self) -> &str {
        "Install ExifTool"
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

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
const EXIFTOOL_BYTES: &[u8] = include_bytes!("../vendor/ExifTool_install_13.54_64.exe");
#[cfg(all(target_os = "windows", target_arch = "x86"))]
const EXIFTOOL_BYTES: &[u8] = include_bytes!("../vendor/ExifTool_install_13.54_32.exe");

#[cfg(target_os = "windows")]
pub fn install_exiftool(progress: &Progress) -> anyhow::Result<()> {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};

    use anyhow::Context;

    // Write the installer to a temp file
    let temp_dir = std::env::temp_dir();
    let installer_path = temp_dir.join("exiftool_installer.exe");

    progress.set_item("Extracting installer...");
    std::fs::write(&installer_path, EXIFTOOL_BYTES)
        .context("Failed to write ExifTool installer")?;

    // Run the installer silently
    progress.set_item("Running installer...");
    let mut child = Command::new(&installer_path)
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to execute ExifTool installer")?;

    let stdout = child.stdout.take().unwrap();
    for line in BufReader::new(stdout).lines() {
        progress.set_item(line.unwrap_or_default());
    }

    let status = child.wait().context("Failed to wait on installer")?;

    // Clean up temp installer
    let _ = std::fs::remove_file(&installer_path);

    if status.success() {
        progress.set_item("ExifTool installed successfully");
        Ok(())
    } else {
        bail!("ExifTool installer failed")
    }
}
#[cfg(target_os = "linux")]
pub fn install_exiftool(progress: &Progress) -> anyhow::Result<()> {
    use anyhow::Context;
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};

    let candidates = [
        ("apt-get", vec!["install", "-y", "libimage-exiftool-perl"]), // Debian/Ubuntu/Mint
        ("dnf",     vec!["install", "-y", "perl-Image-ExifTool"]),    // Fedora/RHEL
        ("pacman",  vec!["-S", "--noconfirm", "perl-image-exiftool"]), // Arch
        ("zypper",  vec!["install", "-y", "perl-Image-ExifTool"]),    // openSUSE
    ];

    for (pm, args) in &candidates {
        if Command::new("which").arg(pm).output().map(|o| o.status.success()).unwrap_or(false) {
            progress.set_item(format!("trying {pm}..."));

            let mut child = Command::new("sudo")
                .arg(pm)
                .args(args)
                .stdout(Stdio::piped())
                .spawn()
                .with_context(|| format!("Failed to execute {pm}"))?;

            let stdout = child.stdout.take().unwrap();
            for line in BufReader::new(stdout).lines() {
                progress.set_item(line.unwrap_or_default());
            }

            let status = child.wait().with_context(|| format!("Failed to wait on {pm}"))?;
            if status.success() {
                return Ok(());
            }

            return Err(anyhow::anyhow!("{pm} failed to install ExifTool"));
        }
    }

    Err(anyhow::anyhow!(
        "No supported package manager found. Please install ExifTool manually: https://exiftool.org/install.html"
    ))
}

#[cfg(target_os = "macos")]
pub fn install_exiftool(progress: &Progress) -> anyhow::Result<()> {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};
    use anyhow::Context;

    let mut child = Command::new("brew")
        .args(["install", "exiftool"])
        .spawn()
        .context("Failed to execute brew command")?;

    let status = child.wait().context("Failed to wait on brew")?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Homebrew failed to install ExifTool"))
    }
}
