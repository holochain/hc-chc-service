use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use anyhow::anyhow;
use axum::{
    routing::{get, post},
    Router,
};
use holochain::{
    core::{Signature, SignedActionHashed},
    prelude::EncryptedEntry,
};
use parking_lot::Mutex;
use tokio::net::TcpListener;

use crate::routes::{add_records, get_record_data};

#[derive(Debug)]
pub struct ChcService {
    address: SocketAddr,
    router: Router,
}

// Sourced from holochain sourcecode, original struct contains inacessible private fields
// https://github.com/holochain/holochain/blob/60102a603b8039eb46e786d82dd6382f3a1b1c93/crates/holochain/src/conductor/chc/chc_local.rs#L33
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct RecordItem {
    pub(crate) action: SignedActionHashed,
    pub encrypted_entry: Option<(Arc<EncryptedEntry>, Signature)>,
}

#[derive(Debug, Default)]
pub struct AppState {
    pub records: Mutex<Vec<RecordItem>>,
}

impl ChcService {
    pub fn new(interface: impl Into<Ipv4Addr>, port: u16) -> Self {
        let address = SocketAddr::new(std::net::IpAddr::V4(interface.into()), port);

        let router = Router::new()
            .route("/get_record_data", get(get_record_data))
            .route("/users", post(add_records))
            .with_state(Arc::new(AppState::default()));

        ChcService { address, router }
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let listerner = TcpListener::bind(self.address)
            .await
            .map_err(|e| anyhow!("Failed to bind to address: {}", e))?;
        axum::serve(listerner, self.router)
            .await
            .map_err(|e| anyhow!("Failed to start server: {}", e))?;

        Ok(())
    }
}
