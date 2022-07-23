use cloudevents::{event::AttributeValue, Data, Event};
use futures::stream::StreamExt;
use paho_mqtt as mqtt;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub struct Server {
    client: mqtt::AsyncClient,
    group_id: Option<String>,
    application: String,
    device: String,
    state: Arc<AtomicBool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub struct Request {
    pub locked: bool,
}

impl Server {
    pub fn new(
        client: mqtt::AsyncClient,
        group_id: Option<String>,
        application: String,
        device: String,
        state: Arc<AtomicBool>,
    ) -> Self {
        Self {
            client,
            group_id,
            application,
            device,
            state,
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
                            let mut application = String::new();
                            let mut device = String::new();
                            for a in e.iter() {
                                log::trace!("Attribute {:?}", a);
                                if a.0 == "device" {
                                    if let AttributeValue::String(d) = a.1 {
                                        device = d.to_string();
                                    }
                                } else if a.0 == "application" {
                                    if let AttributeValue::String(d) = a.1 {
                                        application = d.to_string();
                                    }
                                }
                            }

                            let status: Option<Result<Request, anyhow::Error>> = if let Some(Data::Json(v)) = e.data() {
                                Some(serde_json::from_str(v.as_str().unwrap()).map_err(|e| e.into()))
                            } else {
                                None
                            };

                            log::trace!("Status decode: {:?}", status);

                            if device == self.device && application == self.application {
                                if let Some(Ok(status)) = status {
                                    log::info!("Lock status: {:?}", status);
                                    self.state.store(status.locked, Ordering::SeqCst);
                                }
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
