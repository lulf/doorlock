use clap::{Parser, Subcommand};
mod device;
use device::*;

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
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Command {
    SetStep {
        #[clap(long)]
        step: u8,
    },
    SetSpeed {
        #[clap(long)]
        speed: u32,
    },
    Lock,
    Unlock,
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

    let mut s = LockDevice::new(&args.device, central);

    match args.command {
        Command::SetStep { step } => s.set_step(step).await?,
        Command::SetSpeed { speed } => s.set_speed(speed).await?,
        Command::Lock => s.lock().await?,
        Command::Unlock => s.unlock().await?,
    }
    Ok(())
}
