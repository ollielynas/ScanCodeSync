use crate::data::data_entry::{DataValue, DeviceId, DeviceTime, TimelineEntry};

use palette::{Lab, Srgb, color_difference::DeltaE, IntoColor};

const COLORS: &[(&str, (u8, u8, u8))] = &[
    ("red",     (255, 0,   0)),
    ("green",   (0,   255, 0)),
    ("blue",    (0,   0,   255)),
    ("yellow",  (255, 255, 0)),
    ("orange",  (255, 165, 0)),
    ("purple",  (128, 0,   128)),
    // ("pink",    (255, 192, 203)),
    ("white",   (255, 255, 255)),
    ("black",   (0,   0,   0)),
    // ("gray",    (128, 128, 128)),
];

fn closest_color(r: u8, g: u8, b: u8) -> &'static str {
    let target: Lab = Srgb::new(r as f32 / 255., g as f32 / 255., b as f32 / 255.)
        .into_color();

    COLORS.iter()
        .min_by_key(|(_, (cr, cg, cb))| {
            let c: Lab = Srgb::new(*cr as f32 / 255., *cg as f32 / 255., *cb as f32 / 255.)
                .into_color();
            (target.delta_e(c) * 1000.) as u32
        })
        .map(|(name, _)| *name)
        .unwrap()
}

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
            c => {
                if last == "green" {
                    if green > 0
                        && red > 0
                        && (green-red).abs() <= (red) as i32 {
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
        ((x.1 - x.0) as i32 / (x.2 / 4) - (64 + 16 + 8)).abs()
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
pub fn get_h_row<'a>(height: u32, width: u32, data: &'a [u8]) -> Vec<(u8,u8,u8)> {
    let row = height as usize;
    let width = width as usize;
    let start = row * width * 3;
    data[start..start + width * 3]
        .chunks_exact(3)
        .map(|c| (c[0], c[1], c[2]))
        .collect()
}
pub fn detect_barcodes(data: &[u8], width: u32, height: u32, device_time: &DeviceTime) -> anyhow::Result<Vec<TimelineEntry>>  {


    let mut potential_readings = vec![];

    let line_through_middle = detect_barcode_1d(&get_h_row(height/2, width, &data));

     if let Some((timecode, id)) = line_through_middle {
         potential_readings.push(
             TimelineEntry {
                time: DeviceTime { internal_clock: timecode, device_id: DeviceId::ClientId(id) },
                val: DataValue::ClockOffset(device_time.clone()),
            }
         );
         println!("managed to read barcode");
     }

    return Ok(potential_readings);
}
