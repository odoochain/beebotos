//! 飞书 Channel 使用示例
//!
//! 演示如何使用新的统一 Channel trait 连接飞书

use beebot_agents::communication::channel::{
    Channel, ChannelConfig, ChannelEvent, ConnectionMode, LarkChannel, LarkConfig,
};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 从环境变量加载配置
    let config = LarkConfig::from_env().expect("请设置 LARK_APP_ID 和 LARK_APP_SECRET");

    println!("飞书配置:");
    println!("  App ID: {}", config.app_id);
    println!("  连接模式: {}", config.connection_mode);
    println!("  自动重连: {}", config.auto_reconnect);
    println!("  最大重连次数: {}", config.max_reconnect_attempts);

    // 创建 Channel
    let mut channel = LarkChannel::new(config);

    // 连接
    channel.connect().await?;
    println!("✅ 飞书连接成功");

    // 创建事件通道
    let (event_tx, mut event_rx) = mpsc::channel::<ChannelEvent>(100);

    // 启动监听
    let channel_ref = &channel;
    let listener_handle = tokio::spawn(async move {
        if let Err(e) = channel_ref.start_listener(event_tx).await {
            eprintln!("监听错误: {}", e);
        }
    });

    // 处理事件
    println!("🎧 开始监听消息...");
    while let Some(event) = event_rx.recv().await {
        match event {
            ChannelEvent::MessageReceived {
                platform,
                channel_id,
                message,
            } => {
                println!(
                    "📨 收到消息 [{}]: {} - {}",
                    platform, channel_id, message.content
                );

                // 回复消息
                let reply = beebot_agents::communication::Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    thread_id: Some(channel_id.clone()),
                    platform,
                    message_type: beebot_agents::communication::MessageType::Text,
                    content: format!("收到: {}", message.content),
                    sender_id: "bot".to_string(),
                    sender_name: Some("BeeBot".to_string()),
                    timestamp: chrono::Utc::now(),
                    metadata: Default::default(),
                };

                if let Err(e) = channel.send(&channel_id, &reply).await {
                    eprintln!("发送回复失败: {}", e);
                }
            }
            ChannelEvent::ConnectionStateChanged {
                platform,
                connected,
                reason,
            } => {
                println!(
                    "🔗 连接状态变化 [{}]: connected={}, reason={:?}",
                    platform, connected, reason
                );
            }
            ChannelEvent::Error { platform, error } => {
                eprintln!("❌ 错误 [{}]: {}", platform, error);
            }
        }
    }

    // 等待监听器结束
    listener_handle.await?;

    // 断开连接
    channel.disconnect().await?;
    println!("👋 已断开连接");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lark_channel_lifecycle() {
        // 设置测试环境变量
        std::env::set_var("LARK_APP_ID", "test_app_id");
        std::env::set_var("LARK_APP_SECRET", "test_app_secret");
        std::env::set_var("LARK_CONNECTION_MODE", "websocket");

        let config = LarkConfig::from_env().unwrap();
        assert_eq!(config.connection_mode, ConnectionMode::WebSocket);

        let mut channel = LarkChannel::new(config);
        assert_eq!(channel.name(), "lark");
        assert!(!channel.is_connected());
    }
}
