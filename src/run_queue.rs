//! Per CPU run queue for EquationOS' task scheduler.
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::task::EqTask;

const RUN_QUEUE_SIZE: usize = 64;

pub struct EqTaskQueue {
    queue: [Option<EqTask>; RUN_QUEUE_SIZE],
    head: AtomicUsize,
    tail: AtomicUsize,
    size: AtomicUsize,
}

impl Default for EqTaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl EqTaskQueue {
    pub fn new() -> Self {
        Self {
            queue: [(); RUN_QUEUE_SIZE].map(|_| None),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            size: AtomicUsize::new(0),
        }
    }

    /// Insert a task into the run queue. Returns Err if the queue is full.
    pub fn insert(&mut self, task: EqTask) -> Result<(), EqTask> {
        loop {
            let size = self.size.load(Ordering::Acquire);
            if size == RUN_QUEUE_SIZE {
                return Err(task);
            }
            // Try to increment size atomically
            if self
                .size
                .compare_exchange(size, size + 1, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                let tail = self.tail.fetch_add(1, Ordering::AcqRel) % RUN_QUEUE_SIZE;
                // Safety: Only one thread can insert at this slot due to size CAS above
                self.queue[tail] = Some(task);
                return Ok(());
            }
        }
    }

    /// Pop a task from the run queue. Returns None if the queue is empty.
    pub fn pop(&mut self) -> Option<EqTask> {
        loop {
            let size = self.size.load(Ordering::Acquire);
            if size == 0 {
                return None;
            }
            // Try to decrement size atomically
            if self
                .size
                .compare_exchange(size, size - 1, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                let head = self.head.fetch_add(1, Ordering::AcqRel) % RUN_QUEUE_SIZE;
                // Safety: Only one thread can pop at this slot due to size CAS above
                return self.queue[head].take();
            }
        }
    }

    /// Get the number of tasks in the run queue.
    pub fn get_task_num(&self) -> usize {
        self.size.load(Ordering::Acquire)
    }
}

impl core::fmt::Debug for EqTaskQueue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "EqTaskQueue {{ head: {}, tail: {}, size: {} }}",
            self.head.load(Ordering::Acquire),
            self.tail.load(Ordering::Acquire),
            self.size.load(Ordering::Acquire)
        )?;
        let mut i = self.head.load(Ordering::Acquire);
        let size = self.size.load(Ordering::Acquire);
        for j in 0..size {
            let task = self.queue[i % RUN_QUEUE_SIZE].as_ref();
            if let Some(task) = task {
                writeln!(f, "[{}] {:?}", j, task)?;
            } else {
                writeln!(f, "[{}] None", j)?;
            }
            i += 1;
        }
        Ok(())
    }
}
