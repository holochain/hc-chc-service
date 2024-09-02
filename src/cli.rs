use std::{net::IpAddr, str::FromStr};

use crate::ChcService;

#[derive(clap::Parser, Debug)]
#[command(name = "hc-chc-service")]
#[command(about = "Run a local chc server")]
pub struct LocalChcServerCli {
    /// The network interface to use (e.g., 127.0.0.1).
    #[arg(short, long, default_value = "127.0.0.1")]
    pub interface: Option<String>,

    /// The port to bind to. Will default to an available port if not passed.
    #[arg(short, long)]
    pub port: Option<u16>,
}

impl TryInto<ChcService> for LocalChcServerCli {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<ChcService, Self::Error> {
        let address = IpAddr::from_str(&self.interface.unwrap_or_default())?;
        let port = self
            .port
            .unwrap_or_else(|| portpicker::pick_unused_port().expect("No available port found"));
        Ok(ChcService::new(address, port))
    }
}
