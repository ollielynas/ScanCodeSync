use core::fmt;
use std::{fmt::Display, mem, path::PathBuf};

use anyhow::*;
use egui_macroquad::egui::ahash::HashSet;

#[derive(Clone)]
pub enum ValueHolder<T> where T: PlaceholderDisplayValue + Clone {
    Value(Box<T>),
    BackupValue(Box<T>, u64),
}

impl<T> ValueHolder<T> where T: PlaceholderDisplayValue + Clone {


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


    /// puts the inner value back into the holder. this should be used when the
    pub fn populate(&mut self, value: Box<T>) -> anyhow::Result<()> {
        match self {
            ValueHolder::Value(_) => {
                anyhow::bail!("value is already populated");
            },
            ValueHolder::BackupValue(_, _) => {
                let mut temp = ValueHolder::Value(value);
                std::mem::swap(&mut temp, self);
            },
        }
        return Ok(());
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


impl<T> ToString for ValueHolder<T> where T: PlaceholderDisplayValue + Clone {
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
