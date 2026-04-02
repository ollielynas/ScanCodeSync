use crate::{task::Task, tasks::{change_input_folder::ChangeInputFolderTask, change_output_folder::ChangeOutputFolderTask, import_media_task::ImportMediaTask, init_task::InitTask, update_input_file_list_task::UpdateInputFileListTask}};
use std::sync::LazyLock;

pub fn build_init_task() -> Box<dyn Task> {
    return Box::new(InitTask::default());
}
pub fn build_update_input_files_list_task() -> Box<dyn Task> {
    return Box::new(UpdateInputFileListTask::default());
}
pub fn build_change_input_folder_task() -> Box<dyn Task> {
    return Box::new(ChangeInputFolderTask::default());
}
pub fn build_change_output_folder_task() -> Box<dyn Task> {
    return Box::new(ChangeOutputFolderTask::default());
}
pub fn build_import_media_task_folders() -> Box<dyn Task> {
    return Box::new(ImportMediaTask::new(false));
}
pub fn build_import_media_task_files() -> Box<dyn Task> {
    return Box::new(ImportMediaTask::new(true));
}


pub fn user_accessible_tasks() -> Vec<Box<dyn Task>> {
    return vec![
        build_import_media_task_files(),
        build_import_media_task_folders(),
        build_change_input_folder_task(),
        build_change_output_folder_task(),
    ];
}
