use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct Gateway {
    client: reqwest::Client,
    http: String,
    user: String,
    password: String,
}

impl Gateway {
    pub fn new(http: String, user: String, password: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            http: format!("{}/v1/lock", http),
            user,
            password,
        }
    }

    // Best effort publish event
    pub async fn publish(
        &self,
        device: &str,
        data: &Request,
        timeout: Duration,
    ) -> Result<Option<Response>, anyhow::Error> {
        let resp = self
            .client
            .post(&self.http)
            .query(&[("as", device), ("ct", &format!("{}", timeout.as_secs()))])
            .basic_auth(&self.user, Some(&self.password))
            .json(data)
            .send()
            .await?;
        if !resp.status().is_success() {
            Err(anyhow!(
                "Error response {}: {}",
                resp.status(),
                resp.text().await.unwrap_or_default()
            ))
        } else {
            if let Ok(result) = resp.json().await {
                Ok(Some(result))
            } else {
                Ok(None)
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Request {
    pub locked: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub command: Command,
}

#[derive(Serialize, Deserialize)]
pub enum Command {
    Lock,
    Unlock,
}
