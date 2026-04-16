use crate::{data::{data_entry::{DataValue, DeviceId, DeviceTime, TimelineEntry}, image_processing_algorithm::selective_blur}, dbp};

use rayon::iter::{ParallelBridge, ParallelIterator};

// const COLORS: &[(&str, (u8, u8, u8))] = &[
//     ("red",     (255, 0,   0)),
//     ("green",   (0,   255, 0)),
//     ("blue",    (0,   0,   255)),
//     ("yellow",  (255, 255, 0)),
//     ("orange",  (255, 165, 0)),
//     ("purple",  (128, 0,   128)),
//     // ("pink",    (255, 192, 203)),
//     ("white",   (255, 255, 255)),
//     ("black",   (0,   0,   0)),
//     // ("gray",    (128, 128, 128)),
// ];


fn pre_process(data: &mut [u8], height: u32, width: u32) {

    // let id = format!("debug_data/{}{}{}.png", data[0],data[1],data[2]);

    selective_blur(data, width as usize, height as usize);
    selective_blur(data, width as usize, height as usize);
    selective_blur(data, width as usize, height as usize);

    data.chunks_exact_mut(3).par_bridge().for_each(|x| {
        let lm = (((150 * x[0] as u16) + (100 * x[1] as u16) + (5 * x[2] as u16)) >> 8) as u8;

        if lm < 30 {
            x[0] = 0;
            x[1] = 0;
            x[2] = 0;
        }else {
            if x[0] > x[1].saturating_add(x[2]) {
                x[0] = 255;
                x[1] = 0;
                x[2] = 0;
            }else if x[1] / 3 > x[2] / 2 {
                x[0] = 0;
                x[1] = 255;
                x[2] = 0;
            }else if lm > 50 {
                x[0] = 255;
                x[1] = 255;
                x[2] = 255;
            }
        }
    });

    selective_blur(data, width as usize, height as usize);

    #[cfg(debug_assertions)]
        {

            // let img: RgbImage = ImageBuffer::from_raw(width, height, data.to_vec())
            //     .expect("Failed to create image from buffer");

            // img.save(&id).expect("Failed to save debug image");
            // open::that(&id).expect("Failed to open image");
        }
}

fn closest_color(r: u8, g: u8, b: u8) -> &'static str {

    if (r,g,b) == (0,0,0) {return "black"}
    if (r,g,b) == (255,0,0) {return "red"}
    if (r,g,b) == (0,255,0) {return "green"}
    return "white";

    // let target: Lab = Srgb::new(r as f32 / 255., g as f32 / 255., b as f32 / 255.)
    //     .into_color();

    // COLORS.iter()
    //     .min_by_key(|(_, (cr, cg, cb))| {
    //         let c: Lab = Srgb::new(*cr as f32 / 255., *cg as f32 / 255., *cb as f32 / 255.)
    //             .into_color();
    //         (target.delta_e(c) * 1000.) as u32
    //     })
    //     .map(|(name, _)| *name)
    //     .unwrap()


}
/// this needs to be completly re written
pub fn detect_barcode_1d(lst: &[(u8,u8,u8)]) -> Option<(u64, u16)> {

    let colors:Vec<&str> =  lst.iter().map(|x| closest_color(x.0, x.1, x.2)).collect();


    let mut red = 0_i32;
    let mut red_start = 0_usize;
    let mut green = 0_i32;
    let mut last = "none";
    // start index, end index, green size
    let mut strips: Vec<(usize, usize, i32)> = vec![];
    for (i, c) in colors.iter().enumerate() {
        match c {
            &"red" => {
                if last == "red" {
                    red += 1;
                }else {
                    red = 1;
                    red_start = i;
                }
            }
            &"green" => {
                if last == "green" {
                    green += 1;
                }else {
                    green = 1;
                }
            }
            &"black" | &"white" => {}
            _ => {
                if last == "green" {
                    if green > 0
                        && red > 0
                        // && (green-red).abs() <= (red) as i32
                        {
                            strips.push((red_start, i, red));
                    }
                }
                red = 0;
                green = 0;

            }
        }
        last = c;
    }
    strips.push((red_start, colors.len() -1 , red));

    /// 64 = size of time, 16 = device id, 8 = size of color start and end zones;
    /// (64 + 16 + 8) = total number of bars on barcode, including the colored ones
    /// sorts by how well the total size matches the expected size of the green start zone
    fn get_ratio_match_score(x: (usize, usize, i32)) -> i32 {
        // this is a pretty dirty fix: todo: better solution, figure out why this can happen
        if x.2 == 0 {
            return 1000;
        }
        ((x.1 - x.0) as i32 / (x.2 / 4).max(1) - (64 + 16 + 8)).abs()
    }
    strips.sort_by_key(|x| get_ratio_match_score(*x));
    // bring the closest to the front
    strips.reverse();

    if strips.len() > 0 {
        let (start, end, green_size) = strips[0];
        if get_ratio_match_score(strips[0]) <= green_size / 10 {
            const FORMAT_CHUNKS: usize = 4 + 64 + 16 + 4;
            let chunk_size = (end - start) / FORMAT_CHUNKS;
            // it would be good to avoid this collect
            let chunks: Vec<&[&str]> = colors[start..end].chunks(chunk_size).collect();
            let timecode_colors = &chunks[4..68];   // 64 chunks
            let device_id       = &chunks[68..84];  // 16 chunks

            let timecode_u64: u64 = timecode_colors.iter()
                .enumerate()
                .fold(0u64, |acc, (i, x)| {
                    let bit = x[x.len() / 2] == "white";
                    acc | ((bit as u64) << (i))
                });
            let id_u16: u16 = device_id.iter()
                .enumerate()
                .fold(0u16, |acc, (i, x)| {
                    let bit = x[x.len() / 2] == "white";
                    acc | ((bit as u16) << (i))
                });
            return Some((timecode_u64, id_u16));
        }
    }

    return None;
}

/// It would be best to avoid cloning this data
pub fn get_h_row<'a>(row_to_get: u32, width: u32, data: &'a [u8]) -> Vec<(u8,u8,u8)> {
    let row = row_to_get as usize;
    let width = width as usize;
    let start = row * width * 3;
    data[start..start + width * 3]
        .chunks_exact(3)
        .map(|c| (c[0], c[1], c[2]))
        .collect()
}

pub fn get_v_col(col_to_get: u32, height: u32, width: u32, data: &[u8]) -> Vec<(u8, u8, u8)> {
    let col = col_to_get as usize;
    let height = height as usize;
    let width = width as usize;
    let stride = width * 3; // Number of bytes to skip to reach the next row

    (0..height)
        .map(|row| {
            let start = (row * stride) + (col * 3);
            (data[start], data[start + 1], data[start + 2])
        })
        .collect()
}


pub fn detect_barcodes(data: &mut [u8], width: u32, height: u32, device_time: &DeviceTime) -> anyhow::Result<Vec<TimelineEntry>>  {

    // pre_process(data, height, width);

    let mut center_strip: Vec<u8> = (0..10).into_iter().flat_map(|x| get_h_row(height/2 + x, width, &data))
            .flat_map(|(r, g, b)| [r, g, b])
            .collect();
    let mut vertical_center_strip: Vec<u8> = (0..10).into_iter()
        .flat_map(|x| get_v_col(width/2 + x, height, width, &data))
        .flat_map(|(r, g, b)| [r, g, b])
        .collect();


    pre_process(center_strip.as_mut(), 10, width);
    pre_process(vertical_center_strip.as_mut(), 10, height);

    let barcode_row = detect_barcode_1d(get_h_row(5, width, &center_strip).as_slice());
    let barcode_col = detect_barcode_1d(get_h_row(5, height, &vertical_center_strip).as_slice());

    let mut potential_readings = vec![];


    for barcode in [barcode_row, barcode_col] {
     if let Some((timecode, id)) = barcode {
         potential_readings.push(
             TimelineEntry {
                time: DeviceTime { internal_clock: timecode, device_id: DeviceId::ClientId(id) },
                val: DataValue::ClockOffset(device_time.clone()),
            }
         );
         crate::dbp!("managed to read barcode");
     }
    }

    return Ok(potential_readings);
}
