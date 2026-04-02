use std::{hash::{BuildHasher, Hash, Hasher}, path::PathBuf};

use egui_macroquad::egui::ahash::RandomState;
use rexif::{parse_file, TagValue, ExifTag};
use chrono::{TimeZone, Utc};
use anyhow::{Context, Result, anyhow};

use crate::data::data_entry::DeviceId;



fn exif_datetime_to_ms(datetime_str: &str) -> Result<u64> {
    let parts: Vec<&str> = datetime_str.splitn(2, ' ').collect();
    anyhow::ensure!(parts.len() == 2, "invalid EXIF datetime format: {datetime_str}");

    let date_parts: Vec<i32> = parts[0].split(':')
        .map(|s| s.parse::<i32>().context("invalid date component"))
        .collect::<Result<_>>()?;

    let time_parts: Vec<u32> = parts[1].split(':')
        .map(|s| s.parse::<u32>().context("invalid time component"))
        .collect::<Result<_>>()?;

    anyhow::ensure!(date_parts.len() == 3, "expected 3 date components");
    anyhow::ensure!(time_parts.len() == 3, "expected 3 time components");

    let dt = Utc.with_ymd_and_hms(
        date_parts[0],
        date_parts[1] as u32,
        date_parts[2] as u32,
        time_parts[0],
        time_parts[1],
        time_parts[2],
    )
    .single()
    .ok_or_else(|| anyhow!("ambiguous or invalid datetime: {datetime_str}"))?;

    Ok(dt.timestamp_millis() as u64)
}

pub fn get_creation_time_ms(path: &PathBuf) -> Result<u64> {

    let exif = parse_file(&path)
        .with_context(|| format!("failed to parse EXIF data from {path:?}"))?;

    let priority_tags = [
        ExifTag::DateTimeOriginal,
        ExifTag::DateTimeDigitized,
        ExifTag::DateTime,
    ];

    for tag in &priority_tags {
        for entry in &exif.entries {
            if &entry.tag == tag {
                if let TagValue::Ascii(ref s) = entry.value {
                    return exif_datetime_to_ms(s)
                        .with_context(|| format!("failed to parse timestamp from tag {:?}", tag));
                }
            }
        }
    }
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        if let Some((_, rest)) = stem.split_once("TIMESTAMP:") {
            if let Some((ts, _)) = rest.split_once('-') {
                if let Ok(ms) = ts.parse::<u64>() {
                    return Ok(ms);
                }
            }
        }
    }

    Err(anyhow!("no creation time tag found in {path:?}"))
}





pub fn get_device_id(path: &PathBuf) -> Result<DeviceId> {
    // filename takes priority
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        if let Some((_, rest)) = stem.split_once("DEVICE_ID:") {
            if let Some((id, _)) = rest.split_once("TIMESTAMP:") {
                if let Ok(n) = id.trim().parse::<u16>() {
                    return Ok(DeviceId::ClientId(n));
                }
            }
        }
    }

    // fall back to EXIF camera fingerprint
    let exif = parse_file(path.to_str().context("path is not valid UTF-8")?)
        .with_context(|| format!("failed to parse EXIF data from {}", path.display()))?;

    let mut fields: Vec<(&str, ExifTag, Option<String>)> = vec![
        ("make",   ExifTag::Make,             None),
        ("model",  ExifTag::Model,            None),
        ("software", ExifTag::Software, None),

    ];

    for entry in &exif.entries {
        if let TagValue::Ascii(ref s) = entry.value {
            let s = s.trim().to_string();
            for (_, tag, value) in fields.iter_mut() {
                if entry.tag == *tag && value.is_none() {
                    *value = Some(s.clone());
                }
            }
        }
    }

    let raw = fields
        .iter()
        .filter_map(|(label, _, value)| value.as_deref().map(|v| format!("{label}={v}")))
        .collect::<Vec<_>>()
        .join("|");

    if raw.is_empty() {
        return Err(anyhow!("no device id found in filename or EXIF for {}", path.display()));
    }

    let mut hasher = RandomState::with_seeds(0, 0, 0, 0).build_hasher();
    raw.hash(&mut hasher);
    let hash = hasher.finish();

    let prefix = fields
        .iter()
        .filter_map(|(_, _, value)| value.as_deref().map(|v| v.replace(' ', "")))
        .collect::<Vec<_>>()
        .join("-");

    let prefix = if prefix.is_empty() { "unknown".to_string() } else { prefix };

    Ok(DeviceId::CameraId(format!("{prefix}-{hash:016x}")))
}
