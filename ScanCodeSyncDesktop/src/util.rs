use std::{fs::{self, read_dir}, path::{Path, PathBuf}};

use anyhow::{self, Context};
use atomic_progress::Progress;
use directories::ProjectDirs;

pub fn set_config<T: ToString, S: ToString>(key: T, value: S) -> anyhow::Result<()> {
    let value = value.to_string().replace("=", "");
    let key = key.to_string();
    if let Some(proj_dirs) = ProjectDirs::from("com", "Ollie Lyans",  "Sync") {
        let config = proj_dirs.config_dir().to_path_buf().join("config.txt");
        if !fs::metadata(&config).is_ok() {
            fs::create_dir_all(config.clone().parent().ok_or(anyhow::anyhow!("failed to get parent of root"))?).with_context(||format!("failed to create directory: {:?}", &config))?;
            fs::write(&config, "").with_context(||format!("failed to write file: {:?}", &config))?;
        }

        let text = fs::read_to_string(config).context("failed to read config file")?;
        let mut lines: Vec<String> = text.lines().map(|x| x.to_string()).collect();
        for line in &mut lines {
            let mut entry = line.split("=");
            if entry.next() == Some(&key) {
                *line = format!("{key}={value}");
            }
        }
    }
    return Ok(())
}
pub fn get_config<T: ToString>(key: T) -> Option<String> {
    let key = key.to_string();
    if let Some(proj_dirs) = ProjectDirs::from("com", "OllieLyans",  "Sync") {
        let config = proj_dirs.config_dir().to_path_buf().join("config.txt");
        let text = fs::read_to_string(config).ok()?;
        let mut lines: Vec<String> = text.lines().map(|x| x.to_string()).collect();
        for line in &mut lines {
            let mut entry = line.split("=");
            if entry.next() == Some(&key) {
                return Some(entry.last()?.to_string());
            }
        }
    }
    return None;
}


// Source - https://stackoverflow.com/a/76820878
// Posted by RAVN Mateus, modified by community. See post 'Timeline' for change history
// Retrieved 2026-04-01, License - CC BY-SA 4.0

pub fn recurse_files(path: impl AsRef<Path>, progress: Progress) -> std::io::Result<Vec<PathBuf>> {
    let mut buf = vec![];
    let entries = read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        let meta = entry.metadata()?;

        if meta.is_dir() {
            let mut subdir = recurse_files(entry.path(), progress.clone())?;
            buf.append(&mut subdir);
        }

        if meta.is_file() {
            buf.push(entry.path());
            progress.set_item(entry.file_name().into_string().unwrap_or(String::from("filename error")));
        }
    }

    Ok(buf)
}
