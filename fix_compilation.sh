#!/bin/bash
# 修复 beebotos-agents 编译错误的脚本

cd /home/beebotos/crates/agents

echo "=== 1. 修复 teams.rs 的 Git 合并冲突 ==="
cat > /tmp/fix_teams.py << 'EOF'
import re

with open('src/communication/channel/teams.rs', 'r') as f:
    content = f.read()

# 移除 Git 合并冲突标记
content = re.sub(r'<<<<<<< HEAD\n.*?=======(.*?)>>>>>>> wang\n', r'\1', content, flags=re.DOTALL)

with open('src/communication/channel/teams.rs', 'w') as f:
    f.write(content)

print("Fixed teams.rs")
EOF
python3 /tmp/fix_teams.py

echo "=== 2. 修复 signal_channel.rs - 添加缺少的导入 ==="
sed -i '1s/^/use tokio::io::{AsyncWriteExt, BufReader};\nuse tokio::io::AsyncBufReadExt;\n/' src/communication/channel/signal_channel.rs

echo "=== 3. 修复 media/downloader.rs - 添加 Semaphore 导入 ==="
sed -i 's/use tokio::sync::Semaphore;/use tokio::sync::Semaphore;\nuse std::sync::Arc;/' src/media/downloader.rs

echo "=== 4. 修复 error.rs - 添加缺少的错误变体 ==="
cat > /tmp/fix_error.py << 'EOF'
import re

with open('src/error.rs', 'r') as f:
    content = f.read()

# 添加缺少的错误变体
new_variants = '''
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Message receive failed: {0}")]
    MessageReceiveFailed(String),

    #[error("Message send failed: {0}")]
    MessageSendFailed(String),

    #[error("Not connected: {0}")]
    NotConnected(String),

    #[error("Rate limited: {0}")]
    RateLimited(String),
'''

# 在 AgentError enum 的定义后添加新变体
content = content.replace(
    '''#[derive(Error, Debug, Clone)]
pub enum AgentError {''',
    f'''#[derive(Error, Debug, Clone)]
pub enum AgentError {{{new_variants}''')

# 添加对应的构造函数
new_methods = '''
    pub fn authentication_failed<S: Into<String>>(msg: S) -> Self {
        Self::AuthenticationFailed(msg.into())
    }

    pub fn message_receive_failed<S: Into<String>>(msg: S) -> Self {
        Self::MessageReceiveFailed(msg.into())
    }

    pub fn message_send_failed<S: Into<String>>(msg: S) -> Self {
        Self::MessageSendFailed(msg.into())
    }

    pub fn not_connected<S: Into<String>>(msg: S) -> Self {
        Self::NotConnected(msg.into())
    }

    pub fn rate_limited<S: Into<String>>(msg: S) -> Self {
        Self::RateLimited(msg.into())
    }
'''

# 在 impl AgentError 中添加新方法
content = content.replace(
    'impl AgentError {',
    f'impl AgentError {{{new_methods}'
)

with open('src/error.rs', 'w') as f:
    f.write(content)

print("Fixed error.rs")
EOF
python3 /tmp/fix_error.py

echo "=== 5. 修复 lib.rs - 声明 wallet 模块 ==="
sed -i '/pub mod types;/a pub mod wallet;' src/lib.rs

echo "=== 6. 修复 imessage_processor.rs - 添加 axum 导入 ==="
sed -i 's/use axum::routing::post;/use axum::extract::{State, Json};\nuse axum::http::StatusCode;\nuse axum::routing::post;/' src/communication/channel/imessage_processor.rs

echo "=== 7. 修复 lib.rs - 处理 disconnect_all 返回类型 ==="
cat > /tmp/fix_lib.py << 'EOF'
with open('src/lib.rs', 'r') as f:
    content = f.read()

# 修复 disconnect_all 结果处理
content = content.replace(
    '''for result in results {
                    match result {
                        Ok(()) => info!("Platform disconnected successfully"),
                        Err(e) => warn!("Error disconnecting platform: {}", e),
                    }
                }''',
    '''for (_platform, result) in results {
                    match result {
                        Ok(()) => info!("Platform disconnected successfully"),
                        Err(e) => warn!("Error disconnecting platform: {}", e),
                    }
                }''')

with open('src/lib.rs', 'w') as f:
    f.write(content)

print("Fixed lib.rs")
EOF
python3 /tmp/fix_lib.py

echo "=== 8. 修复 spawning/workspace.rs - 使用正确的错误方法 ==="
sed -i 's/AgentError::initialization_failed/AgentError::initialization_failed/g' src/spawning/workspace.rs

echo "=== 9. 修复 communication/mod.rs - 类型不匹配 ==="
sat > /tmp/fix_comm.py << 'EOF'
with open('src/communication/mod.rs', 'r') as f:
    content = f.read()

# 修复 model_router 类型
content = content.replace(
    'model_router: router,',
    'model_router: Arc::new(RwLock::new(router)),'
)

content = content.replace(
    'Ok(Self { model_router: router })',
    'Ok(Self { model_router: Arc::new(RwLock::new(router)) })'
)

# 添加 RwLock 读取
content = content.replace(
    'let response = self.model_router.complete(request).await',
    'let response = self.model_router.read().await.complete(request).await'
)

content = content.replace(
    'self.model_router.complete_stream(request).await',
    'self.model_router.read().await.complete_stream(request).await'
)

with open('src/communication/mod.rs', 'w') as f:
    f.write(content)

print("Fixed communication/mod.rs")
EOF
python3 /tmp/fix_comm.py

echo "=== 10. 修复 message_pipeline.rs - MediaDownloader 返回类型 ==="
sed -i 's/downloader: MediaDownloader::default(),/downloader: MediaDownloader::default().expect("Failed to create MediaDownloader"),/' src/communication/channel/message_pipeline.rs

echo "=== 11. 修复 twitter.rs - 使用正确的错误方法 ==="
sed -i 's/ChannelError::AuthenticationFailed/AgentError::authentication_failed/g' src/communication/channel/twitter.rs
sed -i 's/ChannelError::MessageReceiveFailed/AgentError::message_receive_failed/g' src/communication/channel/twitter.rs
sed -i 's/ChannelError::MessageSendFailed/AgentError::message_send_failed/g' src/communication/channel/twitter.rs
sed -i 's/ChannelError::NotConnected/AgentError::not_connected/g' src/communication/channel/twitter.rs
sed -i 's/ChannelError::RateLimited/AgentError::rate_limited/g' src/communication/channel/twitter.rs

echo "=== 12. 添加 AgentError::From 实现 ==="
cat > /tmp/fix_from.py << 'EOF'
with open('src/error.rs', 'r') as f:
    content = f.read()

# 添加 From<serde_json::Error> 实现
from_impl = '''
impl From<serde_json::Error> for AgentError {
    fn from(err: serde_json::Error) -> Self {
        AgentError::Execution(format!("JSON error: {}", err))
    }
}
'''

if 'impl From<serde_json::Error> for AgentError' not in content:
    content = content + from_impl

with open('src/error.rs', 'w') as f:
    f.write(content)

print("Added From<serde_json::Error>")
EOF
python3 /tmp/fix_from.py

echo "=== 所有修复完成 ==="
