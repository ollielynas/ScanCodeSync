use core::fmt;
use std::{fmt::Display, fs, mem, path::PathBuf};

use anyhow::*;
use directories::ProjectDirs;
use egui_macroquad::egui::ahash::HashSet;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{data::timeline::Timeline, state::SaveLocationOptions, util::get_project_dir};

#[derive(Clone, Serialize)]
pub enum ValueHolder<T> where T: PlaceholderDisplayValue + Clone + Serialize + DeserializeOwned   {
    Value(Box<T>),
    BackupValue(Box<T>, u64),
}

impl<T> ValueHolder<T> where T: PlaceholderDisplayValue + Clone + Serialize + DeserializeOwned   {


    /// this get the inner value of the holder if it is not being used. If it is being used then an error will be returned.
    pub fn depopulate(&mut self, task_id: u64) -> anyhow::Result<Box<T>> {
        match self {
            ValueHolder::Value(val) => {
                let mut placeholder = ValueHolder::BackupValue(val.clone(), task_id);
                std::mem::swap(self, &mut placeholder);
                match placeholder {
                    ValueHolder::Value(val2) => {return anyhow::Ok(val2)}
                    _ => {unreachable!("how tf")},
                }
            },
            ValueHolder::BackupValue(value, id) => {
                anyhow::bail!("The value is being used by: tasl:{}", id);
            }
        }
    }

    pub fn load_from_file(&mut self, name: &str) -> anyhow::Result<()> {
            let dir = get_project_dir()?;
            let path = dir.config_local_dir().join(format!("{}.json", name));

            if path.exists() {
                let json_data = fs::read_to_string(path)?;
                // Deserialise the JSON directly back into the expected type T
                let value: T = serde_json::from_str(&json_data)?;
                crate::dbp!("{}", value.placeholder_text());
                // Put it into the holder as a fresh 'Value' variant
                *self = ValueHolder::Value(Box::new(value));
            } else {
                crate::dbp!("path does not exist {:?}", path);
            }
            Ok(())
        }


    /// puts the inner value back into the holder. this should be used when the
    pub fn populate(&mut self, value: Box<T>, name: &str) -> anyhow::Result<()> {
        let dir = get_project_dir()?;

        let save_path = dir.config_local_dir().join(format!("{}.json", name));

        // 1. Serialize and save the specific field to its own file
        let json = serde_json::to_string_pretty(&value)?;
        let _  = std::fs::create_dir_all(dir.config_local_dir());
        std::fs::write(save_path, json)?;

        // 2. Your existing swap logic
        match self {
            ValueHolder::Value(_) => {
                anyhow::bail!("value is already populated");
            },
            ValueHolder::BackupValue(_, _) => {
                *self = ValueHolder::Value(value);
            },
        }
        Ok(())
    }

    pub fn available(&self) -> bool {
        matches!(self, ValueHolder::Value(_))
    }

    pub fn restore_if_dropped(&mut self, task_ids: &HashSet<u64>) {
        if let ValueHolder::BackupValue(val, id) = self {
            if !task_ids.contains(id) {
                *self = ValueHolder::Value(mem::replace(val, val.clone()));
            }
        }
    }

    pub fn used_by(&self) -> Option<u64> {
        match self {
            ValueHolder::Value(_) => None,
            ValueHolder::BackupValue(_, id) => Some(*id),
        }
    }
    pub fn used_by_string(&self) -> String {
        match self {
            ValueHolder::Value(_) => "".to_string(),
            ValueHolder::BackupValue(_, id) => (id % 999).to_string(),
        }
    }
}


impl<T> ToString for ValueHolder<T> where T: PlaceholderDisplayValue + Clone + Serialize + DeserializeOwned  {
    fn to_string(&self) -> String {
        return match self {
            ValueHolder::BackupValue(t, id) => t.placeholder_text(),
            ValueHolder::Value(v) => v.placeholder_text()
        }
    }
}



pub trait PlaceholderDisplayValue {
    fn placeholder_text(&self) -> String;
}

impl PlaceholderDisplayValue for PathBuf {
    fn placeholder_text(&self) -> String {
        return format!("{}", self.display())
    }
}
impl PlaceholderDisplayValue for Vec<PathBuf> {
    fn placeholder_text(&self) -> String {
        return format!("{} files", self.len())
    }
}

impl PlaceholderDisplayValue for Timeline {
    fn placeholder_text(&self) -> String {
        return format!("{} timeline entries", self.entries.len())
    }
}
impl PlaceholderDisplayValue for SaveLocationOptions {
    fn placeholder_text(&self) -> String {
        return format!("{:#?}", self)
    }
}




pub trait ValueHolderExt {
    fn available(&self) -> bool;
    fn restore_if_dropped(&mut self, task_ids: &HashSet<u64>);
}

impl<T: PlaceholderDisplayValue + Clone + Serialize + DeserializeOwned  > ValueHolderExt for ValueHolder<T> {
    fn available(&self) -> bool {
        self.available()
    }


    fn restore_if_dropped(&mut self, task_ids: &HashSet<u64>) {
        self.restore_if_dropped(task_ids);
    }
}
