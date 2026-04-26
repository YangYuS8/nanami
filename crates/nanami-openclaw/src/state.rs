use nanami_protocol::{Event, EventEnvelope, TaskCompletedPayload, TaskStartedPayload, TaskStatus};

#[derive(Default)]
pub(crate) struct ToolEventMappingState {
    pub(crate) counter: usize,
    pub(crate) task_started: bool,
    pub(crate) task_completed: bool,
    pub(crate) task_id: String,
}

impl ToolEventMappingState {
    pub(crate) fn next_event_id(&mut self) -> String {
        self.counter += 1;
        format!("evt_openclaw_tool_{:03}", self.counter)
    }

    pub(crate) fn ensure_task_id(&mut self) -> String {
        if self.task_id.is_empty() {
            self.task_id = "task_openclaw_stream_001".into();
        }
        self.task_id.clone()
    }
}

pub(crate) fn build_task_started_event(id: String, task_id: &str, title: &str) -> EventEnvelope {
    EventEnvelope::new(
        id,
        chrono::Utc::now(),
        Event::TaskStarted(TaskStartedPayload {
            session_id: None,
            task_id: task_id.to_owned(),
            title: title.to_owned(),
            status: TaskStatus::Running,
        }),
    )
}

pub(crate) fn build_task_completed_event(task_id: &str) -> EventEnvelope {
    EventEnvelope::new(
        "evt_openclaw_task_completed_001",
        chrono::Utc::now(),
        Event::TaskCompleted(TaskCompletedPayload {
            task_id: task_id.to_owned(),
            status: TaskStatus::Completed,
            summary: Some("OpenClaw stream completed".into()),
        }),
    )
}
