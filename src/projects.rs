use serde::{Deserialize, Serialize};

use crate::{TickTick, TickTickError};

use super::{builders::ProjectBuilder, tasks::Task};

/// ID used to identify Projects from TickTick.
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(transparent)]
pub struct ProjectID(pub String);

impl ProjectID {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// ID used to identify Project Groups from TickTick.
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(transparent)]
pub struct GroupID(pub String);

impl GroupID {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// TickTick Project info
/// [API Reference](https://developer.ticktick.com/docs/index.html#/openapi?id=project-1)
#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Project {
    #[serde(skip)]
    pub(crate) http_client: reqwest::Client,
    pub(crate) id: ProjectID,
    pub name: String,
    pub color: String,
    pub sort_order: i64,
    #[serde(default)]
    pub closed: bool,
    pub group_id: GroupID,
    pub view_mode: ProjectViewMode,
    pub permission: ProjectUserPermissions,
    pub kind: ProjectKind,
}

impl Project {
    pub fn builder(ticktick: &TickTick, name: String) -> ProjectBuilder {
        ProjectBuilder::new(ticktick, name)
    }
    pub fn get_id(self) -> ProjectID {
        self.id
    }
    pub async fn get_data(&self) -> Result<ProjectData, TickTickError> {
        let resp = self
            .http_client
            .get(format!(
                "https://ticktick.com/open/v1/project/{}/data",
                self.id.0
            ))
            .send()
            .await?
            .error_for_status()?;
        let mut project_data = resp.json::<ProjectData>().await?;
        project_data
            .tasks
            .iter_mut()
            .for_each(|task| task.http_client = self.http_client.clone());
        Ok(project_data)
    }
    pub async fn get_all(ticktick: &TickTick) -> Result<Vec<Project>, TickTickError> {
        ticktick.get_all_projects().await
    }
    pub async fn get_tasks(&self) -> Result<Vec<Task>, TickTickError> {
        Ok(self.get_data().await?.tasks)
    }
    pub async fn get_columns(&self) -> Result<Vec<Column>, TickTickError> {
        Ok(self.get_data().await?.columns)
    }
    pub async fn get(ticktick: &TickTick, id: &ProjectID) -> Result<Project, TickTickError> {
        ticktick.get_project(id).await
    }
    /// Send changes made to this project to the TickTick API. Clients will require a refresh/sync for changes to take effect.
    /// [API Reference](https://developer.ticktick.com/docs/index.html#/openapi?id=update-project)
    pub async fn publish_changes(&self) -> Result<(), reqwest::Error> {
        self.http_client
            .post(format!(
                "https://ticktick.com/open/v1/project/{}",
                self.id.0
            ))
            .json(self)
            .send()
            .await?
            .text()
            .await?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(from = "String", rename_all = "lowercase")]
pub enum ProjectViewMode {
    #[default]
    List,
    Kanban,
    Timeline,
}

impl From<String> for ProjectViewMode {
    fn from(value: String) -> Self {
        match value.as_str() {
            "list" => Self::List,
            "kanban" => Self::Kanban,
            "timeline" => Self::Timeline,
            _ => Self::List,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(from = "String", rename_all = "lowercase")]
pub enum ProjectUserPermissions {
    #[default]
    Read,
    Write,
    Comment,
}

impl From<String> for ProjectUserPermissions {
    fn from(value: String) -> Self {
        match value.as_str() {
            "read" => Self::Read,
            "write" => Self::Write,
            "comment" => Self::Comment,
            _ => Self::Read,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(from = "String", rename_all = "UPPERCASE")]
pub enum ProjectKind {
    #[default]
    Task,
    Note,
}

impl From<String> for ProjectKind {
    fn from(value: String) -> Self {
        match value.as_str() {
            "TASK" => Self::Task,
            "NOTE" => Self::Note,
            _ => Self::Task,
        }
    }
}

/// TickTick ProjectData
/// [API Reference](https://developer.ticktick.com/docs/index.html#/openapi?id=projectdata)
#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectData {
    pub tasks: Vec<Task>,
    pub columns: Vec<Column>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(transparent)]
pub struct ColumnID(pub String);

impl ColumnID {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    id: ColumnID,
    project_id: ProjectID,
    name: String,
    sort_order: i64,
}
