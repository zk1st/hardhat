use edr_defaults::default_node_config;
use edr_rpc_server::Node;
use edr_test_utils::fixture::CacheDirTestFixture;

#[tokio::test]
async fn chain_id() -> Result<()> {
    let config = default_node_config();
    let node = Node::new()
    let fixture = CacheDirTestFixture::new()
    let fixture = NodeTestFixture::new().await?;

    let chain_id = fixture.node.chain_id().await;
    assert_eq!(chain_id, U64::from(fixture.config.chain_id));

    Ok(())
}
