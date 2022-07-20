use cloudevents::{event::AttributeValue, Data, Event};
use futures::stream::StreamExt;
use paho_mqtt as mqtt;
use serde::{Deserialize, Serialize};

pub struct Server {
    client: mqtt::AsyncClient,
    group_id: Option<String>,
    application: String,
    lock_state: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub struct Request {
    pub locked: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub struct Response {
    pub command: Command,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub enum Command {
    Lock,
    Unlock,
}

impl Server {
    pub fn new(client: mqtt::AsyncClient, group_id: Option<String>, application: String) -> Self {
        Self {
            client,
            group_id,
            application,
            lock_state: None,
        }
    }

    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        let mut stream = self.client.get_stream(100);
        if let Some(group_id) = &self.group_id {
            self.client
                .subscribe(format!("$shared/{}/app/{}", &group_id, &self.application), 1);
        } else {
            self.client.subscribe(format!("app/{}", &self.application), 1);
        }
        loop {
            if let Some(m) = stream.next().await {
                if let Some(m) = m {
                    match serde_json::from_slice::<Event>(m.payload()) {
                        Ok(e) => {
                            let status: Option<Result<Request, anyhow::Error>> = if let Some(Data::Json(v)) = e.data() {
                                Some(serde_json::from_str(v.as_str().unwrap()).map_err(|e| e.into()))
                            } else {
                                None
                            };

                            log::trace!("Status decode: {:?}", status);

                            if let Some(Ok(status)) = status {
                                log::info!("Lock status: {:?}", status);
                            }
                        }
                        Err(e) => {
                            log::warn!("Error parsing event: {:?}", e);
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/*
                                    log::debug!("Received status from {}: {:?}", device, status);
                                    if let Ok(command) = self.updater.process(&application, &device, &status).await {
                                        //log::trace!("Sending command to {}: {:?}", device, command);

                                        let topic = format!("command/{}/{}/{}", application, device, subject);
                                        let message = mqtt::Message::new(topic, command.as_bytes(), 1);
                                        if let Err(e) = self.client.publish(message).await {
                                            log::warn!("Error publishing command back to device: {:?}", e);
                                        }
                                    }
*/
