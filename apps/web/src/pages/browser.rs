//! 浏览器自动化页面
//!
//! 提供 Chrome DevTools 控制、批处理操作、沙箱管理等功能

use leptos::prelude::*;
use leptos::view;
use leptos_meta::Title;

use crate::state::{
    use_browser_state, use_browser_ui_state,
};

/// 浏览器自动化页面
#[component]
pub fn BrowserPage() -> impl IntoView {
    let browser_state = use_browser_state();
    let _ui_state = use_browser_ui_state();

    // Load profiles from API on mount
    let client = crate::api::create_client();
    let browser_service = crate::api::BrowserApiService::new(client);
    let service_stored = leptos::prelude::StoredValue::new(browser_service);

    leptos::task::spawn_local(async move {
        let service = service_stored.get_value();
        match service.list_profiles().await {
            Ok(profiles) => browser_state.profiles.set(profiles),
            Err(e) => browser_state.error.set(Some(crate::browser::BrowserError {
                error_type: crate::browser::BrowserErrorType::ConnectionLost,
                message: e.to_string(),
                current_url: None,
                screenshot_path: None,
                suggestions: vec![],
            })),
        }
        match service.get_status().await {
            Ok(status) => {
                browser_state.connection_status.set(
                    if status.active_instances > 0 { crate::browser::ConnectionStatus::Connected } else { crate::browser::ConnectionStatus::Disconnected }
                );
            }
            Err(_) => {}
        }
    });

    view! {
        <Title text="Browser Automation - BeeBotOS" />
        <div class="browser-page">
            <BrowserHeader />
            <div class="browser-container">
                <BrowserSidebar />
                <BrowserMainContent />
            </div>
        </div>
    }
}

/// 页面头部
#[component]
fn BrowserHeader() -> impl IntoView {
    view! {
        <header class="browser-header">
            <h1>"Browser Automation"</h1>
            <p class="browser-subtitle">
                "Chrome DevTools MCP Control - Compatible with OpenClaw V2026.3.13"
            </p>
        </header>
    }
}

/// 侧边栏
#[component]
fn BrowserSidebar() -> impl IntoView {
    view! {
        <aside class="browser-sidebar">
            <div class="sidebar-section">
                <h3>"Profiles"</h3>
                <button class="btn btn-primary btn-sm">
                    "+ Add Profile"
                </button>
                <ProfileList />
            </div>

            <div class="sidebar-section">
                <h3>"Sandboxes"</h3>
                <button class="btn btn-secondary btn-sm">
                    "+ Create Sandbox"
                </button>
                <SandboxList />
            </div>
        </aside>
    }
}

/// 配置列表
#[component]
fn ProfileList() -> impl IntoView {
    let state = use_browser_state();

    view! {
        <div class="profile-list">
            <For
                each=move || state.profiles.get()
                key=|profile| profile.id.clone()
                children=move |profile| {
                    let is_selected = state.selected_profile_id.get() == Some(profile.id.clone());
                    view! {
                        <div
                            class=move || {
                                if is_selected {
                                    "profile-item selected"
                                } else {
                                    "profile-item"
                                }
                            }
                            style=format!("border-left-color: {}", profile.color)
                        >
                            <div class="profile-name">{profile.name.clone()}</div>
                            <div class="profile-info">
                                {format!("Port: {}", profile.cdp_port)}
                            </div>
                        </div>
                    }
                }
            />
        </div>
    }
}

/// 沙箱列表
#[component]
fn SandboxList() -> impl IntoView {
    let state = use_browser_state();

    view! {
        <div class="sandbox-list">
            <For
                each=move || state.sandboxes.get()
                key=|sandbox| sandbox.id.clone()
                children=move |sandbox| {
                    view! {
                        <div
                            class="sandbox-item"
                            style=format!("border-left-color: {}", sandbox.color)
                        >
                            <div class="sandbox-name">{sandbox.name.clone()}</div>
                            <div class="sandbox-info">
                                {format!("Port: {}", sandbox.cdp_port)}
                            </div>
                        </div>
                    }
                }
            />
        </div>
    }
}

/// 主内容区
#[component]
fn BrowserMainContent() -> impl IntoView {
    view! {
        <main class="browser-main">
            <BrowserToolbar />
            <BrowserViewport />
            <BrowserDebugPanel />
        </main>
    }
}

/// 工具栏
#[component]
fn BrowserToolbar() -> impl IntoView {
    let state = use_browser_state();

    view! {
        <div class="browser-toolbar">
            <div class="toolbar-group">
                <button class="btn btn-icon" title="Toggle Profiles">
                    "📑"
                </button>
                <button class="btn btn-icon" title="Toggle Sandboxes">
                    "🔲"
                </button>
            </div>

            <div class="toolbar-group toolbar-url">
                <input
                    type="text"
                    class="url-input"
                    prop:value=move || state.current_url.get()
                    placeholder="Enter URL..."
                />
                <button class="btn btn-primary">"Go"</button>
            </div>

            <div class="toolbar-group">
                <button class="btn btn-icon" title="Toggle Debug Panel">
                    "🐛"
                </button>
                <button class="btn btn-icon" title="Fullscreen">
                    "⛶"
                </button>
            </div>
        </div>
    }
}

/// 视口区域
#[component]
fn BrowserViewport() -> impl IntoView {
    view! {
        <div class="browser-viewport">
            <div class="browser-placeholder">
                <p>"No browser connected"</p>
                <p>"Select a profile to connect"</p>
            </div>
        </div>
    }
}

/// 调试面板
#[component]
fn BrowserDebugPanel() -> impl IntoView {
    view! {
        <div class="browser-debug-panel">
            <div class="debug-header">
                <h4>"Debug Console"</h4>
                <button class="btn btn-sm">
                    "Clear"
                </button>
            </div>
            <div class="debug-logs">
                <p>"Debug logs will appear here..."</p>
            </div>
        </div>
    }
}
