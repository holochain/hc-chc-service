use std::{net::Ipv4Addr, str::FromStr};

use crate::ChcService;

#[derive(clap::Parser, Debug)]
#[command(name = "hc-local-chc-server")]
#[command(about = "Run a local chc server")]
pub struct LocalChcServerCli {
    /// The network interface to use (e.g., 127.0.0.1). Will defaul to 127.0.0.1 if not
    /// passed.
    #[arg(short, long)]
    pub interface: Option<String>,

    /// The port to bind to. Will default to an available port if not passed.
    #[arg(short, long)]
    pub port: Option<u16>,
}

impl LocalChcServerCli {
    pub fn create_chc_service(&self) -> anyhow::Result<ChcService> {
        let address = Ipv4Addr::from_str(
            &self
                .interface
                .clone()
                .unwrap_or_else(|| "127.0.0.1".to_string()),
        )?;
        let port = self
            .port
            .unwrap_or_else(|| portpicker::pick_unused_port().expect("No available port found"));
        Ok(ChcService::new(address, port))
    }
}
