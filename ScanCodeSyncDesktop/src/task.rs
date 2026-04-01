use atomic_progress::Progress;

use crate::state::State;


pub trait Task {

    /// return value between 0% and 100%
    fn get_progress(&self) -> &Progress;


    /// returns true if the function is currently running
    fn is_running(&self) -> bool;

    /// attempts to run. When running the task should not be blocking
    fn attempt_run(&mut self, state: &mut State) -> anyhow::Result<()>;

    /// display name of task. This should be independent of the state of the task.
    fn get_name(&self) -> &str;

    /// if the task is silent it should not be displayed to the user when it is running
    fn silent(&self) -> bool;


    /// tells task to cancel and return borrowed values
    fn cancel_task(&mut self, state: &mut State);

    /// if the task is finished then the new values are put back into the state. If not the function will do nothing.
    /// if the task failed to complete it will return an error message
    fn attempt_collect(&mut self, state: &mut State) -> anyhow::Result<()>;

    /// if the task is finished then the new values are put back into the state. If not the function will do nothing.
    /// if the task failed to complete it will return an error message
    fn is_finished(&mut self) -> bool;

    /// returns a unique id which can be used to identify which values have been taken
    fn get_id(&self) -> u64;

}
