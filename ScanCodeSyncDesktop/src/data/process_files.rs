use std::{fs, path::PathBuf};
use anyhow::{anyhow, Context};
use atomic_progress::Progress;
use ffmpeg_sidecar::{self, command::FfmpegCommand, event::FfmpegEvent};

use crate::data::{data_entry::{DataEntry, DataValue, DeviceId, DeviceTime}, file_metadata::{self, get_creation_time_ms, get_device_id}};


fn process_csv_text(text: String) -> Vec<DataEntry> {
    return text.lines().map(|x| DataEntry::from_csv_row(x)).filter(|x| x.is_ok()).map(|x| x.unwrap()).collect();
}

fn process_raw_file(path: PathBuf) -> anyhow::Result<Vec<DataEntry>> {
    let entries = vec![];

    let creation_time = get_creation_time_ms(&path)?;
    let camera_id = get_device_id(&path)?;

    FfmpegCommand::new()
            .input(path.to_str().context("path contained non utf8 chars")?)
            .rawvideo()          // shorthand for: -f rawvideo -pix_fmt rgb24 pipe:1
            .spawn()
            .unwrap()
            .iter()
            .unwrap()
            .for_each(|event| {

                if let FfmpegEvent::OutputFrame(frame) = event {



                    let time = DeviceTime {
                        /// I should try to avoid this clone
                        device_id: camera_id.clone(),
                        internal_clock: creation_time + (frame.timestamp * 1000.0).round() as u64,
                    };




                }
            });
    return Ok(entries);
}

/// generates timeline data entries from file and them moves it into the "processed but unsorted" folder
pub fn attempt_process_file(path: PathBuf, progress: Progress) -> anyhow::Result<Vec<DataEntry>> {
    let filename = path.file_name().ok_or(anyhow::anyhow!("file has no filename"))?;
    let filetype = path.extension().ok_or(anyhow::anyhow!("file has no type"))?;

    progress.set_item(format!("processing: {filename:?}"));
    match (filetype.to_str(), filename) {
        (Some(".txt")|Some(".csv"), _) => {
            progress.bump();
            return anyhow::Ok(process_csv_text(fs::read_to_string(path)?));
        }
        (filetype, name) => {
            progress.bump();
            return process_raw_file(path);
        }
    }

    // anyhow::bail!("failed to process file");
}
