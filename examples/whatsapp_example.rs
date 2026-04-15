//! WhatsApp Integration Example for BeeBotOS
//!
//! This example demonstrates how to use the WhatsApp adapter with the Baileys bridge.

use beebotos_agents::communication::channel::whatsapp::{
    WhatsAppAdapter, WhatsAppConfig, WhatsAppMessageProcessor, WhatsAppProcessorConfig,
};
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting WhatsApp integration example...");

    // Create processor configuration
    let config = WhatsAppProcessorConfig {
        adapter_config: WhatsAppConfig {
            node_path: "node".to_string(),
            bridge_path: "./tools/whatsapp-baileys-bridge/whatsapp-baileys-bridge.js".to_string(),
            auth_dir: "./data/whatsapp/auth".to_string(),
            media_dir: "./data/whatsapp/media".to_string(),
            reconnect_interval_ms: 5000,
            max_reconnect_attempts: 10,
            print_qr: true,
            log_level: "info".to_string(),
        },
        auto_reply: true,
        respond_to_groups: true,
        ..Default::default()
    };

    // Create and start the processor
    let processor = WhatsAppMessageProcessor::new(config).await?;

    // Get the adapter for direct access
    let adapter = processor.adapter();

    // Start the processor (this will connect to WhatsApp)
    info!("Starting WhatsApp processor...");
    info!("Please scan the QR code with your phone when it appears.");
    
    processor.start().await?;

    // Wait for connection
    loop {
        let state = processor.connection_state().await;
        info!("Connection state: {}", state);

        if state == "Connected" {
            info!("Successfully connected to WhatsApp!");
            break;
        }

        if let Some(qr) = processor.qr_code().await {
            info!("QR Code available. Please scan with your phone.");
            // You can display the QR code here using a library like qrcode
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    // Get user info
    let adapter_guard = adapter.read().await;
    if let Some(user) = adapter_guard.user().await {
        info!("Connected as: {} ({})", user.name, user.id);
    }
    drop(adapter_guard);

    // Example: Send a text message
    // let adapter_guard = adapter.read().await;
    // adapter_guard.send_text("1234567890@s.whatsapp.net", "Hello from BeeBotOS!").await?;
    // drop(adapter_guard);

    // Run for a while to process messages
    info!("Running for 60 seconds to process messages...");
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

    // Stop the processor
    info!("Stopping WhatsApp processor...");
    processor.stop().await?;

    info!("Example completed!");
    Ok(())
}
