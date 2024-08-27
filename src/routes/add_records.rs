use std::sync::Arc;

use axum::{extract::State, Json};
use holochain::{
    core::{validate_chain, SignedActionHashed},
    prelude::ChainItem,
};
use holochain_serialized_bytes::SerializedBytesError;
use holochain_types::chc::AddRecordsRequest;

use crate::{
    chc::{AppState, RecordItem},
    ChcServiceError,
};

#[tracing::instrument(skip(app_state))]
pub async fn add_records(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<AddRecordsRequest>,
) -> Result<(), ChcServiceError> {
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
    validate_chain(actions, &head).map_err(|e| ChcServiceError::InternalError(e.into()))?;
    m.extend(records);

    Ok(())
}
