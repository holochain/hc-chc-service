use std::sync::Arc;

use anyhow::Context;
use axum::extract::{Path, State};
use holochain::{
    core::{validate_chain, AgentPubKeyB64, DnaHashB64, SignedActionHashed},
    prelude::ChainItem,
};
use holochain_serialized_bytes::SerializedBytesError;
use holochain_types::chc::AddRecordsRequest;

use crate::{
    chc::{AppState, RecordItem},
    msgpack_utils::MsgPack,
    ChcServiceError,
};

use super::ChcPathParams;

#[tracing::instrument(skip(app_state))]
pub async fn add_records(
    Path(params): Path<ChcPathParams>,
    State(app_state): State<Arc<AppState>>,
    MsgPack(request): MsgPack<AddRecordsRequest>,
) -> Result<(), ChcServiceError> {
    // Ensure that the dna_hash and agent_pubkey params are valid
    _ = DnaHashB64::from_b64_str(&params.dna_hash)
        .context("Failed to get DnaHash from base64 str")?;
    _ = AgentPubKeyB64::from_b64_str(&params.agent_pubkey)
        .context("Failed to get AgentPubkey from base64 str")?;

    let mut m = app_state.records.lock();

    let head = m
        .last()
        .map(|r| (r.action.get_hash().clone(), r.action.seq()));

    let records = request
        .into_iter()
        .map(|r| {
            let signed_action: Result<SignedActionHashed, SerializedBytesError> =
                holochain_serialized_bytes::decode(&r.signed_action_msgpack);

            signed_action
                .map(|action| RecordItem {
                    action,
                    encrypted_entry: r.encrypted_entry,
                })
                .map_err(|e| ChcServiceError::InternalError(e.into()))
        })
        .collect::<Result<Vec<_>, ChcServiceError>>()?;

    let actions = records.iter().map(|r| &r.action);
    validate_chain(actions, &head)?;
    m.extend(records);

    Ok(())
}
