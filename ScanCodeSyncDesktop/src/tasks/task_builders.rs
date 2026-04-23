use crate::{task::Task, tasks::{change_input_folder::ChangeInputFolderTask, change_output_folder::ChangeOutputFolderTask, import_media_task::ImportMediaTask, init_task::InitTask, install_exiftools::InstallExifToolsTask, install_ffmpeg::InstallFfmpegTask, install_magick::InstallMagickTask, process_input_files_task::ProcessInputFilesTask, process_unsorted_files::ProcessUnsortedFilesTask, reset_data_task::ResetDataTask, restart::RestartTask, update_input_file_list_task::UpdateInputFileListTask, update_unsorted_file_list::UpdateUnsortedFileListTask}};
use std::sync::LazyLock;

pub fn build_init_task() -> Box<dyn Task> {
    return Box::new(InitTask::default());
}
pub fn build_update_input_files_list_task() -> Box<dyn Task> {
    return Box::new(UpdateInputFileListTask::default());
}
pub fn build_update_unsorted_files_list_task() -> Box<dyn Task> {
    return Box::new(UpdateUnsortedFileListTask::default());
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
pub fn build_process_new_files_task() -> Box<dyn Task> {
    return Box::new(ProcessInputFilesTask::default());
}
pub fn build_process_unsorted_files_task() -> Box<dyn Task> {
    return Box::new(ProcessUnsortedFilesTask::default());
}
pub fn build_install_exiftools_task() -> Box<dyn Task> {
    return Box::new(InstallExifToolsTask::default());
}
pub fn build_install_magick_task() -> Box<dyn Task> {
    return Box::new(InstallMagickTask::default());
}
pub fn build_install_ffmpeg_task() -> Box<dyn Task> {
    return Box::new(InstallFfmpegTask::new());
}
pub fn build_restart_task() -> Box<dyn Task> {
    return Box::new(RestartTask::default());
}
pub fn build_reset_data_task() -> Box<dyn Task> {
    return Box::new(ResetDataTask::default());
}


pub fn user_accessible_tasks() -> Vec<Box<dyn Task>> {
    return vec![
        build_process_new_files_task(),
        build_process_unsorted_files_task(),
        build_import_media_task_files(),
        build_import_media_task_folders(),
        build_change_input_folder_task(),
        build_change_output_folder_task(),
        build_install_exiftools_task(),
        build_install_magick_task(),
        build_install_ffmpeg_task(),
        build_update_unsorted_files_list_task(),
        build_reset_data_task(),
    ];
}
