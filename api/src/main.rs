use anyhow::Context;
use clap::Parser;
use std::sync::{atomic::AtomicBool, Arc};

use paho_mqtt as mqtt;

use std::time::Duration;

mod api;
mod server;

#[derive(Parser, Debug)]
struct Args {
    /// Mqtt server uri (tcp://host:port)
    #[clap(long)]
    mqtt_uri: String,

    /// Mqtt group id for shared subscription (for horizontal scaling)
    #[clap(long)]
    mqtt_group_id: Option<String>,

    /// Name of specific application to manage
    #[clap(long)]
    application: String,

    /// Name of specific device to manage
    #[clap(long)]
    device: String,

    /// Token for authenticating ajour to Drogue IoT
    #[clap(long)]
    token: String,

    /// User for authenticating ajour to Drogue IoT
    #[clap(long)]
    user: String,

    /// Path to CA
    #[clap(long)]
    ca_path: Option<String>,

    /// Disable TLS
    #[clap(long)]
    disable_tls: bool,

    /// Ignore cert validation
    #[clap(long)]
    insecure_tls: bool,

    /// Port for API endpoint
    #[clap(long, default_value_t = 8080)]
    api_port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    env_logger::init();

    let mqtt_uri = args.mqtt_uri;
    let token = args.token;

    let mqtt_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(mqtt_uri)
        .client_id("doorlock")
        .persistence(mqtt::PersistenceType::None)
        .finalize();
    let mut mqtt_client = mqtt::AsyncClient::new(mqtt_opts)?;

    let mut conn_opts = mqtt::ConnectOptionsBuilder::new();
    conn_opts.user_name(args.user);
    conn_opts.password(token.clone());
    conn_opts.keep_alive_interval(Duration::from_secs(30));
    conn_opts.automatic_reconnect(Duration::from_millis(100), Duration::from_secs(5));

    if !args.disable_tls {
        let ca = args.ca_path.unwrap_or("/etc/ssl/certs/ca-bundle.crt".to_string());
        let ssl_opts = if args.insecure_tls {
            mqtt::SslOptionsBuilder::new()
                .trust_store(&ca)?
                .enable_server_cert_auth(false)
                .verify(false)
                .finalize()
        } else {
            mqtt::SslOptionsBuilder::new().trust_store(&ca)?.finalize()
        };
        conn_opts.ssl_options(ssl_opts);
    }

    let conn_opts = conn_opts.finalize();

    mqtt_client.set_disconnected_callback(|c, _, _| {
        log::info!("Disconnected");
        let t = c.reconnect();
        if let Err(e) = t.wait_for(Duration::from_secs(10)) {
            log::warn!("Error reconnecting to broker ({:?}), exiting", e);
            std::process::exit(1);
        }
    });

    mqtt_client.set_connection_lost_callback(|c| {
        log::info!("Connection lost");
        let t = c.reconnect();
        if let Err(e) = t.wait_for(Duration::from_secs(10)) {
            log::warn!("Error reconnecting to broker ({:?}), exiting", e);
            std::process::exit(1);
        }
    });

    mqtt_client
        .connect(conn_opts)
        .await
        .context("Failed to connect to MQTT endpoint")?;

    let state = Arc::new(AtomicBool::new(false));
    let mut api = api::ApiServer::new(
        mqtt_client.clone(),
        args.application.clone(),
        args.device.clone(),
        args.api_port,
        state.clone(),
    );
    let mut app = server::Server::new(mqtt_client, args.mqtt_group_id, args.application, args.device, state);

    futures::try_join!(app.run(), api.run())?;
    Ok(())
}
