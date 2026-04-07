use std::{fs, path::PathBuf, time::Duration};
use anyhow::{anyhow, Context};
use atomic_progress::Progress;
use exiftool::ExifTool;
use ffmpeg_sidecar::{self, command::FfmpegCommand, event::FfmpegEvent};
use image::{GrayImage, ImageBuffer};
use rqrr::PreparedImage;

use crate::data::{data_entry::{DataValue, DeviceId, DeviceTime, TimelineEntry}, file_metadata::{self, get_creation_time_ms, get_device_id}, get_barcode::detect_barcodes};


fn process_csv_text(text: String) -> Vec<TimelineEntry> {
    return text.lines().map(|x| TimelineEntry::from_csv_row(x)).filter(|x| x.is_ok()).map(|x| x.unwrap()).collect();
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
        .filter_map(|g| g.decode().ok())
        .map(|(_, content)| content)
        .collect()
}


fn process_raw_file(path: &PathBuf, ex: &ExifTool ) -> anyhow::Result<Vec<TimelineEntry>> {
    let mut entries = vec![];
    let mut frame_number = 0;
    let creation_time = get_creation_time_ms(&path, &ex)?;
    let camera_id = get_device_id(&path, &ex)?;
    println!("{} {:?}", creation_time, camera_id);
    FfmpegCommand::new()
            .input(path.to_str().context("path contained non utf8 chars")?)
            .rawvideo()          // shorthand for: -f rawvideo -pix_fmt rgb24 pipe:1
            .create_no_window()

            .duration("45")
            .args(["-vf", "fps=10,scale=1080:-1"])
            .spawn()?
            .iter()?
            .for_each(|event| {
                if let FfmpegEvent::OutputFrame(frame) = event {
                    println!("frame found");
                    frame_number += 1;

                    // todo: band aid optomisatio that needs to be chnaged
                    for s in scan_qr(&frame.data, frame.width, frame.height) {
                        entries.append(&mut process_csv_text(s));
                    }


                    let time = DeviceTime {
                        /// I should try to avoid this clone
                        device_id: camera_id.clone(),
                        internal_clock: creation_time + (frame.timestamp * 1000.0).round() as u64,
                    };
                    let barcode_res = detect_barcodes(&frame.data, frame.width, frame.height, &time);
                    println!("res{:?}", barcode_res);
                    if let Ok(mut barcode_data) = barcode_res {
                        entries.append(&mut barcode_data);
                    }




                }
            });
    return Ok(entries);
}

/// generates timeline data entries from file and them moves it into the "processed but unsorted" folder
pub fn attempt_process_file(path: &PathBuf, progress: Progress, ex: &ExifTool) -> anyhow::Result<Vec<TimelineEntry>> {
    let filename = path.file_name().ok_or(anyhow::anyhow!("file has no filename"))?;
    let filetype = path.extension().ok_or(anyhow::anyhow!("file has no type"))?;

    progress.set_item(format!("processing: {filename:?}"));
    match (filetype.to_str(), filename) {
        (Some(".txt")|Some(".csv"), _) => {
            progress.bump();
            return anyhow::Ok(process_csv_text(fs::read_to_string(path)?));
        }
        (_filetype, _name) => {
            progress.bump();
            let new_entries = process_raw_file(path, &ex);
            // println!("new entries {:?}", new_entries);
            return new_entries;
        }
    }

    // anyhow::bail!("failed to process file");
}
