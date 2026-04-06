
use egui_macroquad::egui::ahash::HashSet;
use serde::{Deserialize, Serialize};

use crate::data::data_entry::TimelineEntry;



#[derive(Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub entries: HashSet<TimelineEntry>,
}


impl Default for Timeline {
    fn default() -> Timeline {
        Timeline {
            entries: HashSet::default(),
        }
    }
}

impl Timeline {
    /// todo: load from file
    pub fn new() -> Timeline {
        Timeline::default()
    }
}
