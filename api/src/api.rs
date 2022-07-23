use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use paho_mqtt as mqtt;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub struct ApiServer {
    port: u16,
    client: Arc<mqtt::AsyncClient>,
    application: String,
    device: String,
    state: Arc<AtomicBool>,
}
impl ApiServer {
    pub fn new(
        client: mqtt::AsyncClient,
        application: String,
        device: String,
        port: u16,
        state: Arc<AtomicBool>,
    ) -> Self {
        Self {
            port,
            client: Arc::new(client),
            application,
            device,
            state,
        }
    }

    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        let addr = ([0, 0, 0, 0], self.port).into();
        let service = make_service_fn(|_| {
            let client = self.client.clone();
            let app = self.application.clone();
            let dev = self.device.clone();
            let state = self.state.clone();
            async {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    let client = client.clone();
                    let app = app.clone();
                    let dev = dev.clone();
                    let state = state.clone();
                    async { service(req, client, app, dev, state).await }
                }))
            }
        });

        let server = Server::bind(&addr).serve(service);

        log::info!("Listening on http://{}", addr);

        server.await?;
        Ok(())
    }
}

async fn service(
    req: Request<Body>,
    client: Arc<mqtt::AsyncClient>,
    application: String,
    device: String,
    state: Arc<AtomicBool>,
) -> Result<Response<Body>, anyhow::Error> {
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/healthz") => {
            *response.body_mut() = Body::from("{\"status\": \"OK\"}");
        }
        (&Method::GET, "/doorlock/state") => {
            *response.body_mut() = Body::from(format!("{{\"locked\": {}}}", state.load(Ordering::SeqCst)));
        }
        (&Method::PUT, "/doorlock/lock") => {
            let topic = format!("command/{}/{}/lock", application, device);
            let command = serde_json::to_vec(&DeviceResponse { command: Command::Lock })?;
            let message = mqtt::Message::new(topic, &command[..], 1);
            if let Err(e) = client.publish(message).await {
                log::warn!("Error publishing command back to device: {:?}", e);
                *response.body_mut() = Body::from(format!("{{\"status\": \"ERROR\", \"description\": {:?}\"}}", e));
            } else {
                *response.body_mut() = Body::from("{\"status\": \"OK\"}");
            }
        }
        (&Method::PUT, "/doorlock/unlock") => {
            let topic = format!("command/{}/{}/lock", application, device);
            let command = serde_json::to_vec(&DeviceResponse {
                command: Command::Unlock,
            })?;
            let message = mqtt::Message::new(topic, &command[..], 1);
            if let Err(e) = client.publish(message).await {
                log::warn!("Error publishing command back to device: {:?}", e);
                *response.body_mut() = Body::from(format!("{{\"status\": \"ERROR\", \"description\": {:?}\"}}", e));
            } else {
                *response.body_mut() = Body::from("{\"status\": \"OK\"}");
            }
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    }
    Ok(response)
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub struct DeviceResponse {
    pub command: Command,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub enum Command {
    Lock,
    Unlock,
}
