use beebotos_agents::communication::channel::{
    ChannelRegistry, WeChatFactory, ChannelEvent
};
use beebotos_agents::communication::{Message, MessageType, PlatformType};
use serde_json::json;
use tokio::sync::mpsc;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 BeeBotOS 企业微信测试");
    println!("═══════════════════════════════════════════════════");
    
    // 从环境变量加载配置
    let corp_id = std::env::var("WECHAT_CORP_ID")
        .expect("WECHAT_CORP_ID not set");
    let agent_id = std::env::var("WECHAT_AGENT_ID")
        .expect("WECHAT_AGENT_ID not set");
    let corp_secret = std::env::var("WECHAT_CORP_SECRET")
        .expect("WECHAT_CORP_SECRET not set");
    
    println!("✓ 企业ID: {}", corp_id);
    println!("✓ 应用ID: {}", agent_id);
    println!("✓ Secret: {}...", &corp_secret[..10]);
    
    // 创建事件总线
    let (event_bus_tx, mut event_bus_rx) = mpsc::channel(100);
    
    // 创建通道注册器
    let mut registry = ChannelRegistry::new(event_bus_tx);
    
    // 注册企业微信工厂
    registry.register(Box::new(WeChatFactory::new())).await;
    println!("✓ 企业微信工厂已注册");
    
    // 创建配置 - 使用轮询模式接收消息
    let config = json!({
        "corp_id": corp_id,
        "corp_secret": corp_secret,
        "agent_id": agent_id,
        "connection_mode": "poll"
    });
    
    // 创建通道
    let channel_name = registry
        .create_channel("wechat", &config, Some("企业微信"))
        .await?;
    
    println!("✓ 通道已创建: {}", channel_name);
    println!("");
    println!("📱 企业微信连接成功！");
    println!("═══════════════════════════════════════════════════");
    println!("");
    println!("提示：请在企业微信中给应用发送消息进行测试");
    println!("");
    
    // 处理消息循环
    while let Some(event) = event_bus_rx.recv().await {
        match event {
            ChannelEvent::MessageReceived { platform, channel_id, message } => {
                println!("");
                println!("📨 收到消息:");
                println!("   平台: {:?}", platform);
                println!("   发送者: {}", channel_id);
                println!("   内容: {}", message.content);
                
                // 自动回复
                let reply = Message {
                    id: Uuid::new_v4(),
                    thread_id: message.thread_id,
                    platform: PlatformType::WeChat,
                    message_type: MessageType::Text,
                    content: format!("收到: {}", message.content),
                    metadata: HashMap::new(),
                    timestamp: chrono::Utc::now(),
                };
                
                match registry.get_channel(&channel_name).await {
                    Some(channel) => {
                        match channel.write().await.send(&channel_id, &reply).await {
                            Ok(_) => println!("✅ 回复已发送"),
                            Err(e) => println!("❌ 发送失败: {}", e),
                        }
                    }
                    None => println!("❌ 获取通道失败"),
                }
            }
            ChannelEvent::ConnectionStateChanged { platform, connected, .. } => {
                println!("📡 连接状态: {:?} -> {}", 
                    platform, 
                    if connected { "已连接" } else { "断开" }
                );
            }
            _ => {}
        }
    }
    
    Ok(())
}
