mod add_records;
mod get_record_data;
mod not_found;

use anyhow::Context;
use holochain::{
    core::{AgentPubKeyB64, DnaHashB64},
    prelude::CellId,
};

pub use add_records::add_records;
pub use get_record_data::{get_record_data, GetRecordDataResult};
pub use not_found::not_found;

#[derive(Debug, serde::Deserialize)]
#[allow(unused)]
pub struct ChcPathParams {
    pub dna_hash: String,
    pub agent_pubkey: String,
}

impl TryInto<CellId> for ChcPathParams {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<CellId, Self::Error> {
        let dna_hash = DnaHashB64::from_b64_str(&self.dna_hash)
            .context("Failed to get DnaHash from base64 str")?;
        let agent_pubkey = AgentPubKeyB64::from_b64_str(&self.agent_pubkey)
            .context("Failed to get AgentPubkey from base64 str")?;
        Ok(CellId::new(dna_hash.into(), agent_pubkey.into()))
    }
}
