use chc_service::{telemetry::initialize_tracing_subscriber, ChcService, GetRecordDataResult};
use fixt::prelude::*;
use holochain::{
    conductor::chc::{AddRecordPayload, GetRecordsPayload, GetRecordsRequest},
    fixt::DnaHashFixturator,
    prelude::{
        hash_type::Agent, Action, ActionHashed, Dna, HoloHash, SignedActionHashed,
        SignedActionHashedExt, Timestamp,
    },
};
use holochain_keystore::{AgentPubKeyExt, MetaLairClient};

#[tokio::test(flavor = "multi_thread")]
async fn test_add_and_get_records() {
    initialize_tracing_subscriber("info");

    // Initialize the ChcService and run it in a background task
    let service = ChcService::new([127, 0, 0, 1], portpicker::pick_unused_port().unwrap());
    let addr = service.address();
    tokio::task::spawn(async move {
        service.run().await.unwrap();
    });

    // Wait for the server to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let keystore = holochain_keystore::test_keystore();
    let agent_pubkey = keystore.new_sign_keypair_random().await.unwrap();
    let dna_hash = fixt!(DnaHash);

    // Call `get_record_data` to check that there are no records initially
    let response = client
        .post(format!(
            "http://{}/get_record_data/{}/{}",
            addr, dna_hash, agent_pubkey
        ))
        .body(
            holochain_serialized_bytes::encode(
                &get_records_request(&keystore, &agent_pubkey).await,
            )
            .unwrap(),
        )
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 498);

    //  Add a record using `add_records`
    let record = add_record_payload(&keystore, &agent_pubkey, &dna_hash).await;

    let response = client
        .post(format!(
            "http://{}/add_records/{}/{}",
            addr, dna_hash, agent_pubkey
        ))
        .body(holochain_serialized_bytes::encode(&[record]).unwrap())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    // Call `get_record_data` again to ensure the record was added
    let response = client
        .post(format!(
            "http://{}/get_record_data/{}/{}",
            addr, dna_hash, agent_pubkey
        ))
        .body(
            holochain_serialized_bytes::encode(
                &get_records_request(&keystore, &agent_pubkey).await,
            )
            .unwrap(),
        )
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let bytes = response.bytes().await.unwrap();
    let result: GetRecordDataResult = holochain_serialized_bytes::decode(&bytes).unwrap();
    assert_eq!(result.len(), 1);
}

async fn add_record_payload(
    keystore: &MetaLairClient,
    agent_pubkey: &HoloHash<Agent>,
    dna_hash: &holochain_types::dna::DnaHash,
) -> AddRecordPayload {
    let dna_action = Dna {
        author: agent_pubkey.clone(),
        hash: dna_hash.clone(),
        timestamp: Timestamp::now(),
    };

    let action = Action::Dna(dna_action);
    let action_hashed = ActionHashed::from_content_sync(action);

    let signed_action_hashed = SignedActionHashed::sign(keystore, action_hashed)
        .await
        .unwrap();

    AddRecordPayload {
        signed_action_msgpack: holochain_serialized_bytes::encode(&signed_action_hashed).unwrap(),
        signed_action_signature: signed_action_hashed.signature,
        encrypted_entry: None,
    }
}

async fn get_records_request(
    keystore: &MetaLairClient,
    agent_pubkey: &HoloHash<Agent>,
) -> GetRecordsRequest {
    let get_records_payload = GetRecordsPayload {
        since_hash: None,
        nonce: holochain_nonce::fresh_nonce(Timestamp::now()).unwrap().0,
    };

    let signature = agent_pubkey
        .sign(keystore, &get_records_payload)
        .await
        .unwrap();

    GetRecordsRequest {
        payload: get_records_payload,
        signature,
    }
}
