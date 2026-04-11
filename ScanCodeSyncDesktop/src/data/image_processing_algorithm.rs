pub fn selective_blur(data: &mut [u8], width: usize, height: usize) {
    let original = data.to_vec(); // Copy to read from while writing to 'data'

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 3;

            // Skip processing if the center pixel is black
            if original[idx] == 0 && original[idx+1] == 0 && original[idx+2] == 0 {
                continue;
            }

            let mut r_sum: u32 = 0;
            let mut g_sum: u32 = 0;
            let mut b_sum: u32 = 0;
            let mut count: u32 = 0;

            // 3x3 Kernel
            for ky in -1..=1 {
                for kx in -1..=1 {
                    let py = y as isize + ky;
                    let px = x as isize + kx;

                    // Bounds check
                    if py >= 0 && py < height as isize && px >= 0 && px < width as isize {
                        let k_idx = (py as usize * width + px as usize) * 3;
                        let r = original[k_idx];
                        let g = original[k_idx+1];
                        let b = original[k_idx+2];

                        // ONLY add to average if pixel is NOT solid black
                        if r > 0 || g > 0 || b > 0 {
                            r_sum += r as u32;
                            g_sum += g as u32;
                            b_sum += b as u32;
                            count += 1;
                        }
                    }
                }
            }

            if count > 0 {
                data[idx] = (r_sum / count) as u8;
                data[idx+1] = (g_sum / count) as u8;
                data[idx+2] = (b_sum / count) as u8;
            }
        }
    }
}
