use std::sync::Arc;

use axum::extract::{Path, State};
use holochain::{
    conductor::chc::AddRecordsRequest,
    prelude::{validate_chain, ChainItem, SignedActionHashed},
};
use holochain_serialized_bytes::SerializedBytesError;

use crate::{
    chc::{AppState, CellState, RecordItem},
    msgpack_utils::MsgPack,
    ChcServiceError,
};

use super::ChcPathParams;

#[tracing::instrument(skip(app_state, request))]
pub async fn add_records(
    Path(params): Path<ChcPathParams>,
    State(app_state): State<Arc<AppState>>,
    MsgPack(request): MsgPack<AddRecordsRequest>,
) -> Result<(), ChcServiceError> {
    if request.is_empty() {
        return Ok(());
    }

    let cell_id = params.try_into()?;

    let mut m = app_state.records.write();
    let CellState {
        records,
        latest_action_hash,
        latest_action_seq,
    } = m.entry(cell_id).or_insert(Default::default());

    let head = records
        .last()
        .map(|r| (r.action.get_hash().clone(), r.action.seq()));

    let records_to_add = request.into_iter().try_fold(Vec::new(), |mut acc, r| {
        let signed_action: Result<SignedActionHashed, SerializedBytesError> =
            holochain_serialized_bytes::decode(&r.signed_action_msgpack);

        let record_item = signed_action
            .map(|action| {
                *latest_action_hash = Some(action.as_hash().clone());
                *latest_action_seq = action.seq();

                if r.encrypted_entry.is_some() && action.action().entry_hash().is_none() {
                    return Err(ChcServiceError::BadRequest(
                        "Unexpected encrypted entry provided with action in payload".to_string(),
                    ));
                }

                Ok(RecordItem {
                    action,
                    encrypted_entry: r.encrypted_entry,
                })
            })
            .map_err(|e| ChcServiceError::InternalError(e.into()))?;

        acc.push(record_item?);
        Ok::<Vec<RecordItem>, ChcServiceError>(acc)
    })?;

    let actions = records.iter().map(|r| &r.action);
    validate_chain(actions, &head)?;
    records.extend(records_to_add);

    Ok(())
}
