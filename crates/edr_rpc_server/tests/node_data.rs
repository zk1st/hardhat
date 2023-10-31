#[tokio::test]
async fn next_filter_id() -> Result<()> {
    let mut fixture = NodeDataTestFixture::new().await?;

    let mut prev_filter_id = fixture.node_data.last_filter_id;
    for _ in 0..10 {
        let filter_id = fixture.node_data.next_filter_id();
        assert!(prev_filter_id < filter_id);
        prev_filter_id = filter_id;
    }

    Ok(())
}
