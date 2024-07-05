use serde::Serialize;

use crate::{TickTick, TickTickError};

use super::{
    projects::{Project, ProjectID, ProjectKind, ProjectViewMode},
    tasks::{ChecklistItem, Task, TaskPriority, TaskStatus},
};

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskBuilder {
    #[serde(skip)]
    http_client: reqwest::Client,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_id: Option<ProjectID>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_all_day: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    completed_time: Option<String>, // TODO: Could make this a chrono datetime and see if serde can serialize it? It's a date-time string.
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    desc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_date: Option<String>, // TODO: Same as above, date-time string
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    items: Vec<ChecklistItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<TaskPriority>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    reminders: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    repeat_flag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort_order: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_date: Option<String>, // TODO: Above, date-time
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<TaskStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time_zone: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
}

impl TaskBuilder {
    pub fn new(ticktick: &TickTick, title: String) -> Self {
        Self {
            title,
            http_client: ticktick.http_client.clone(),
            ..Default::default()
        }
    }
    pub fn title(mut self, value: &str) -> Self {
        self.title = value.into();
        self
    }
    pub fn project_id(mut self, value: ProjectID) -> Self {
        self.project_id = Some(value);
        self
    }
    pub fn is_all_day(mut self, value: bool) -> Self {
        self.is_all_day = Some(value);
        self
    }
    pub fn completed_time(mut self, value: &str) -> Self {
        self.completed_time = Some(value.into());
        self
    }
    pub fn content(mut self, value: &str) -> Self {
        self.content = Some(value.into());
        self
    }
    pub fn desc(mut self, value: &str) -> Self {
        self.desc = Some(value.into());
        self
    }
    pub fn due_date(mut self, value: &str) -> Self {
        self.due_date = Some(value.into());
        self
    }
    pub fn items(mut self, value: Vec<ChecklistItem>) -> Self {
        self.items = value;
        self
    }
    pub fn priority(mut self, value: TaskPriority) -> Self {
        self.priority = Some(value);
        self
    }
    pub fn reminders(mut self, value: Vec<String>) -> Self {
        self.reminders = value;
        self
    }
    pub fn repeat_flag(mut self, value: &str) -> Self {
        self.repeat_flag = Some(value.into());
        self
    }
    pub fn sort_order(mut self, value: i64) -> Self {
        self.sort_order = Some(value);
        self
    }
    pub fn start_date(mut self, value: &str) -> Self {
        self.start_date = Some(value.into());
        self
    }
    pub fn status(mut self, value: TaskStatus) -> Self {
        self.status = Some(value);
        self
    }
    pub fn time_zone(mut self, value: &str) -> Self {
        self.time_zone = Some(value.into());
        self
    }
    pub fn tags(mut self, value: Vec<String>) -> Self {
        self.tags = value;
        self
    }

    pub async fn build_and_publish(self) -> Result<Task, TickTickError> {
        let mut task = self
            .http_client
            .post("https://ticktick.com/open/v1/task")
            .body(serde_json::to_string(&self).unwrap())
            .send()
            .await?
            .json::<Task>()
            .await?;
        task.http_client = self.http_client.clone();
        Ok(task)
    }
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectBuilder {
    #[serde(skip)]
    http_client: reqwest::Client,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort_order: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    view_mode: Option<ProjectViewMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<ProjectKind>,
}

impl ProjectBuilder {
    pub fn new(ticktick: &TickTick, name: String) -> Self {
        Self {
            name,
            http_client: ticktick.http_client.clone(),
            ..Default::default()
        }
    }
    pub fn name(mut self, value: &str) -> Self {
        self.name = value.into();
        self
    }
    pub fn color(mut self, value: &str) -> Self {
        self.color = Some(value.into());
        self
    }
    pub fn view_mode(mut self, value: ProjectViewMode) -> Self {
        self.view_mode = Some(value);
        self
    }
    pub fn kind(mut self, value: ProjectKind) -> Self {
        self.kind = Some(value);
        self
    }

    pub async fn build_and_publish(self) -> Result<Project, TickTickError> {
        let mut project = self
            .http_client
            .post("https://ticktick.com/open/v1/project")
            .json(&self)
            .send()
            .await?
            .json::<Project>()
            .await?;
        project.http_client = self.http_client.clone();
        Ok(project)
    }
}
