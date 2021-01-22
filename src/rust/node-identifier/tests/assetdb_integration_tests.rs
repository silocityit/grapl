#![cfg(feature = "integration")]

use grapl_graph_descriptions::graph_description::host::*;
use node_identifier::{assetdb::AssetIdDb,};
use tokio::runtime::Runtime;
use grapl_config::env_helpers::FromEnv;
use rusoto_dynamodb::DynamoDbClient;

// Given a hostname 'H' to asset id 'A' mapping at c_timestamp 'X'
// When attributing 'H' at c_timestamp 'Y', where 'Y' > 'X'
// Then we should retrieve asset id 'A'
#[test]
fn map_hostname_to_asset_id() {
    let mut runtime = Runtime::new().unwrap();

    let asset_id_db = AssetIdDb::new(DynamoDbClient::from_env());

    runtime
        .block_on(asset_id_db.create_mapping(
            &HostId::Hostname("fakehostname".to_owned()),
            "asset_id_a".into(),
            1500,
        ))
        .expect("Mapping creation failed");

    let mapping = runtime
        .block_on(asset_id_db.resolve_asset_id(&HostId::Hostname("fakehostname".to_owned()), 1510))
        .expect("Failed to resolve asset id mapping")
        .expect("Failed to resolve asset id mapping");

    assert_eq!(mapping, "asset_id_a");
}