use clap::Parser;
mod device;
use device::*;
mod gateway;
use gateway::*;

use std::time::Duration;

#[derive(Parser, Debug)]
struct Args {
    /// Adjust the output verbosity.
    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,

    /// Enable device discovery
    #[clap(long)]
    enable_discovery: bool,

    /// The MAC address of the device to update.
    #[clap(long)]
    device: String,

    /// The command to issue
    #[clap(long)]
    drogue_http: String,

    #[clap(long)]
    user: String,

    #[clap(long)]
    password: String,

    #[clap(long, parse(try_from_str=humantime::parse_duration))]
    interval: Option<Duration>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    stderrlog::new().verbosity(args.verbose).init().unwrap();

    use btleplug::api::{Central, Manager as _, ScanFilter};
    use btleplug::platform::Manager;
    let manager = Manager::new().await?;
    let central = manager
        .adapters()
        .await?
        .into_iter()
        .nth(0)
        .ok_or(anyhow::anyhow!("no adapter found"))?;

    if args.enable_discovery {
        central.start_scan(ScanFilter::default()).await?;
    }

    let interval = args.interval.unwrap_or(Duration::from_secs(30));

    let gateway = Gateway::new(args.drogue_http.clone(), args.user.clone(), args.password.clone());
    let mut lock = LockDevice::new(&args.device, central);

    loop {
        if let Ok(is_locked) = lock.is_locked().await {
            log::info!("Reporting lock state. Is locked: {}", is_locked);
            match gateway
                .publish(&args.device, &Request { locked: is_locked }, interval)
                .await
            {
                Ok(Some(response)) => match response.command {
                    Command::Lock => {
                        let _ = lock.lock().await;
                    }
                    Command::Unlock => {
                        let _ = lock.unlock().await;
                    }
                },
                Ok(None) => {
                    log::info!("No command received");
                }
                Err(e) => {
                    log::warn!("Error reporting lock state: {:?}", e);
                }
            }
        } else {
            tokio::time::sleep(interval).await;
        }
    }
}
