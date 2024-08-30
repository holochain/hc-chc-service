use std::sync::Arc;

use axum::extract::{Path, State};
use holochain::{
    core::{Signature, SignedActionHashed},
    prelude::{ChainItem, EncryptedEntry},
};
use holochain_types::chc::GetRecordsRequest;

use crate::{
    chc::{AppState, RecordItem},
    msgpack_utils::MsgPack,
    ChcServiceError,
};

use super::ChcPathParams;

pub type GetRecordDataResult = Vec<(SignedActionHashed, Option<(Arc<EncryptedEntry>, Signature)>)>;

#[tracing::instrument(skip(app_state))]
pub async fn get_record_data(
    Path(params): Path<ChcPathParams>,
    State(app_state): State<Arc<AppState>>,
    MsgPack(request): MsgPack<GetRecordsRequest>,
) -> Result<MsgPack<GetRecordDataResult>, ChcServiceError> {
    let cell_id = params.try_into()?;

    let m = app_state.records.read();
    let records = match m.get(&cell_id) {
        Some(records) if records.is_empty() => {
            return Err(ChcServiceError::HashNotFound(
                "Hash was not found in the CHC".to_string(),
            ))
        }
        Some(records) => records,
        None => {
            return Err(ChcServiceError::HashNotFound(
                "Hash was not found in the CHC".to_string(),
            ))
        }
    };

    let records = if let Some(hash) = &request.payload.since_hash {
        records
            .iter()
            .skip_while(|r| hash != r.action.get_hash())
            .skip(1)
            .cloned()
            .collect()
    } else {
        records.clone()
    };

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
