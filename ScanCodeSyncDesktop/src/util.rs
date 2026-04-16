use std::{fs::{self, read_dir}, path::{Path, PathBuf}, sync::LazyLock};
use ffmpeg_sidecar::paths::ffmpeg_path;
use macroquad::{miniquad, window::Conf};
use rayon::prelude::*;
use std::process::Command;
use std::collections::BTreeSet;
use chrono::{DateTime, Utc, Local};


use anyhow::{self, Context};
use atomic_progress::Progress;
use directories::ProjectDirs;

pub fn get_project_dir() -> anyhow::Result<ProjectDirs> {
    return ProjectDirs::from("com", "Ollie Lyans",  "Sync").ok_or(anyhow::anyhow!("failed to get project dir"));
}

pub fn get_total_size_of_files(paths: &[PathBuf]) -> u64 {
    paths.par_iter()
        .filter_map(|path| {
            fs::metadata(path).ok().map(|m| m.len())
        })
        .sum()
}

pub fn format_filesize_human_readable(bytes: u64) -> String {
    let mut size = bytes as f64;
    let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB"];
    let mut unit_idx = 0;

    // Keep dividing by 1024 until the size is under 1024 or we run out of units
    while size >= 1024.0 && unit_idx < units.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.2} {}", size, units[unit_idx])
}



/// this system should be redone to be more async safe
pub fn set_config<T: ToString, S: ToString>(key: T, value: S) -> anyhow::Result<()> {
    let value = value.to_string().replace("=", "");
    let key = key.to_string();
    let proj_dirs = get_project_dir()?;
        let config = proj_dirs.config_dir().to_path_buf().join("config.txt");
        if !fs::metadata(&config).is_ok() {
            fs::create_dir_all(config.clone().parent().ok_or(anyhow::anyhow!("failed to get parent of root"))?).with_context(||format!("failed to create directory: {:?}", &config))?;
            fs::write(&config, "").with_context(||format!("failed to write file: {:?}", &config))?;
        }

        let text = fs::read_to_string(&config).context("failed to read config file (set config util)")?;
        let mut lines: Vec<String> = text.lines().map(|x| x.to_string()).collect();
        for line in &mut lines {
            let mut entry = line.split("=");
            if entry.next() == Some(&key) {
                *line = format!("{key}={value}");
            }
        }

        fs::write(&config, lines.join("\n"))?;
        crate::dbp!("wrote to file");

    return Ok(())
}

/// this system should be redone to be more async safe
pub fn get_config<T: ToString>(key: T) -> Option<String> {
    let key = key.to_string();
    if let Ok(proj_dirs) = get_project_dir() {
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
            progress.set_item("found".to_string() + &entry.file_name().into_string().unwrap_or(String::from("filename error")));
        }
    }

    Ok(buf)
}


pub fn truncate_front<T: ToString>(s: T, max_chars: usize) -> String {
    let char_count = s.to_string().chars().count();

    if char_count <= max_chars {
        return s.to_string();
    }

    // Keep the last (max_chars - 3) characters
    let keep_count = max_chars.saturating_sub(3);
    let truncated: String = s.to_string().chars().skip(char_count - keep_count).collect();

    format!("...{}", truncated)
}



fn resize_icon(bytes: &[u8], size: u32) -> Vec<u8> {
    let img = image::load_from_memory(bytes).expect("Failed to decode icon");
    let resized = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
    resized.to_rgba8().into_raw()
}

pub fn window_conf() -> Conf {
    let icon_bytes = include_bytes!("favicon.png");

    let small  = resize_icon(icon_bytes, 16);
    let medium = resize_icon(icon_bytes, 32);
    let big    = resize_icon(icon_bytes, 64);

    Conf {
        window_title: "Sync File Sorter".to_string(),
        sample_count: 4,
        window_width: 1000,
        high_dpi: true,
        icon: Some(miniquad::conf::Icon {
            small:  small.try_into().unwrap(),
            medium: medium.try_into().unwrap(),
            big:    big.try_into().unwrap(),
        }),
        ..Default::default()
    }
}


pub fn is_magick_installed() -> bool {

    let mut cmd = Command::new("magick");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    cmd.arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}


pub static FFMPEG_FRMATS: LazyLock<BTreeSet<String>> = LazyLock::new(|| {
    let l = get_supported_extensions_set();
    crate::dbp!("ffmpeg formats {}", l.len());
    return l;
});

fn get_supported_extensions_set() -> BTreeSet<String> {
    let mut extensions = BTreeSet::new();

    let output = Command::new(ffmpeg_path())
        .args(["-hide_banner", "-formats"])
        .output()
        .expect("Failed to run FFmpeg");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    for line in stdout.lines().chain(stderr.lines()) {
        let line = line.trim_start();
        let mut parts = line.split_whitespace();

        let flags = match parts.next() {
            Some(flags) => flags,
            None => continue,
        };

        let format_name = match parts.next() {
            Some(format_name) => format_name,
            None => continue,
        };

        if !(flags.contains('D') || flags.contains('E')) || format_name == "=" {
            continue;
        }

        for ext in format_name.split(',').filter(|ext| !ext.is_empty()) {
            extensions.insert(ext.to_string());
        }
    }

    extensions
}

pub fn get_base_media_type(path: &PathBuf) -> String {
    let mut cmd = Command::new("magick");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    let output = match cmd
        // This regex trims everything after the slash internally
        .args(["-S", "-s3", "-p", "${mimetype;s/\\/.*//}", "-fast"])
        .arg(path)
        .output() {
            Ok(a) => a,
            Err(_) => return "unidentified file type".to_string()
        };

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// I should switch to the batch implementation for better speed.
pub fn get_mime_type(path: &PathBuf) -> String {

    let mut cmd = Command::new("ex");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    let output = cmd
        .args(["-b", "-mimetype"])
        .arg(path)
        .output()
        .expect("Failed to execute exiftool");

    // Convert stdout bytes to a trimmed string
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub fn get_batch_mimetypes(paths: Vec<&PathBuf>) -> anyhow::Result<Vec<(String, String)>> {
    let mut cmd = Command::new("magick");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    let output = cmd
        .args(["-T", "-S", "-mimetype", "-filename"]) // -T for tab-delimited, -S for short values
        .args(&paths)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() == 2 {
                Some((parts[1].trim().to_string(), parts[0].trim().to_string()))
            } else {
                None
            }
        })
        .collect())
}



pub fn epoch_ms_to_date(epoch_ms: u128) -> String {
    let secs = (epoch_ms / 1000) as i128;
    let nanos = ((epoch_ms % 1000) * 1_000_000) as u32;

    let utc = DateTime::from_timestamp(secs as i64, nanos).expect("Invalid timestamp");
    let local = utc.with_timezone(&Local);

    local.format("%d-%m-%Y").to_string()
}
