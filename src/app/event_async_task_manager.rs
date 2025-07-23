use crate::app::event_msg::Msg;
use std::collections::HashMap;
use std::future::Future;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub type TaskId = u64;

pub struct AsyncTaskManager {
    handles: HashMap<TaskId, JoinHandle<()>>,
    receiver: mpsc::UnboundedReceiver<Msg>,
    sender: mpsc::UnboundedSender<Msg>,
    next_id: TaskId,
}

impl AsyncTaskManager {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        Self {
            handles: HashMap::new(),
            receiver,
            sender,
            next_id: 1,
        }
    }

    pub fn spawn_task<F>(&mut self, future: F) -> TaskId
    where
        F: Future<Output = Msg> + Send + 'static,
    {
        let task_id = self.next_id;
        self.next_id += 1;

        let sender = self.sender.clone();
        let handle = tokio::spawn(async move {
            let result = future.await;
            let _ = sender.send(result);
        });

        self.handles.insert(task_id, handle);
        task_id
    }

    pub fn cancel_task(&mut self, task_id: TaskId) -> bool {
        if let Some(handle) = self.handles.remove(&task_id) {
            handle.abort();
            true
        } else {
            false
        }
    }

    pub fn poll_messages(&mut self) -> Vec<Msg> {
        let mut messages = Vec::new();
        while let Ok(msg) = self.receiver.try_recv() {
            messages.push(msg);
        }
        messages
    }

    pub fn cleanup_completed_tasks(&mut self) {
        self.handles.retain(|_id, handle| !handle.is_finished());
    }

    pub fn active_task_count(&self) -> usize {
        self.handles.len()
    }
}

impl Drop for AsyncTaskManager {
    fn drop(&mut self) {
        for (_, handle) in self.handles.drain() {
            handle.abort();
        }
    }
}
