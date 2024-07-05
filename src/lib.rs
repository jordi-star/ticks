use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

use oauth2::TokenResponse;
use oauth2::{AuthUrl, AuthorizationCode, ClientId, CsrfToken, RedirectUrl, Scope, TokenUrl};
use objects::{
    projects::{Project, ProjectID},
    tasks::{Task, TaskID},
};
use reqwest::header::{HeaderMap, HeaderValue};
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
}

impl From<reqwest::Error> for AuthorizationError {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestClientError(value)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccessToken {
    #[serde(rename = "access_token")]
    pub value: String,
    pub token_type: String,
    pub expires_in: u32,
    pub scope: String,
}

impl AccessToken {
    pub async fn new_authorization(
        client_id: String,
        client_secret: String,
    ) -> Result<Self, AuthorizationError> {
        let auth_client = oauth2::basic::BasicClient::new(
            ClientId::new(client_id),
            None,
            AuthUrl::new("https://ticktick.com/oauth/authorize".to_string()).unwrap(),
            Some(TokenUrl::new("https://ticktick.com/oauth/token".to_string()).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new("http://localhost:8080".to_string()).unwrap());
        let (auth_url, csrf_state) = auth_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("tasks:read".to_string()))
            .add_scope(Scope::new("tasks:write".to_string()))
            .url();
        println!("Browse to: {}", auth_url);
        let (code, state) = {
            let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
            let Ok((mut stream, _)) = listener.accept() else {
                panic!("ERRRR");
            };
            let mut stream_reader = BufReader::new(&stream);
            let mut response = String::new();
            stream_reader.read_line(&mut response).unwrap();
            println!("resp {:?}", response);

            let code: AuthorizationCode =
                AuthorizationCode::new(get_value_from_http_response(&response, "code").unwrap());
            let state: CsrfToken =
                CsrfToken::new(get_value_from_http_response(&response, "state").unwrap());
            stream.write_all("HTTP/1.1 200 OK".as_bytes()).unwrap();
            (code, state)
        };
        println!("c   {}   , s   {}   ", code.secret(), state.secret());
        let http_client = reqwest::Client::new();
        let mut token_request_form = HashMap::new();
        token_request_form.insert("client_id", auth_client.client_id().as_str());
        // TODO!HIGH Make private.
        token_request_form.insert("client_secret", &client_secret);
        token_request_form.insert("code", code.secret());
        token_request_form.insert("grant_type", "authorization_code");
        token_request_form.insert("scope", "tasks:write tasks:read");
        token_request_form.insert("redirect_uri", "http://localhost:8080");

        let token_request_result = http_client
            .post("https://ticktick.com/oauth/token")
            .form(&token_request_form)
            .send()
            .await;
        Ok(token_request_result?.json::<Self>().await?)
    }
}

pub fn get_value_from_http_response(response: &String, key: &str) -> Option<String> {
    let mut response_split = response.split(&[' ', '=', '&']);
    loop {
        if let Some(chunk) = response_split.next() {
            if chunk.contains(key) {
                break response_split.next().map(str::to_string);
            }
        } else {
            panic!("RETURN ERROR NO CODE");
        }
    }
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
