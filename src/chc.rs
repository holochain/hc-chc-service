use std::{
    collections::BTreeMap,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use anyhow::anyhow;
use axum::{routing::post, Router};
use holochain::{
    conductor::chc::EncryptedEntry,
    prelude::{CellId, Signature, SignedActionHashed},
};
use parking_lot::RwLock;
use tokio::net::TcpListener;

use crate::routes::{add_records, get_record_data, not_found};

#[derive(Debug)]
pub struct ChcService {
    address: SocketAddr,
    router: Router,
}

// Sourced from holochain sourcecode, original struct contains inacessible private fields
// https://github.com/holochain/holochain/blob/60102a603b8039eb46e786d82dd6382f3a1b1c93/crates/holochain/src/conductor/chc/chc_local.rs#L33
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RecordItem {
    pub action: SignedActionHashed,
    pub encrypted_entry: Option<(Arc<EncryptedEntry>, Signature)>,
}

#[derive(Debug, Default)]
pub struct AppState {
    pub records: RwLock<BTreeMap<CellId, Vec<RecordItem>>>,
}

impl ChcService {
    pub fn new(interface: impl Into<IpAddr>, port: u16) -> Self {
        let address = SocketAddr::new(interface.into(), port);

        let router = Router::new()
            .route(
                "/:dna_hash/:agent_pubkey/get_record_data",
                post(get_record_data),
            )
            .route("/:dna_hash/:agent_pubkey/add_records", post(add_records))
            .fallback(not_found)
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
