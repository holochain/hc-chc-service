mod add_records;
mod get_record_data;

pub use add_records::add_records;
pub use get_record_data::{get_record_data, GetRecordDataResult};

#[derive(Debug, serde::Deserialize)]
#[allow(unused)]
pub struct ChcPathParams {
    pub dna_hash: String,
    pub agent_pubkey: String,
}
