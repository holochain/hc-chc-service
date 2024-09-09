use chc_service::{telemetry::initialize_tracing_subscriber, ChcService, GetRecordDataResult};
use fixt::prelude::*;
use holochain::{
    conductor::chc::{AddRecordPayload, GetRecordsPayload, GetRecordsRequest},
    fixt::DnaHashFixturator,
    prelude::{
        hash_type::Agent, Action, ActionHashed, AgentValidationPkg, Create, Dna, HoloHash, Record,
        SignedActionHashed, SignedActionHashedExt, Timestamp,
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
    let genesis_records = genesis_records(&keystore, &agent_pubkey, &dna_hash).await;

    let response = client
        .post(format!(
            "http://{}/add_records/{}/{}",
            addr, dna_hash, agent_pubkey
        ))
        .body(holochain_serialized_bytes::encode(&genesis_records).unwrap())
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
    assert_eq!(result.len(), 3);
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

pub async fn genesis_records(
    keystore: &MetaLairClient,
    agent_pubkey: &HoloHash<Agent>,
    dna_hash: &holochain_types::dna::DnaHash,
) -> Vec<AddRecordPayload> {
    // DNA
    let dna_action = Action::Dna(Dna {
        author: agent_pubkey.clone(),
        timestamp: Timestamp::now(),
        hash: dna_hash.clone(),
    });
    let dna_action = ActionHashed::from_content_sync(dna_action);
    let dna_action = SignedActionHashed::sign(keystore, dna_action)
        .await
        .unwrap();
    let dna_action_address = dna_action.as_hash().clone();
    let dna_record = Record::new(dna_action, None);

    // Agent Validation
    let agent_validation_action = Action::AgentValidationPkg(AgentValidationPkg {
        author: agent_pubkey.clone(),
        timestamp: Timestamp::now(),
        action_seq: 1,
        prev_action: dna_action_address,
        membrane_proof: None,
    });
    let agent_validation_action = ActionHashed::from_content_sync(agent_validation_action);
    let agent_validation_action = SignedActionHashed::sign(keystore, agent_validation_action)
        .await
        .unwrap();
    let agent_validation_address = agent_validation_action.as_hash().clone();
    let agnet_validation_record = Record::new(agent_validation_action, None);

    // Agent Action
    let agent_action = Action::Create(Create {
        author: agent_pubkey.clone(),
        timestamp: Timestamp::now(),
        action_seq: 2,
        prev_action: agent_validation_address,
        entry_type: holochain::prelude::EntryType::AgentPubKey,
        entry_hash: agent_pubkey.clone().into(),
        weight: Default::default(),
    });
    let agent_action = ActionHashed::from_content_sync(agent_action);
    let agent_action = SignedActionHashed::sign(keystore, agent_action)
        .await
        .unwrap();
    let agent_record = Record::new(
        agent_action,
        Some(holochain::prelude::Entry::Agent(agent_pubkey.clone())),
    );

    AddRecordPayload::from_records(
        keystore.clone(),
        agent_pubkey.clone(),
        vec![dna_record, agnet_validation_record, agent_record],
    )
    .await
    .unwrap()
}
