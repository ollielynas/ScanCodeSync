use crate::{task::Task, tasks::{init_task::InitTask, update_input_file_list_task::UpdateInputFileListTask}};


pub fn build_init_task() -> Box<dyn Task> {
    return Box::new(InitTask::default());
}
pub fn build_update_input_files_list_task() -> Box<dyn Task> {
    return Box::new(UpdateInputFileListTask::default());
}


pub fn user_accessible_tasks() -> Vec<Box<dyn Task>> {
    return vec![];
}
