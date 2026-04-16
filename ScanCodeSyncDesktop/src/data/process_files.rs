use std::{fs, path::PathBuf, process::{Command, Stdio}};
use anyhow::Context;
use atomic_progress::Progress;
use exiftool::ExifTool;
use ffmpeg_sidecar::{self, command::FfmpegCommand, event::FfmpegEvent};
use image::GrayImage;
use rqrr::PreparedImage;

use crate::{command_pool::SharedCommandPool, data::{data_entry::{DataValue, DeviceId, DeviceTime, TimelineEntry}, file_metadata::{get_creation_time_ms, get_device_id}}, util::FFMPEG_FRMATS};


fn process_csv_text(text: String, recording_device_id: Option<&DeviceId>) -> Vec<TimelineEntry> {
    return text.lines().map(|x| TimelineEntry::from_csv_row(x, recording_device_id)).filter(|x| x.is_ok()).map(|x| x.unwrap()).collect();
}


fn scan_qr(data: &[u8], width: u32, height: u32) -> Vec<String> {
    let pixel_count = (width * height) as usize;
    let mut gray = Vec::with_capacity(pixel_count);


    // this bit is responsible for a good chunk of the flame graph

    // Safety: chunks_exact already guarantees 3-byte alignment,
    // but a manual loop lets the compiler auto-vectorise (SIMD) more easily
    for i in 0..pixel_count {
        let base = i * 3;
        let luma = (data[base] as u32 * 77
            + data[base + 1] as u32 * 150
            + data[base + 2] as u32 * 29) >> 8;
        gray.push(luma as u8);
    }

    let gray_img = GrayImage::from_raw(width, height, gray)
        .expect("buffer dimensions don't match data length");

    let mut prepared = PreparedImage::prepare(gray_img);
    prepared
        .detect_grids()
        .iter()
        .filter_map(|g| {
            crate::dbp!("found grid {:?}", g.decode());
            g.decode().ok()
        })
        .map(|(_, content)| content)
        .collect()
}


fn process_timecodes_from_string(codes: Vec<String>) -> Option<DeviceTime> {
    let mut max_time: u64 = 0;
    let mut id: u16 = 0;
    for c in codes {
        let (a,b) = c.split_once("-")?;
        max_time = a.parse::<u64>().ok()?.max(max_time);
        id = b.parse::<u16>().ok()?;
    }

    if max_time != 0 {
        return Some(
            DeviceTime { internal_clock: max_time, device_id: DeviceId::ClientId(id) }
        )
    }else {
        return None;
    }
}

fn process_raw_data(data: &mut [u8], height: u32, width: u32, entries: &mut Vec<TimelineEntry>, camera_id: &DeviceId, creation_time: u64, timestamp: f32) {
    let time = DeviceTime {
        // I should try to avoid this clone
        device_id: camera_id.clone(),
        internal_clock: creation_time + (timestamp * 1000.0).round() as u64,
    };


    let mut timecode_qr_codes = vec![];

    for s in scan_qr(data, width, height) {
        if s.contains(",") {
            // assume csv data
            entries.append(&mut process_csv_text(s, Some(camera_id)));
            return;
        } else {
            // otherwise assume timecode data
            timecode_qr_codes.push(s);
        }
    }

    if let Some(scanned_time) = process_timecodes_from_string(timecode_qr_codes) {
        entries.push(
            TimelineEntry {
                time: scanned_time,
                val: DataValue::ClockOffset(time.clone()),
            }
        );
        // assume there is no qr code to scan at this point
        return;
    };


    // the barcode method is considered deprecated for now, but Thats only because its just not
    // reliable enough at the moment
    // let barcode_res = detect_barcodes(data, width, height, &time);
    // if let Ok(mut barcode_data) = barcode_res {
    //     entries.append(&mut barcode_data);
    // }


}

/// the main thing that needs to be optimised is re scanning the same qr code over and over.
fn process_raw_file(path: &PathBuf, command_pool: SharedCommandPool, filetype: String ) -> anyhow::Result<Vec<TimelineEntry>> {
    let mut entries = vec![];
    let ex = command_pool.get_exif_command()?;
    let creation_time = get_creation_time_ms(&path, &ex)?;
    let camera_id = get_device_id(&path, &ex)?;
    command_pool.return_exif_command(ex);
    if FFMPEG_FRMATS.contains(&filetype) {
        FfmpegCommand::new()

                .input(path.to_str().context("path contained non utf8 chars")?)
                .rawvideo()          // shorthand for: -f rawvideo -pix_fmt rgb24 pipe:1
                .create_no_window()

                .duration("45")
                .args(["-vf", "fps=10,scale=1080:-1,mpdecimate -loglevel debug"])
                .spawn()?
                .iter()?
                .for_each(|event| {
                    if let FfmpegEvent::OutputFrame(mut frame) = event {
                        process_raw_data(&mut frame.data, frame.height, frame.width, &mut entries, &camera_id, creation_time, frame.timestamp);
                    }
                });
    }else {
        let mut cmd = Command::new("magick");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        let output = cmd
            .args(["identify", "-format", "%w %h"])
            .arg(path)
            .output()
            .expect("Failed to execute command");

        let result = String::from_utf8_lossy(&output.stdout);
        let dims: Vec<&str> = result.split_whitespace().collect();

        let mut width: u32 = 0;
        let mut height: u32 = 0;
        if dims.len() == 2 {
                width = dims[0].parse().unwrap_or(0);
                height = dims[1].parse().unwrap_or(0);
        }

        if width > 0 && height > 0 {

            let mut cmd = Command::new("magick");
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x08000000);
            }

            let child = cmd
                .arg(&path)
                .args(["-depth", "8", "rgb:-"])
                .stdout(Stdio::piped())
                .spawn()?;

            if let Ok(mut output) = child.wait_with_output() {
                // let height = output.stdout.len() as u32 / (3 * 1080);
                process_raw_data(&mut output.stdout, height, width, &mut entries, &camera_id, creation_time, 0.0);
            }

        }


    }
    return Ok(entries);
}

/// generates timeline data entries from file and them moves it into the "processed but unsorted" folder
pub fn attempt_process_file(path: &PathBuf, progress: Progress, command_pool: SharedCommandPool) -> anyhow::Result<Vec<TimelineEntry>> {
    let filename = path.file_name().ok_or(anyhow::anyhow!("file has no filename"))?;
    let filetype = path.extension().ok_or(anyhow::anyhow!("file has no type"))?;

    progress.set_item(format!("processing: {filename:?}"));
    match (filetype.to_str(), filename) {
        (Some(".txt")|Some(".csv"), _) => {
            progress.bump();
            return anyhow::Ok(process_csv_text(fs::read_to_string(path)?, None));
        }
        (_filetype, _name) => {
            let new_entries = process_raw_file(path, command_pool, filetype.to_string_lossy().to_lowercase());
            progress.bump();
            return new_entries;
        }
    }

    // anyhow::bail!("failed to process file");
}
