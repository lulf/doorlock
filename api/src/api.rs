use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use lazy_static::lazy_static;
use paho_mqtt as mqtt;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tera::{Context, Tera};

lazy_static! {
    pub static ref TERA: Tera = {
        let mut tera = match Tera::new("templates/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".html", ".sql"]);
        tera
    };
}

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
    let response = match (req.method(), req.uri().path()) {
        (&Method::GET, "/healthz") => {
            let body = Body::from("{\"status\": \"OK\"}");
            Response::new(body)
        }
        (&Method::GET, "/") => {
            let mut ctx = Context::new();
            let locked = state.load(Ordering::SeqCst);
            let action = if locked { "Unlock" } else { "Lock" };
            let url = if locked { "/doorlock/unlock" } else { "/doorlock/lock" };
            ctx.insert("action", action);
            ctx.insert("url", url);

            let body = Body::from(TERA.render("index.html", &ctx).unwrap().to_string());
            Response::new(body)
        }
        (&Method::POST, "/doorlock/lock") => {
            let topic = format!("command/{}/{}/lock", application, device);
            let command = serde_json::to_vec(&DeviceResponse { command: Command::Lock })?;
            let message = mqtt::Message::new(topic, &command[..], 1);
            let body = if let Err(e) = client.publish(message).await {
                log::warn!("Error publishing command back to device: {:?}", e);
                Body::from(format!("{{\"status\": \"ERROR\", \"description\": {:?}\"}}", e))
            } else {
                let ctx = Context::new();
                Body::from(TERA.render("action.html", &ctx).unwrap().to_string())
            };
            Response::new(body)
        }
        (&Method::POST, "/doorlock/unlock") => {
            let topic = format!("command/{}/{}/lock", application, device);
            let command = serde_json::to_vec(&DeviceResponse {
                command: Command::Unlock,
            })?;
            let message = mqtt::Message::new(topic, &command[..], 1);
            let body = if let Err(e) = client.publish(message).await {
                log::warn!("Error publishing command back to device: {:?}", e);
                Body::from(format!("{{\"status\": \"ERROR\", \"description\": {:?}\"}}", e))
            } else {
                let ctx = Context::new();
                Body::from(TERA.render("action.html", &ctx).unwrap().to_string())
            };
            Response::new(body)
        }
        _ => {
            let mut response = Response::new(Body::empty());
            *response.status_mut() = StatusCode::NOT_FOUND;
            response
        }
    };
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
