use std::sync::Arc;

use anyhow::Context;
use axum::extract::{Path, State};
use holochain::{
    core::{AgentPubKeyB64, DnaHashB64, Signature, SignedActionHashed},
    prelude::{ChainItem, EncryptedEntry},
};
use holochain_types::chc::GetRecordsRequest;

use crate::{
    chc::{AppState, RecordItem},
    msgpack_utils::MsgPack,
    ChcServiceError,
};

use super::ChcPathParams;

type GetRecordDataResult = Vec<(SignedActionHashed, Option<(Arc<EncryptedEntry>, Signature)>)>;

#[tracing::instrument(skip(app_state))]
pub async fn get_record_data(
    Path(params): Path<ChcPathParams>,
    State(app_state): State<Arc<AppState>>,
    MsgPack(request): MsgPack<GetRecordsRequest>,
) -> Result<MsgPack<GetRecordDataResult>, ChcServiceError> {
    // Ensure that the dna_hash and agent_pubkey params are valid
    _ = DnaHashB64::from_b64_str(&params.dna_hash)
        .context("Failed to get DnaHash from base64 str")?;
    _ = AgentPubKeyB64::from_b64_str(&params.agent_pubkey)
        .context("Failed to get AgentPubkey from base64 str")?;

    let m = app_state.records.lock();
    let records = if let Some(hash) = &request.payload.since_hash {
        m.iter()
            .skip_while(|r| hash != r.action.get_hash())
            .skip(1)
            .cloned()
            .collect()
    } else {
        m.clone()
    };

    if records.is_empty() {
        return Err(ChcServiceError::HashNotFound(
            "Hash was not found in the CHC".to_string(),
        ));
    }

    let record_data = records
        .into_iter()
        .map(
            |RecordItem {
                 action,
                 encrypted_entry,
             }| (action, encrypted_entry),
        )
        .collect::<Vec<_>>();

    Ok(MsgPack(record_data))
}
