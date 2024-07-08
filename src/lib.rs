use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

use oauth2::TokenResponse;
use oauth2::{AuthUrl, AuthorizationCode, ClientId, CsrfToken, RedirectUrl, Scope, TokenUrl};
use objects::{
    projects::{Project, ProjectData, ProjectID},
    tasks::{Task, TaskID},
};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Url,
};
use serde::{Deserialize, Serialize};

pub mod objects;

#[derive(Debug)]
pub enum TickTickError {
    ClientError(reqwest::Error),
    ResponseParseError(serde_json::Error),
}

impl From<reqwest::Error> for TickTickError {
    fn from(value: reqwest::Error) -> Self {
        Self::ClientError(value)
    }
}

impl From<serde_json::Error> for TickTickError {
    fn from(value: serde_json::Error) -> Self {
        Self::ResponseParseError(value)
    }
}

#[derive(Debug)]
pub struct TickTick {
    http_client: reqwest::Client,
}

impl TickTick {
    pub fn new(access_token: AccessToken) -> Result<Self, TickTickError> {
        let mut headers_map = HeaderMap::new();
        let mut auth_header_value =
            HeaderValue::from_str(format!("Bearer {}", access_token.value).as_str())
                .expect("Invalid access token value.");
        auth_header_value.set_sensitive(true);
        headers_map.insert(reqwest::header::AUTHORIZATION, auth_header_value);
        let http_client_result = reqwest::Client::builder()
            .default_headers(headers_map)
            .build();
        Ok(Self {
            http_client: http_client_result?,
        })
    }
    pub async fn get_project_data(
        &self,
        project_id: &ProjectID,
    ) -> Result<ProjectData, TickTickError> {
        let resp = self
            .http_client
            .get(format!(
                "https://ticktick.com/open/v1/project/{}/data",
                project_id.0
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
    pub async fn get_task(
        &self,
        project_id: &ProjectID,
        task_id: &TaskID,
    ) -> Result<Task, TickTickError> {
        let resp = self
            .http_client
            .get(format!(
                "https://ticktick.com/open/v1/project/{}/task/{}",
                project_id.0, task_id.0
            ))
            .send()
            .await?
            .error_for_status()?;
        let mut task = resp.json::<Task>().await?;
        task.http_client = self.http_client.clone();
        Ok(task)
    }

    pub async fn get_all_tasks(&self) -> Result<Vec<Task>, TickTickError> {
        let projects = self.get_all_projects().await?;
        let mut value: Vec<Task> = Vec::new();
        for proj in projects {
            value.append(&mut proj.get_tasks().await?);
        }
        Ok(value)
    }

    pub async fn get_project(&self, project_id: &ProjectID) -> Result<Project, TickTickError> {
        let resp = self
            .http_client
            .get(format!(
                "https://ticktick.com/open/v1/project/{}",
                project_id.0
            ))
            .send()
            .await?
            .error_for_status()?;
        let mut proj = resp.json::<Project>().await?;
        proj.http_client = self.http_client.clone();
        Ok(proj)
    }

    pub async fn get_all_projects(&self) -> Result<Vec<Project>, TickTickError> {
        let mut projects = self
            .http_client
            .get("https://ticktick.com/open/v1/project/")
            .send()
            .await?
            .json::<Vec<Project>>()
            .await?;
        for proj in &mut projects {
            proj.http_client = self.http_client.clone();
        }
        Ok(projects)
    }
}

#[derive(Debug)]
pub enum AuthorizationError {
    ReqwestClientError(reqwest::Error),
    InvalidCSRFState {
        expected: CsrfToken,
        recieved: CsrfToken,
    },
}

impl From<reqwest::Error> for AuthorizationError {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestClientError(value)
    }
}

pub struct Authorization {}

impl Authorization {
    pub fn begin_auth(
        client_id: String,
        redirect_uri: String,
    ) -> Result<AwaitingAuthCode, AuthorizationError> {
        let auth_client = oauth2::basic::BasicClient::new(
            ClientId::new(client_id),
            None,
            AuthUrl::new("https://ticktick.com/oauth/authorize".to_string()).unwrap(),
            Some(TokenUrl::new("https://ticktick.com/oauth/token".to_string()).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_uri).unwrap());
        let (authorization_url, csrf_state) = auth_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("tasks:read".to_string()))
            .add_scope(Scope::new("tasks:write".to_string()))
            .url();
        Ok(AwaitingAuthCode {
            authorization_url,
            csrf_state,
            auth_client,
        })
    }
}

pub struct AwaitingAuthCode {
    pub authorization_url: Url,
    csrf_state: CsrfToken,
    auth_client: oauth2::basic::BasicClient,
}

impl AwaitingAuthCode {
    pub fn get_url(&self) -> &Url {
        &self.authorization_url
    }
    pub async fn finish_auth(
        self,
        client_secret: String,
        auth_code: String,
        state: String,
    ) -> Result<AccessToken, AuthorizationError> {
        let http_client = reqwest::Client::new();
        let mut token_request_form = HashMap::new();
        token_request_form.insert("client_id", self.auth_client.client_id().as_str());
        token_request_form.insert("client_secret", &client_secret);
        token_request_form.insert("code", &auth_code);
        token_request_form.insert("grant_type", "authorization_code");
        token_request_form.insert("scope", "tasks:write tasks:read");
        token_request_form.insert("redirect_uri", self.auth_client.redirect_url().unwrap());
        if &state != self.csrf_state.secret() {
            return Err(AuthorizationError::InvalidCSRFState {
                expected: self.csrf_state,
                recieved: CsrfToken::new(state),
            });
        };
        let token_request_result = http_client
            .post("https://ticktick.com/oauth/token")
            .form(&token_request_form)
            .send()
            .await;
        Ok(token_request_result?.json::<AccessToken>().await?)
    }
}

// pub struct AuthUrl(pub Url);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccessToken {
    #[serde(rename = "access_token")]
    pub value: String,
    pub token_type: String,
    pub expires_in: u32,
    pub scope: String,
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
