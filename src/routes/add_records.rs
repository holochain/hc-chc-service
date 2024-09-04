use std::sync::Arc;

use axum::extract::{Path, State};
use holochain::{
    conductor::chc::AddRecordsRequest,
    prelude::{validate_chain, ChainItem, SignedActionHashed},
};
use holochain_serialized_bytes::SerializedBytesError;

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
    let cell_id = params.try_into()?;

    let mut m = app_state.records.write();
    let records = m.entry(cell_id).or_insert(Default::default());

    let head = records
        .last()
        .map(|r| (r.action.get_hash().clone(), r.action.seq()));

    let records_to_add = request
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
    records.extend(records_to_add);

    Ok(())
}
