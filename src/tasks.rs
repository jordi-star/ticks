use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{ticktick_datetime_format, TickTick, TickTickError};

use super::{builders::TaskBuilder, projects::ProjectID};

/// ID used to identify Tasks from TickTick.
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(transparent)]
pub struct TaskID(pub String);

impl TaskID {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// ID used to identify Subtasks from TickTick.
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(transparent)]
pub struct SubtaskID(pub String);

impl SubtaskID {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// TickTick Subtask. In the API Reference, this is defined as a "ChecklistItem", but has been renamed to Subtask here for clarity.
/// [API Reference](https://developer.ticktick.com/docs/index.html#/openapi?id=checklistitem)
#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct Subtask {
    #[serde(skip_serializing_if = "SubtaskID::is_empty")]
    id: SubtaskID,
    title: String,
    status: SubtaskStatus,
    #[serde(with = "ticktick_datetime_format")]
    completed_time: DateTime<Utc>,
    is_all_day: bool,
    sort_order: i64,
    #[serde(with = "ticktick_datetime_format")]
    start_date: DateTime<Utc>,
    time_zone: String,
}

/// TickTick task
/// [API Reference](https://developer.ticktick.com/docs/index.html#/openapi?id=task-1)
#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct Task {
    #[serde(skip)]
    pub(crate) http_client: reqwest::Client,
    #[serde(skip_serializing_if = "TaskID::is_empty")]
    pub(crate) id: TaskID,
    #[serde(skip_serializing_if = "ProjectID::is_empty")]
    pub project_id: ProjectID,
    pub title: String,
    pub is_all_day: bool,
    #[serde(with = "ticktick_datetime_format")]
    pub completed_time: DateTime<Utc>,
    pub content: String,
    pub desc: String,
    #[serde(with = "ticktick_datetime_format")]
    pub due_date: DateTime<Utc>,
    /// Subtasks associated with this Task. This has been renamed from "items" for clarity.
    #[serde(rename = "items")]
    pub subtasks: Vec<Subtask>,
    pub priority: TaskPriority,
    pub reminders: Vec<String>,
    pub repeat_flag: String,
    pub sort_order: i64,
    #[serde(with = "ticktick_datetime_format")]
    pub start_date: DateTime<Utc>,
    pub status: TaskStatus,
    pub time_zone: String,
    pub tags: Vec<String>,
}

impl Task {
    pub fn builder(ticktick: &TickTick, title: &str) -> TaskBuilder {
        TaskBuilder::new(ticktick, title.into())
    }
    /// Get task using ProjectID & TaskID
    /// [API Reference](https://developer.ticktick.com/docs/index.html#/openapi?id=get-task-by-project-id-and-task-id)
    pub async fn get(
        ticktick: &TickTick,
        project_id: &ProjectID,
        task_id: &TaskID,
    ) -> Result<Self, TickTickError> {
        ticktick.get_task(project_id, task_id).await
    }

    /// Get all tasks associated with projects.
    pub async fn get_all_in_projects(ticktick: &TickTick) -> Result<Vec<Task>, TickTickError> {
        ticktick.get_all_tasks_in_projects().await
    }
    pub fn get_id(&self) -> &TaskID {
        &self.id
    }
    /// Delete task
    /// [API Reference](https://developer.ticktick.com/docs/index.html#/openapi?id=delete-task)
    pub async fn delete(self) -> Result<(), TickTickError> {
        self.http_client
            .delete(format!(
                "https://ticktick.com/open/v1/project/{}/task/{}",
                self.project_id.0, self.id.0
            ))
            .send()
            .await?
            .error_for_status()?;
        drop(self);
        Ok(())
    }
    /// Send changes made to this task to the TickTick API. Clients will require a refresh/sync for changes to take effect.
    /// [API Reference](https://developer.ticktick.com/docs/index.html#/openapi?id=update-task)
    pub async fn publish_changes(&self) -> Result<(), reqwest::Error> {
        self.http_client
            .post(format!("https://ticktick.com/open/v1/task/{}", self.id.0))
            .json(self)
            .send()
            .await?
            .text()
            .await?;
        Ok(())
    }

    /// Change task status to TaskStatus::Completed
    /// [API Reference](https://developer.ticktick.com/docs/index.html#/openapi?id=complete-task)
    pub async fn complete(&mut self) -> Result<(), reqwest::Error> {
        self.status = TaskStatus::Completed;
        self.http_client
            .post(format!(
                "/open/v1/project/{}/task/{}/complete",
                self.project_id.0, self.id.0
            ))
            .json(self)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

/// Enum matching Task Priority values listed in the Task API Reference
/// [API Reference](https://developer.ticktick.com/docs/index.html#/openapi?id=task-1)
#[derive(Serialize_repr, Deserialize_repr, Debug, Default)]
#[repr(u8)]
pub enum TaskPriority {
    #[default]
    None = 0,
    Low = 1,
    Medium = 3,
    High = 5,
}

/// Enum matching Task Status values listed in the Task API Reference
/// [API Reference](https://developer.ticktick.com/docs/index.html#/openapi?id=task-1)
#[derive(Serialize_repr, Deserialize_repr, Debug, Default)]
#[repr(u8)]
pub enum TaskStatus {
    #[default]
    Normal = 0,
    Completed = 2,
}

/// Enum matching Subtask Status values listed in the ChecklistItem API Reference
/// [API Reference](https://developer.ticktick.com/docs/index.html#/openapi?id=checklistitem)
#[derive(Serialize_repr, Deserialize_repr, Debug, Default)]
#[repr(u8)]
pub enum SubtaskStatus {
    #[default]
    Normal = 0,
    Completed = 1,
}
