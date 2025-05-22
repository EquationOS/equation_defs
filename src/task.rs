//！ Task Definition under Equation semantics.

pub struct EqTask {
    /// The Instance ID of the task.
    pub instance_id: usize,
    /// The Process ID of the task.
    pub process_id: usize,
    /// The ID of the task.
    pub task_id: usize,
}

impl core::fmt::Debug for EqTask {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "EqTask:I[{}]P({})T<{}>",
            self.instance_id, self.process_id, self.task_id
        )
    }
}
