use chc_service::{telemetry::initialize_tracing_subscriber, ChcService, RecordItem};
use fixt::*;
use holochain::{
    core::{
        hash_type::Agent, Action, ActionHashed, Dna, DnaHashFixturator, HoloHash,
        SignedActionHashed, Timestamp,
    },
    prelude::{AddRecordPayload, GetRecordsPayload, GetRecordsRequest, SignedActionHashedExt},
};
use holochain_keystore::{AgentPubKeyExt, MetaLairClient};
use holochain_nonce::fresh_nonce;
use reqwest::Client;
use serde_json::json;
use tokio::task;

#[tokio::test(flavor = "multi_thread")]
async fn test_add_and_get_records() {
    initialize_tracing_subscriber("info");

    // Initialize the ChcService and run it in a background task
    let service = ChcService::new([127, 0, 0, 1], portpicker::pick_unused_port().unwrap());
    let addr = service.address();
    task::spawn(async move {
        service.run().await.unwrap();
    });

    // Wait for the server to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = Client::new();
    let keystore = holochain_keystore::test_keystore();
    let agent_pubkey = keystore.new_sign_keypair_random().await.unwrap();

    // Call `get_record_data` to check that there are no records initially
    let response = client
        .get(format!("http://{}/get_record_data", addr))
        .json(&get_records_request(&keystore, &agent_pubkey).await)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let records: Vec<RecordItem> = response.json().await.expect("Failed to parse response");
    assert_eq!(records.len(), 0, "Expected no records initially");

    //  Add a record using `add_records`
    let record = add_record_payload(&keystore, &agent_pubkey).await;

    let response = client
        .post(format!("http://{}/add_records", addr))
        .json(&json!([&record]))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    // Call `get_record_data` again to ensure the record was added
    let response = client
        .get(format!("http://{}/get_record_data", addr))
        .json(&get_records_request(&keystore, &agent_pubkey).await)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let records: Vec<RecordItem> = response.json().await.expect("Failed to parse response");
    println!("{:?}", records);
    assert_eq!(records.len(), 1, "Expected one record after insertion");
}

async fn add_record_payload(
    keystore: &MetaLairClient,
    agent_pubkey: &HoloHash<Agent>,
) -> AddRecordPayload {
    let dna_action = Dna {
        author: agent_pubkey.clone(),
        hash: fixt!(DnaHash),
        timestamp: Timestamp::now(),
    };

    let action = Action::Dna(dna_action);
    let action_hashed = ActionHashed::from_content_sync(action);

    let signed_action_hashed = SignedActionHashed::sign(&keystore, action_hashed)
        .await
        .unwrap();

    let add_record_payload = AddRecordPayload {
        signed_action_msgpack: holochain_serialized_bytes::encode(&signed_action_hashed).unwrap(),
        signed_action_signature: signed_action_hashed.signature,
        encrypted_entry: None,
    };

    add_record_payload
}

async fn get_records_request(
    keystore: &MetaLairClient,
    agent_pubkey: &HoloHash<Agent>,
) -> GetRecordsRequest {
    let get_records_payload = GetRecordsPayload {
        since_hash: None,
        nonce: fresh_nonce(Timestamp::now()).unwrap().0,
    };

    let signature = agent_pubkey
        .sign(&keystore, &get_records_payload)
        .await
        .unwrap();

    GetRecordsRequest {
        payload: get_records_payload,
        signature,
    }
}
