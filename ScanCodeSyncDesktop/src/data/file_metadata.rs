use std::path::PathBuf;


use chrono::{TimeZone, Utc};
use anyhow::{Context, Result, anyhow};
use exiftool::ExifTool;
use filenamify;

use crate::{command_pool::SharedCommandPool, data::data_entry::DeviceId};

use serde_json;


fn exif_datetime_to_ms(datetime_str: &str) -> Result<u64> {
    // Strip subseconds and timezone, we only need up to seconds
    let clean = datetime_str
        .split_once('.')
        .map(|(s, _)| s)  // strip subseconds
        .unwrap_or(datetime_str)
        .trim();

    // Strip timezone offset (+05:30 or Z)
    let clean = if let Some(pos) = clean.rfind(['+', '-', 'Z']) {
        // make sure we're not stripping the date's dashes
        if pos > 10 { &clean[..pos] } else { clean }
    } else {
        clean
    }.trim();

    // Handle both "2023:01:15 10:30:00" and "2023-01-15T10:30:00"
    let normalized = clean.replace('T', " ").replace('-', ":");
    // now format is always "YYYY:MM:DD HH:MM:SS"

    let parts: Vec<&str> = normalized.splitn(2, ' ').collect();
    anyhow::ensure!(parts.len() == 2, "invalid EXIF datetime format: {datetime_str}");

    let date_parts: Vec<i32> = parts[0].split(':')
        .map(|s| s.parse::<i32>().context("invalid date component"))
        .collect::<Result<_>>()?;
    let time_parts: Vec<u32> = parts[1].split(':')
        .map(|s| s.parse::<u32>().context("invalid time component"))
        .collect::<Result<_>>()?;

    anyhow::ensure!(date_parts.len() == 3, "expected 3 date components");
    anyhow::ensure!(time_parts.len() == 3, "expected 3 time components");

    // Guard against placeholder zero dates
    anyhow::ensure!(
        date_parts[0] > 0 && date_parts[1] > 0 && date_parts[2] > 0,
        "placeholder zero date: {datetime_str}"
    );

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

pub fn get_creation_time_ms(path: &PathBuf, ex: &ExifTool) -> Result<u64> {
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        if let Some((_, rest)) = stem.split_once("TIMESTAMP") {
            if let Some((ts, _)) = rest.split_once('-') {
                if let Ok(ms) = ts.parse::<u64>() {
                    return Ok(ms);
                }
            }
        }
    }
    for tag in ["SubSecDateTimeOriginal", "SubSecCreateDate", "SubSecModifyDate"] {
            if let Ok(val) = ex.read_tag::<serde_json::Value>(path, tag, &[]) {
                if let Some(s) = val.as_str() {
                    if let Ok(ms) = exif_datetime_to_ms(s) {
                        return Ok(ms);
                    }
                }
            }
        }

        // Try EXIF datetime+subsec pairs
        let exif_pairs = [
            ("DateTimeOriginal", "SubSecTimeOriginal"),
            ("CreateDate",       "SubSecTimeDigitized"),
            ("ModifyDate",       "SubSecTime"),
        ];
        for (dt_tag, subsec_tag) in exif_pairs {
            if let Ok(val) = ex.read_tag::<serde_json::Value>(path, dt_tag, &[]) {
                if let Some(s) = val.as_str() {
                    if let Ok(ms) = exif_datetime_to_ms(s) {
                        let subsec_ms = ex.read_tag::<serde_json::Value>(path, subsec_tag, &[])
                            .ok()
                            .and_then(|v| v.as_str().map(|s| s.to_string()))
                            .and_then(|s| {
                                // "907" -> 907ms, "91" -> 910ms, "9" -> 900ms
                                let truncated = &s[..s.len().min(3)];
                                let padded = format!("{:0<3}", truncated);
                                padded.parse::<u64>().ok()
                            })
                            .unwrap_or(0);
                        return Ok(ms + subsec_ms);
                    }
                }
            }
        }

        // Video fallbacks (no subsecond support in QuickTime integer timestamps)
        for tag in ["MediaCreateDate", "TrackCreateDate", "Keys:CreationDate", "FileModifyDate"] {
            if let Ok(val) = ex.read_tag::<serde_json::Value>(path, tag, &[]) {
                if let Some(s) = val.as_str() {
                    if let Ok(ms) = exif_datetime_to_ms(s) {
                        return Ok(ms);
                    }
                }
            }
        }

        anyhow::bail!("No creation time found in {}", path.display())

}





pub fn get_device_id(path: &PathBuf, ex: &ExifTool) -> Result<DeviceId> {
    // filename takes priority
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        if let Some((_, rest)) = stem.split_once("DEVICE_ID") {
            if let Some((id, _)) = rest.split_once("TIMESTAMP") {
                if let Ok(n) = id.trim().parse::<u16>() {
                    return Ok(DeviceId::ClientId(n));
                }else {
                    return Ok(DeviceId::CameraId(id.trim().to_string()));
                }
            }
        }
    }
    let tags = [
    "Make",
    "Model",
    "Software",
    "Artist",
    "OwnerName",
    "OwnerID",
    "CameraOwnerName",
    "SerialNumber",
    "SerialNumber2",
    "BodySerialNumber",
    "DeviceSerialNumber",
    "BodyID",
    "CanonModelID",
    "InternalSerialNumber",
    "InternalSerialNumber2",
    "CameraType",
    "SonyDeviceID",
    "MachineID",
    ];
    let normalize_tag_value = |raw: &str| {
        raw.split_whitespace().collect::<Vec<_>>().join(" ")
    };

    let prefix: Vec<String> = tags
        .iter()
        .filter_map(|&tag| {
            ex.read_tag::<serde_json::Value>(path, tag, &[])
                .ok()
                .and_then(|a| match a {
                    serde_json::Value::Null => None,
                    serde_json::Value::String(s) if s.is_empty() => None,
                    serde_json::Value::String(s) => {
                        let cleaned = normalize_tag_value(&s);
                        if cleaned.is_empty() { None } else { Some(cleaned) }
                    }
                    other => {
                        let cleaned = normalize_tag_value(&format!("{other}"));
                        if cleaned.is_empty() { None } else { Some(cleaned) }
                    }
                })
        }).collect();


    Ok(DeviceId::CameraId(filenamify::filenamify(format!("{}", prefix.join("-")))))
}




/// Writes a custom metadata key-value pair to a file using ExifTool.
/// Values are stored under the XMP-xmp namespace as "XMP-xmp:Description"
/// or a custom namespace tag like "XMP-custom:MyTag".
///
/// Uses the `-overwrite_original` flag to avoid creating backup files.
pub fn write_custom_metadata(
    path: &PathBuf,
    key: &str,
    value: &str,
    ex: &ExifTool,
) -> Result<()> {
    // Sanitize key: only allow alphanumeric and underscores to prevent injection
    let key: String = key.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
        .collect();

    let tag = format!("XMP-custom:{key}");
    ex.write_tag(path, &tag, value, &["-overwrite_original"])
        .with_context(|| format!("failed to write metadata tag '{tag}' to {}", path.display()))
}

/// Reads a custom metadata value from a file by key.
/// Returns `None` if the tag is absent or empty.
pub fn read_custom_metadata(
    path: &PathBuf,
    key: &str,
    ex: &ExifTool,
) -> Result<Option<String>> {
    anyhow::ensure!(
        key.chars().all(|c| c.is_alphanumeric() || c == '_'),
        "metadata key must be alphanumeric (got: {key})"
    );

    let tag = format!("XMP-custom:{key}");
    match ex.read_tag::<serde_json::Value>(path, &tag, &[]) {
        Ok(serde_json::Value::String(s)) if !s.trim().is_empty() => Ok(Some(s)),
        Ok(serde_json::Value::Null) | Err(_) => Ok(None),
        Ok(other) => {
            // Coerce non-string values (numbers, bools, etc.) to string
            Ok(Some(other.to_string()))
        }
    }
}

/// Reads all custom XMP-custom metadata tags from a file as key-value pairs.
pub fn read_all_custom_metadata(
    path: &PathBuf,
    ex: &ExifTool,
) -> Result<std::collections::HashMap<String, String>> {
    let raw = ex
        .read_tag::<serde_json::Value>(path, "XMP-custom:all", &["-j"])
        .with_context(|| format!("failed to read XMP-custom tags from {}", path.display()))?;

    let mut map = std::collections::HashMap::new();

    if let serde_json::Value::Object(obj) = raw {
        for (k, v) in obj {
            let value_str = match v {
                serde_json::Value::String(s) if !s.trim().is_empty() => s,
                serde_json::Value::Null => continue,
                other => other.to_string(),
            };
            map.insert(k, value_str);
        }
    }

    Ok(map)
}

/// Deletes a custom metadata key from a file by setting it to an empty value.
pub fn delete_custom_metadata(
    path: &PathBuf,
    key: &str,
    ex: &ExifTool,
) -> Result<()> {
    anyhow::ensure!(
        key.chars().all(|c| c.is_alphanumeric() || c == '_'),
        "metadata key must be alphanumeric (got: {key})"
    );

    let tag = format!("XMP-custom:{key}=");  // empty RHS = delete in ExifTool
    ex.write_tag(path, &tag, "", &["-overwrite_original"])
        .with_context(|| format!("failed to delete metadata tag '{key}' from {}", path.display()))
}
