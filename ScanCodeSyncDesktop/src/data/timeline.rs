
use egui_macroquad::egui::ahash::HashSet;

use crate::data::data_entry::DataEntry;




pub struct Timeline {
    pub entries: HashSet<DataEntry>,
}


impl Default for Timeline {
    fn default() -> Timeline {
        Timeline {
            entries: HashSet::default(),
        }
    }
}
