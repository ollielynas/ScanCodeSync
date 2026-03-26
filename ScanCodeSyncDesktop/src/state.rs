use std::path::PathBuf;
use crate::val_hold::*;

pub struct AppState {
    input_folder: ValueHolder<PathBuf>,
    output_folder: ValueHolder<PathBuf>,


    new_files: ValueHolder<Vec<PathBuf>>,

}
