use egui_macroquad::egui::ahash::HashMap;
use egui_macroquad::egui::ahash::HashSet;
use nalgebra::{DMatrix, DVector};
use std::collections::BTreeSet;

use crate::data::data_entry::{DeviceId, DeviceTime};

/// For each device, returns its readings paired with their estimated global time.
/// The device with the most readings is used as the reference (rate=1, offset=0).
pub fn solve_clock_drift_multipliers(
    chains: HashMap<DeviceId, BTreeSet<u64>>,
    connections: HashSet<(DeviceTime, DeviceTime)>,
) -> HashMap<DeviceId, Vec<(u64, f64)>> {

    // -------------------------------------------------------------------------
    // Step 1: subtract the earliest timestamp to keep f64 precision high
    // -------------------------------------------------------------------------
    let base = chains.values()
        .flat_map(|s| s.iter())
        .min()
        .copied()
        .unwrap_or(0);

    // -------------------------------------------------------------------------
    // Step 2: pick the device with the most readings as the reference
    // -------------------------------------------------------------------------
    let reference_device = chains
        .iter()
        .max_by_key(|(_, readings)| readings.len())
        .map(|(device_id, _)| device_id.clone())
        .expect("no devices");

    // -------------------------------------------------------------------------
    // Step 3: flood fill from the reference to find all reachable devices
    //         anything not reachable is excluded — we can't solve for it
    // -------------------------------------------------------------------------
    let mut reachable: HashSet<DeviceId> = HashSet::default();
    let mut queue = vec![reference_device.clone()];

    while let Some(current) = queue.pop() {
        if reachable.contains(&current) {
            continue;
        }
        reachable.insert(current.clone());

        for (a, b) in &connections {
            if a.device_id == current && !reachable.contains(&b.device_id) {
                queue.push(b.device_id.clone());
            }
            if b.device_id == current && !reachable.contains(&a.device_id) {
                queue.push(a.device_id.clone());
            }
        }
    }

    // -------------------------------------------------------------------------
    // Step 4: filter everything down to reachable devices only
    //         also drop devices where all readings are the same value —
    //         a flat signal gives us no information about rate
    // -------------------------------------------------------------------------
    let chains: HashMap<DeviceId, BTreeSet<u64>> = chains
        .into_iter()
        .filter(|(device_id, readings)| {
            let is_reachable = reachable.contains(device_id);
            let has_spread = readings.iter().min() != readings.iter().max();
            is_reachable && has_spread
        })
        .collect();

    let connections: HashSet<(DeviceTime, DeviceTime)> = connections
        .into_iter()
        .filter(|(a, b)| {
            reachable.contains(&a.device_id) && reachable.contains(&b.device_id)
        })
        .collect();

    // -------------------------------------------------------------------------
    // Step 5: split devices into solvable (2+ readings) and single-reading
    //         solvable devices get both rate and offset estimated
    //         single-reading devices borrow the rate from their connected device
    // -------------------------------------------------------------------------
    let solvable: HashMap<DeviceId, BTreeSet<u64>> = chains
        .iter()
        .filter(|(_, readings)| readings.len() >= 2)
        .map(|(d, r)| (d.clone(), r.clone()))
        .collect();

    let single_reading: HashMap<DeviceId, u64> = chains
        .iter()
        .filter(|(_, readings)| readings.len() == 1)
        .map(|(d, r)| (d.clone(), *r.iter().next().unwrap()))
        .collect();

    // -------------------------------------------------------------------------
    // Step 6: assign each solvable device a stable index
    //         reference device is always index 0
    // -------------------------------------------------------------------------
    let devices: Vec<DeviceId> = std::iter::once(reference_device.clone())
        .chain(solvable.keys().filter(|d| *d != &reference_device).cloned())
        .collect();

    let device_index: HashMap<DeviceId, usize> = devices
        .iter()
        .enumerate()
        .map(|(i, d)| (d.clone(), i))
        .collect();

    let reference = 0;

    // -------------------------------------------------------------------------
    // Step 7: build the matrix — one row per simultaneous observation
    //
    //         each row encodes:
    //           rate_i * reading_i + offset_i = rate_j * reading_j + offset_j
    //
    //         the reference device (rate=1, offset=0) goes straight to the rhs
    // -------------------------------------------------------------------------
    let n_unknowns = (devices.len() - 1) * 2;
    let n_observations = connections.len();

    let mut matrix = DMatrix::<f64>::zeros(n_observations, n_unknowns);
    let mut rhs = DVector::<f64>::zeros(n_observations);

    for (row, (a, b)) in connections.iter().enumerate() {
        // skip if either device isn't in the solvable set
        let Some(&idx_a) = device_index.get(&a.device_id) else { continue };
        let Some(&idx_b) = device_index.get(&b.device_id) else { continue };

        let reading_a = (a.internal_clock - base) as f64;
        let reading_b = (b.internal_clock - base) as f64;

        let mut fill = |device_idx: usize, reading: f64, sign: f64| {
            if device_idx == reference {
                rhs[row] -= sign * reading;
            } else {
                let rate_col   = 2 * (device_idx - 1);
                let offset_col = 2 * (device_idx - 1) + 1;
                matrix[(row, rate_col)]   += sign * reading;
                matrix[(row, offset_col)] += sign;
            }
        };

        fill(idx_a, reading_a,  1.0);
        fill(idx_b, reading_b, -1.0);
    }

    // -------------------------------------------------------------------------
    // Step 8: solve via SVD least squares
    // -------------------------------------------------------------------------
    let svd = matrix.svd(true, true);
    let solution = svd
        .solve(&rhs, 1e-10)
        .expect("could not solve — check that all devices are connected");

    // -------------------------------------------------------------------------
    // Step 9: extract rate and offset per solvable device
    // -------------------------------------------------------------------------
    let rate_offset: HashMap<DeviceId, (f64, f64)> = devices
        .iter()
        .enumerate()
        .map(|(idx, device_id)| {
            let transform = if idx == reference {
                (1.0, 0.0)
            } else {
                let rate   = solution[2 * (idx - 1)];
                let offset = solution[2 * (idx - 1) + 1];
                (rate, offset)
            };
            (device_id.clone(), transform)
        })
        .collect();

    // -------------------------------------------------------------------------
    // Step 10: convert solvable device readings to global time
    // -------------------------------------------------------------------------
    let mut result: HashMap<DeviceId, Vec<(u64, f64)>> = solvable
        .iter()
        .map(|(device_id, readings)| {
            let (rate, offset) = rate_offset[device_id];
            let converted = readings
                .iter()
                .map(|&reading| {
                    let global_time = rate * (reading - base) as f64 + offset;
                    (reading, global_time)
                })
                .collect();
            (device_id.clone(), converted)
        })
        .collect();

    // -------------------------------------------------------------------------
    // Step 11: handle single-reading devices
    //          borrow the rate from the connected device,
    //          use the one simultaneous observation to solve for offset
    // -------------------------------------------------------------------------
    for (device_id, reading) in &single_reading {
        let connected = connections.iter().find_map(|(a, b)| {
            if a.device_id == *device_id {
                Some((&b.device_id, b.internal_clock, a.internal_clock))
            } else if b.device_id == *device_id {
                Some((&a.device_id, a.internal_clock, b.internal_clock))
            } else {
                None
            }
        });

        if let Some((other_device, other_reading, this_reading)) = connected {
            if let Some(&(borrowed_rate, other_offset)) = rate_offset.get(other_device) {
                // global time at the moment of connection, as seen by the other device
                let global_at_connection = borrowed_rate * (other_reading - base) as f64 + other_offset;

                // work backwards to find what offset makes this device line up
                let offset = global_at_connection - borrowed_rate * (this_reading - base) as f64;

                let global_time = borrowed_rate * (*reading - base) as f64 + offset;
                result.insert(device_id.clone(), vec![(*reading, global_time)]);
            }
        }
    }

    result
}

/// looks up the global time for a specific device reading.
/// returns None if the device or reading is not found.
/// looks up the global time for a specific device reading by interpolating
/// between the two nearest known readings on that device.
/// extrapolates if the reading is before or after all known readings.
pub fn get_global_time(
    adjusted_times: &HashMap<DeviceId, Vec<(u64, f64)>>,
    time: &DeviceTime,
) -> Option<f64> {
    let readings = adjusted_times.get(&time.device_id)?;

    if readings.is_empty() {
        return None;
    }

    let target = time.internal_clock;

    // find the first reading that is greater than the target
    let right = readings.iter().find(|(raw, _)| *raw > target);
    let left  = readings.iter().rev().find(|(raw, _)| *raw <= target);

    match (left, right) {
        // target sits between two known readings — interpolate
        (Some(&(left_raw, left_global)), Some(&(right_raw, right_global))) => {
            let t = (target - left_raw) as f64 / (right_raw - left_raw) as f64;
            Some(left_global + t * (right_global - left_global))
        }

        // target is after all known readings — extrapolate forward
        (Some(&(left_raw, left_global)), None) => {
            // need the previous point to get the rate
            let prev = readings.iter().rev().find(|(raw, _)| *raw < left_raw);
            if let Some(&(prev_raw, prev_global)) = prev {
                let rate = (left_global - prev_global) / (left_raw - prev_raw) as f64;
                Some(left_global + rate * (target - left_raw) as f64)
            } else {
                // only one reading — can't estimate rate, just return its global time
                Some(left_global)
            }
        }

        // target is before all known readings — extrapolate backward
        (None, Some(&(right_raw, right_global))) => {
            let next = readings.iter().find(|(raw, _)| *raw > right_raw);
            if let Some(&(next_raw, next_global)) = next {
                let rate = (next_global - right_global) / (next_raw - right_raw) as f64;
                Some(right_global - rate * (right_raw - target) as f64)
            } else {
                // only one reading — can't estimate rate, just return its global time
                Some(right_global)
            }
        }

        // no readings at all
        (None, None) => None,
    }
}
