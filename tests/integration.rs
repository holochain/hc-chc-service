use chc_service::{ChcService, RecordItem};
use holochain::core::SignedActionHashed;
use reqwest::Client;
use serde_json::json;
use tokio::task;

#[tokio::test]
async fn test_add_and_get_records() {
    // Initialize the ChcService and run it in a background task
    let service = ChcService::new([127, 0, 0, 1], portpicker::pick_unused_port().unwrap());
    let addr = service.address();
    task::spawn(async move {
        service.run().await.unwrap();
    });

    // Wait for the server to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = Client::new();

    // Call `get_record_data` to check that there are no records initially
    let response = client
        .get(format!("http://{}/get_record_data", addr))
        .json(&json!({
            "payload": {
                "since_hash": null,
                "nonce": null
            },
            "singnature": null
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let records: Vec<RecordItem> = response.json().await.expect("Failed to parse response");
    assert_eq!(records.len(), 0, "Expected no records initially");

    //  Add a record using `add_records`
    let record = RecordItem {
        action: create_signed_action_hashed(),
        encrypted_entry: None,
    };

    let response = client
        .post(format!("http://{}/add_records", addr))
        .json(&json!([record.clone()]))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200, "Failed to add a record");

    // Call `get_record_data` again to ensure the record was added
    let response = client
        .get(format!("http://{}/get_record_data", addr))
        .json(&json!({
            "payload": {
                "since_hash": null,
                "nonce": null
            },
            "signature": null
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let records: Vec<RecordItem> = response.json().await.expect("Failed to parse response");
    assert_eq!(records.len(), 1, "Expected one record after insertion");
}

fn create_signed_action_hashed() -> SignedActionHashed {
    todo!()
}
