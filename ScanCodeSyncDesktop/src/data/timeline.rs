/// this file should be better named to reflect how it works

use anyhow::Context;
use atomic_progress::Progress;
// use petgraph::graph::UnGraph;
// use petgraph::stable_graph::StableGraph;

use std::{collections::BTreeSet, path::PathBuf};

use egui_macroquad::egui::ahash::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use crate::{command_pool::{SharedCommandPool}, data::{data_entry::{DataValue, DeviceId, DeviceTime, TimelineEntry}, file_metadata::get_device_id, lin_algebra::{get_global_time, solve_clock_drift_multipliers}}, dbp, state::SaveLocationOptions, util::{epoch_ms_to_date, get_base_media_type, get_mime_type}};


#[derive(Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub entries: HashSet<TimelineEntry>,

    /// represents chains of connected nodes
    /// binary tree to keep values sorted
    pub time_chains: HashMap<DeviceId, BTreeSet<u64>>,

    /// represents connections between chains
    pub chain_connections: HashSet<(DeviceTime, DeviceTime)>,

    pub metadata: HashSet<TimelineEntry>,

}


impl Default for Timeline {
    fn default() -> Timeline {
        Timeline {
            entries: HashSet::default(),

            time_chains: HashMap::default(),
            chain_connections: HashSet::default(),

            metadata: HashSet::default(),
        }
    }
}




impl Timeline {
    /// todo: load from file
    pub fn new() -> Timeline {
        Timeline::default()
    }

    pub fn sort_files(&mut self, output_files_folder: &PathBuf, progress: Progress, location_options: SaveLocationOptions, pool: SharedCommandPool) -> anyhow::Result<()> {

        self.sort_entries();
        progress.set_total(self.metadata.len() as u64);

        let ex = pool.get_exif_command()?;

        let (adjusted_clock_measurements, offset) = solve_clock_drift_multipliers(self.time_chains.clone(), self.chain_connections.clone());

        let mut camera_renames: HashMap<DeviceId, String> = HashMap::default();

        let mut sorted =  self.metadata.drain().map(|x| {
           (get_global_time(&adjusted_clock_measurements, &x.time).unwrap_or(f64::MAX), x.val)
        }).collect::<Vec<(f64, DataValue)>>();
        sorted.sort_by_key(|x| x.0 as u64);
        let mut current_production = "undefined production".to_string();

        let mut scene_number = 0;
        let mut scene_enabled = false;
        let mut current_scene = "scene undefined".to_string();

        let mut take_enabled = false;
        let mut current_take = 0;

        for (time, value) in sorted {
            dbp!("{} {:?}", time, value);
            match value {
                DataValue::RenameDevice((device_id, new_name)) => {
                    camera_renames.insert(device_id, new_name);
                },
                DataValue::IsMaster(_) => {
                    dbp!("deprecated")
                },
                DataValue::IsDirector(_) => {
                },
                DataValue::IsOperator(_) => {
                    dbp!("deprecated, this no longer needs to be recorded")
                },
                DataValue::ProductionName(n) => {
                    current_production = n;
                },
                DataValue::EnableOperatorName(_) => {
                    dbp!("setting op name is deprecated")
                },
                DataValue::OperatorName(_) =>  {},
                DataValue::EnableSceneName(n) => {
                    scene_enabled = n;
                },
                DataValue::SceneName(n) => {
                    if n!=current_scene {
                        current_scene = n;
                        scene_number += 1;
                    }
                },
                DataValue::EnableTakeNumber(n) => {take_enabled = n},
                DataValue::TakeNumber(n) => {current_take = n},
                DataValue::ClockOffset(_device_time) => {
                    dbp!("this should not be reached but I ant about be panicking about it")
                },
                DataValue::MediaCreated(input_path) => {
                    let mut output_path = output_files_folder.clone();

                    output_path.push(&current_production);

                    if location_options.date && ((time as u64).saturating_add(offset as u64) < (u64::MAX))  {
                        let date = epoch_ms_to_date(time as u128 + offset as u128);
                        output_path.push(format!("{} {}",&date, filenamify::filenamify(chrono::Local::now().format("%Z").to_string())));
                    }

                    if location_options.file_type {
                        let base_type = get_base_media_type(&input_path);
                        output_path.push(base_type);
                    }



                    if scene_enabled && location_options.scene {
                        output_path.push(format!("{:03} Scene - {}", scene_number, filenamify::filenamify(&current_scene)));
                    }
                    if take_enabled && location_options.take {
                        output_path.push(format!("Take {}", current_take));
                    }

                    if let Ok(id) = get_device_id(&input_path, &ex) {
                        let id_string = match camera_renames.get(&id) {
                            Some(a) => a.to_owned(),
                            None => id.to_string(),
                        };
                        output_path.push(format!("Device id - {}", id_string));
                    }

                    dbp!("output path: {:?},", output_path);


                    std::fs::create_dir_all(&output_path).context(format!("failed to create folder {:?}", output_path))?;

                    let mut filename = input_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                        if let Some((_, n)) = filename.split_once("TIMESTAMP") {
                            if let Some((_, n2)) = n.split_once("-") {
                                filename = n2.to_string();
                            }
                        }

                    output_path.push(filename);

                    match std::fs::copy(&input_path, &output_path) {
                        Ok(_) => {
                            match std::fs::remove_file(input_path) {
                                Ok(_) => {},
                                Err(a) => crate::dbp!("failed to delete file {:?}", a),
                            }
                        },
                        Err(_) => {},
                    };
                },

            }
            progress.bump();
        }

        pool.return_exif_command(ex);
        Ok(())
    }


    pub fn sort_entries(&mut self) {

        for n in self.entries.drain() {
            match n {
                TimelineEntry{time: time1, val: DataValue::ClockOffset(time2)} => {

                    if !self.time_chains.contains_key(&time1.device_id) {
                        self.time_chains.insert(time1.device_id.clone(), BTreeSet::new());
                    }
                    let chain1 = self.time_chains.get_mut(&time1.device_id).unwrap();
                    chain1.insert(time1.internal_clock);

                    if !self.time_chains.contains_key(&time2.device_id) {
                        self.time_chains.insert(time2.device_id.clone(), BTreeSet::new());
                    }
                    let chain2 = self.time_chains.get_mut(&time2.device_id).unwrap();
                    chain2.insert(time2.internal_clock);

                    self.chain_connections.insert((time1, time2));
                    // self.time_chains
                }

                n => {
                    self.metadata.insert(n);
                }
            }
        }
    }
}
