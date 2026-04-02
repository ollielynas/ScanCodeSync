use std::{fs::{self, read_dir}, path::{Path, PathBuf}};
use macroquad::{miniquad, window::Conf};
use rayon::prelude::*;

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
        println!("wrote to file");

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


use image::GenericImageView;

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
