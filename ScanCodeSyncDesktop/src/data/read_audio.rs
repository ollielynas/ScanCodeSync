use std::path::PathBuf;

use ffmpeg_sidecar::command::FfmpegCommand;

use crate::data::data_entry::TimelineEntry;

/// TODO
pub fn get_encoded_time_from_audio(path: &PathBuf, _start_time_ms: u64, ) -> anyhow::Result<Vec<TimelineEntry>> {
    // let mut child = FfmpegCommand::new()
    //         .input(path.to_str().unwrap_or("invalid file path"))
    //         .args([
    //             "-ac", "1",           // Mono
    //             "-ar", "44100",       // Sample rate
    //             "-f", "s16le",        // Raw 16-bit Little Endian
    //             "-",                  // Output to stdout
    //         ])
    //         .spawn()?;

    //     let mut audio_buffer = Vec::new();

    //     for event in child.iter()? {
    //         if let FfmpegEvent::Data(bytes) = event {
    //             // 1. Convert bytes to f32 samples
    //             let mut samples: Vec<f32> = bytes.chunks_exact(2)
    //                 .map(|c| i16::from_le_bytes([c[0], c[1]]) as f32 / 32768.0)
    //                 .collect();

    //             audio_buffer.append(&mut samples);

    //             // 2. Process when we have enough samples (e.g., 2048 for precision)
    //             if audio_buffer.len() >= 2048 {
    //                 let spectrum = samples_fft_to_spectrum(
    //                     &audio_buffer[0..2048],
    //                     44100,
    //                     FrequencyLimit::All,
    //                     Some(&divide_by_N),
    //                 ).unwrap();

    //                 // 3. Check specific bins
    //                 let val_1200 = spectrum.get_value_at(1200.0);
    //                 let val_2200 = spectrum.get_value_at(2200.0);

    //                 // Thresholding: Is the signal strong enough above the noise?
    //                 if val_1200 > 0.5 { println!("Found 1200Hz!"); }
    //                 if val_2200 > 0.5 { println!("Found 2200Hz!"); }

    //                 audio_buffer.drain(0..1024); // Slide the window
    //             }
    //         }
    //     }

    return Ok(vec![]);
}
