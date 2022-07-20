use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use paho_mqtt as mqtt;

pub struct ApiServer {
    port: u16,
}
impl ApiServer {
    pub fn new(client: mqtt::AsyncClient, application: String, port: u16) -> Self {
        Self { port }
    }

    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        let addr = ([0, 0, 0, 0], self.port).into();
        let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(service)) });

        let server = Server::bind(&addr).serve(service);

        log::info!("Listening on http://{}", addr);

        server.await?;
        Ok(())
    }
}

async fn service(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/healthz") => {
            *response.body_mut() = Body::from("{\"status\": \"OK\"}");
        }
        (&Method::PUT, "/doorlock/lock") => {
            *response.body_mut() = Body::from("{\"status\": \"OK\"}");
        }
        (&Method::PUT, "/doorlock/unlock") => {
            *response.body_mut() = Body::from("{\"status\": \"OK\"}");
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    }
    Ok(response)
}
