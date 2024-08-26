use std::sync::Arc;

use axum::{extract::State, Json};
use holochain::{
    core::{Signature, SignedActionHashed},
    prelude::{ChainItem, EncryptedEntry},
};
use holochain_types::chc::GetRecordsRequest;

use crate::chc::{AppState, RecordItem};

type GetRecordDataResult = Vec<(SignedActionHashed, Option<(Arc<EncryptedEntry>, Signature)>)>;

#[tracing::instrument]
pub async fn get_record_data(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<GetRecordsRequest>,
) -> Json<GetRecordDataResult> {
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

    let record_data = records
        .into_iter()
        .map(
            |RecordItem {
                 action,
                 encrypted_entry,
             }| (action, encrypted_entry),
        )
        .collect::<Vec<_>>();

    Json(record_data)
}
