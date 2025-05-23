//！ Task Definition under Equation semantics.

use crate::context::TaskContext;

pub struct EqTask {
    /// The Instance ID of the task.
    pub instance_id: usize,
    /// The Process ID of the task.
    pub process_id: usize,
    /// The ID of the task.
    pub task_id: usize,
    /// The context of the task.
    pub context: TaskContext,
}

impl core::fmt::Debug for EqTask {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "EqTask:I[{}]P({})T<{}>, ksp {:?}, rsp {:?}",
            self.instance_id,
            self.process_id,
            self.task_id,
            self.context.kstack_top,
            self.context.rsp
        )
    }
}
