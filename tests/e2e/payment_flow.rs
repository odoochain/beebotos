use beebotos_chain::{ChainClient, PaymentClient};
use beebotos_test_utils::TestContext;

#[tokio::test]
async fn test_payment_mandate() {
    let ctx = TestContext::new().await;
    let client = ChainClient::new(&ctx.chain_config).await.unwrap();

    // Create mandate
    let mandate = client
        .create_mandate(
            "0xGranteeAddress",
            "1000000000000000000", // 1 BEE
            None,
            30, // 30 days
        )
        .await
        .unwrap();

    assert!(!mandate.id.is_empty());

    // Spend from mandate
    let tx = client
        .spend_from_mandate(&mandate.id, "500000000000000000", "0xRecipient")
        .await
        .unwrap();

    assert!(tx.success);
}

#[tokio::test]
async fn test_escrow_payment() {
    let ctx = TestContext::new().await;
    let client = PaymentClient::new(&ctx.chain_config).await.unwrap();

    // Create escrow
    let escrow = client
        .create_escrow(
            "0xSellerAddress",
            "1000000000000000000",
            "deal_123",
        )
        .await
        .unwrap();

    assert_eq!(escrow.state, "LOCKED");

    // Confirm delivery
    let result = client.confirm_delivery(&escrow.id).await.unwrap();
    assert!(result.success);
}

#[tokio::test]
async fn test_streaming_payment() {
    let ctx = TestContext::new().await;
    let client = PaymentClient::new(&ctx.chain_config).await.unwrap();

    // Create stream
    let stream = client
        .create_stream(
            "0xRecipientAddress",
            "1000000000", // tokens per second
            86400, // 1 day
        )
        .await
        .unwrap();

    assert_eq!(stream.status, "ACTIVE");

    // Withdraw
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let withdrawn = client.withdraw_from_stream(&stream.id).await.unwrap();
    assert!(withdrawn > 0);
}
