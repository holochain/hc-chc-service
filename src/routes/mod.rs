mod add_records;
mod get_record_data;

pub use add_records::add_records;
pub use get_record_data::get_record_data;

// Path params passed to the CHC POST routes, they are not used in this reference
// implementation
#[derive(Debug, serde::Deserialize)]
#[allow(unused)]
pub struct PathParams {
    pub dna_hash: String,
    pub agent_pubkey: String,
}
