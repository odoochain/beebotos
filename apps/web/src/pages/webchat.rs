//! WebChat 页面
//!
//! 提供聊天界面、会话管理、侧边提问等功能
//! 已接入 WebChat Channel：通过 WebSocket 接收 Agent 回复，通过 HTTP POST 发送消息

use leptos::prelude::*;
use leptos::view;
use leptos_meta::Title;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::api::{create_webchat_service, create_client};
use crate::components::webchat::{MessageInput, MessageList};
use crate::state::{use_chat_ui_state, use_webchat_state};
use crate::webchat::{ChatMessage, ChatSession, MessageRole};
use gloo_storage::{LocalStorage, Storage};

/// 获取或创建持久化的用户 ID
fn get_user_id() -> String {
    LocalStorage::get("beebotos_webchat_user_id")
        .unwrap_or_else(|_| {
            let id = uuid::Uuid::new_v4().to_string();
            let _ = LocalStorage::set("beebotos_webchat_user_id", &id);
            id
        })
}

/// 获取或创建持久化的会话 ID
fn get_session_id() -> String {
    LocalStorage::get("beebotos_webchat_session_id")
        .unwrap_or_else(|_| {
            let id = uuid::Uuid::new_v4().to_string();
            let _ = LocalStorage::set("beebotos_webchat_session_id", &id);
            id
        })
}

/// WebChat 页面
#[component]
pub fn WebchatPage() -> impl IntoView {
    let chat_state = use_webchat_state();
    let ui_state = use_chat_ui_state();

    // 初始化默认会话
    if chat_state.current_session_id.get().is_none() {
        let session_id = get_session_id();
        let session = ChatSession::new("New Chat");
        let session_id_clone = session_id.clone();
        chat_state.sessions.update(|sessions| {
            if !sessions.iter().any(|s| s.id == session_id_clone) {
                let mut s = session.clone();
                s.id = session_id_clone;
                sessions.push(s);
            }
        });
        chat_state.current_session_id.set(Some(session_id));
    }

    // WebSocket 连接：订阅 webchat 频道接收 Agent 回复
    let chat_state_for_effect = chat_state.clone();
    Effect::new(move |_| {
        let window = web_sys::window()?;
        let location = window.location();
        let protocol = location.protocol().ok()?;
        let host = location.host().ok()?;
        let ws_protocol = if protocol == "https:" { "wss" } else { "ws" };
        let ws_url = format!("{}://{}/ws", ws_protocol, host);

        let ws = web_sys::WebSocket::new(&ws_url).ok()?;
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let chat_state_clone = chat_state_for_effect.clone();
        let onmessage = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
            if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                let text_str = text.as_string().unwrap_or_default();
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text_str) {
                    if json.get("type").and_then(|v| v.as_str()) == Some("chat_message") {
                        if let Some(msg_json) = json.get("message") {
                            if let Ok(message) = serde_json::from_value::<ChatMessage>(msg_json.clone()) {
                                chat_state_clone.add_message(message);
                            }
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);
        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        let ws_for_open = ws.clone();
        let onopen = Closure::wrap(Box::new(move |_e: web_sys::Event| {
            let subscribe = serde_json::json!({
                "type": "subscribe",
                "channel": "webchat"
            });
            let _ = ws_for_open.send_with_str(&subscribe.to_string());
        }) as Box<dyn FnMut(_)>);
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        let chat_state_err = chat_state_for_effect.clone();
        let onerror = Closure::wrap(Box::new(move |_e: web_sys::Event| {
            chat_state_err.set_error(Some("WebSocket connection error".to_string()));
        }) as Box<dyn FnMut(_)>);
        ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onerror.forget();

        let onclose = Closure::wrap(Box::new(move |_e: web_sys::Event| {
            // 连接关闭，可选：自动重连逻辑
        }) as Box<dyn FnMut(_)>);
        ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
        onclose.forget();

        Some(())
    });

    // 发送消息处理
    let chat_state_for_send = chat_state.clone();
    let handle_send = move |content: String| {
        let session_id = chat_state_for_send.current_session_id.get();
        if session_id.is_none() {
            return;
        }
        let session_id = session_id.unwrap();

        // 本地添加用户消息
        let user_message = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            role: MessageRole::User,
            content: content.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            attachments: vec![],
            metadata: Default::default(),
            token_usage: None,
        };
        chat_state_for_send.add_message(user_message);
        chat_state_for_send.is_sending.set(true);
        chat_state_for_send.set_error(None);

        // 异步发送到后端
        let chat_state_send = chat_state_for_send.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let client = create_client();
            let service = create_webchat_service(client);
            match service.send_message(&session_id, &content).await {
                Ok(_) => {}
                Err(e) => {
                    chat_state_send.set_error(Some(format!("Failed to send: {}", e)));
                }
            }
            chat_state_send.is_sending.set(false);
        });
    };
    let on_submit: Box<dyn Fn(String)> = Box::new(handle_send);

    view! {
        <Title text="Chat - BeeBotOS" />
        <div class="webchat-page">
            <div class="webchat-container">
                <Show
                    when=move || ui_state.show_sessions_panel.get()
                    fallback=|| view! { <div class="sidebar-collapsed" /> }
                >
                    <SessionsSidebar />
                </Show>

                <main class="chat-main">
                    <ChatHeader />
                    {move || view! {
                        <MessageList
                            messages=chat_state.current_messages.get()
                            is_streaming=chat_state.is_streaming.get()
                            streaming_content=chat_state.streaming_content.get()
                        />
                    }}
                    <MessageInput
                        placeholder="Type a message... (use /btw for side question)".to_string()
                        disabled=chat_state.is_sending.get()
                        on_submit=on_submit
                    />
                    {move || {
                        if let Some(ref error) = chat_state.error.get() {
                            view! {
                                <div class="chat-error">{error.clone()}</div>
                            }.into_any()
                        } else {
                            view! { <div /> }.into_any()
                        }
                    }}
                </main>

                <Show
                    when=move || ui_state.show_side_panel.get()
                    fallback=|| view! { <div class="side-panel-collapsed" /> }
                >
                    <SideQuestionPanel />
                </Show>
            </div>
        </div>
    }
}

/// 会话侧边栏
#[component]
fn SessionsSidebar() -> impl IntoView {
    let ui_state = use_chat_ui_state();
    let chat_state = use_webchat_state();

    let on_new_chat = move |_| {
        let session_id = uuid::Uuid::new_v4().to_string();
        let _ = LocalStorage::set("beebotos_webchat_session_id", &session_id);
        let session = ChatSession::new("New Chat");
        let mut s = session;
        s.id = session_id.clone();
        chat_state.sessions.update(|sessions| sessions.push(s.clone()));
        chat_state.current_session_id.set(Some(session_id));
        chat_state.current_messages.set(Vec::new());
    };

    view! {
        <aside class="sessions-sidebar">
            <div class="sidebar-header">
                <h3>"Sessions"</h3>
                <button class="btn btn-icon" on:click=move |_| ui_state.toggle_sessions_panel()>
                    "◀"
                </button>
            </div>

            <div class="sidebar-actions">
                <button class="btn btn-primary btn-block" on:click=on_new_chat>
                    "+ New Chat"
                </button>
            </div>

            <div class="search-box">
                <input
                    type="text"
                    placeholder="Search sessions..."
                />
            </div>

            <SessionList />
        </aside>
    }
}

/// 会话列表
#[component]
fn SessionList() -> impl IntoView {
    let chat_state = use_webchat_state();

    view! {
        <div class="session-list">
            <For
                each=move || chat_state.sessions.get()
                key=|session| session.id.clone()
                children=move |session| {
                    let chat_state_click = chat_state.clone();
                    let session_id = session.id.clone();
                    let is_active = chat_state.current_session_id.get() == Some(session.id.clone());
                    view! {
                        <div
                            class=move || {
                                if is_active {
                                    "session-item active"
                                } else {
                                    "session-item"
                                }
                            }
                            on:click=move |_| {
                                let _ = LocalStorage::set("beebotos_webchat_session_id", &session_id);
                                chat_state_click.current_session_id.set(Some(session_id.clone()));
                                chat_state_click.current_messages.set(Vec::new());
                            }
                        >
                            <div class="session-title">{session.title.clone()}</div>
                            <div class="session-meta">
                                {format!("{} messages", session.messages.len())}
                                <Show when=move || session.is_pinned>
                                    <span class="pin-icon">"📌"</span>
                                </Show>
                            </div>
                        </div>
                    }
                }
            />
        </div>
    }
}

/// 聊天头部
#[component]
fn ChatHeader() -> impl IntoView {
    let ui_state = use_chat_ui_state();

    view! {
        <header class="chat-header">
            <div class="header-left">
                <h2>"Chat Session"</h2>
            </div>

            <div class="header-actions">
                <button class="btn btn-icon" title="Usage" on:click={
                    let ui_state = ui_state.clone();
                    move |_| ui_state.toggle_usage_panel()
                }>
                    "📊"
                </button>
                <button class="btn btn-icon" title="Side Questions" on:click={
                    let ui_state = ui_state.clone();
                    move |_| ui_state.toggle_side_panel()
                }>
                    "💬"
                </button>
            </div>
        </header>
    }
}

/// 侧边提问面板
#[component]
fn SideQuestionPanel() -> impl IntoView {
    let ui_state = use_chat_ui_state();

    view! {
        <aside class="side-question-panel">
            <div class="panel-header">
                <h4>"Side Questions"</h4>
                <button class="btn btn-icon" on:click=move |_| ui_state.toggle_side_panel()>
                    "▶"
                </button>
            </div>

            <div class="side-questions-list">
                <p>"Side questions will appear here..."</p>
            </div>
        </aside>
    }
}
