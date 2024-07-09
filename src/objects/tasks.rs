
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{TickTick, TickTickError};

use super::{builders::TaskBuilder, projects::ProjectID};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(transparent)]
pub struct TaskID(pub String);

impl TaskID {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChecklistItem {} // TODO IMPLEMENT

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
    pub completed_time: String, // TODO: Could make this a chrono datetime and see if serde can serialize it? It's a date-time string.
    pub content: String,
    pub desc: String,
    pub due_date: String, // TODO: Same as above, date-time string
    pub items: Vec<ChecklistItem>,
    pub priority: TaskPriority,
    pub reminders: Vec<String>,
    pub repeat_flag: String,
    pub sort_order: i64,
    pub start_date: String, // TODO: Above, date-time
    // #[serde(skip_serializing)]
    pub status: TaskStatus,
    pub time_zone: String,
    pub tags: Vec<String>,
}

impl Task {
    pub fn builder(ticktick: &TickTick, title: &str) -> TaskBuilder {
        TaskBuilder::new(ticktick, title.into())
    }
    pub async fn get(
        ticktick: &TickTick,
        project_id: &ProjectID,
        task_id: &TaskID,
    ) -> Result<Self, TickTickError> {
        ticktick.get_task(project_id, task_id).await
    }
    pub async fn get_all(ticktick: &TickTick) -> Result<Vec<Task>, TickTickError> {
        ticktick.get_all_tasks().await
    }
    pub fn get_id(&self) -> &TaskID {
        &self.id
    }
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
    pub async fn publish_changes(&self) -> Result<(), reqwest::Error> {
        println!(
            "{} {:#?}, {:#?}",
            serde_json::to_string_pretty(&self).unwrap(),
            self.http_client,
            self.http_client
                .post(format!("https://ticktick.com/open/v1/task/{}", self.id.0))
                .json(self)
                .send()
                .await?
                .text()
                .await
        );
        Ok(())
    }

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

#[derive(Serialize_repr, Deserialize_repr, Debug, Default)]
#[repr(u8)]
pub enum TaskPriority {
    #[default]
    None = 0,
    Low = 1,
    Medium = 3,
    High = 5,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Default)]
#[repr(u8)]
pub enum TaskStatus {
    #[default]
    Normal = 0,
    Completed = 2,
}
