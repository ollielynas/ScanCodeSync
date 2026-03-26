use std::path::PathBuf;

use anyhow::*;

pub enum ValueHolder<T> where T: PlaceholderDisplayValue {
    Value(Box<T>),
    PlaceholderText(String),
}


impl<T> ValueHolder<T> where T: PlaceholderDisplayValue {
    pub fn depopulate(&mut self) -> anyhow::Result<Box<T>> {
        match self {
            ValueHolder::Value(val) => {
                let mut placeholder = ValueHolder::PlaceholderText(val.placeholder_text());
                std::mem::swap(self, &mut placeholder);
                match placeholder {
                    ValueHolder::Value(val2) => {return anyhow::Ok(val2)}
                    _ => {unreachable!("how tf")},
                }
            },
            ValueHolder::PlaceholderText(text) => {
                anyhow::bail!("The value is being used, It has provided the following placeholder text: {}", text);
            }
        }
    }

    pub fn populate(&mut self, value: Box<T>) -> anyhow::Result<()> {
        match self {
            ValueHolder::Value(_) => {
                anyhow::bail!("value is already populated");
            },
            ValueHolder::PlaceholderText(_) => {
                let mut temp = ValueHolder::Value(value);
                std::mem::swap(&mut temp, self);
            },
        }
        return Ok(());
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
