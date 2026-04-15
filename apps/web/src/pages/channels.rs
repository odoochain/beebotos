//! Channels Management Page
//!
//! Reference: /data/copaw-style-web channel management design

use crate::api::{ChannelService, ChannelInfo, ChannelStatus, WeChatQrResponse, QrStatusResponse};
use crate::components::InlineLoading;
use crate::i18n::I18nContext;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::view;

/// Available channel types with their metadata
const CHANNEL_TYPES: &[(&str, &str, &str, &str)] = &[
    ("wechat", "微信", "WeChat", "💬"),
    ("dingtalk", "钉钉", "DingTalk", "💼"),
    ("feishu", "飞书", "Lark", "🚀"),
    ("qq", "QQ", "Tencent QQ", "🐧"),
    ("discord", "Discord", "Discord Bot", "🎮"),
    ("telegram", "Telegram", "Telegram Bot", "✈️"),
];

fn get_demo_channels() -> Vec<ChannelInfo> {
    CHANNEL_TYPES.iter().map(|(id, name, desc, icon)| ChannelInfo {
        id: id.to_string(),
        name: name.to_string(),
        description: desc.to_string(),
        icon: icon.to_string(),
        enabled: id == &"wechat",
        status: if id == &"wechat" {
            ChannelStatus::Connected
        } else {
            ChannelStatus::Disabled
        },
        config: None,
        last_error: None,
        created_at: None,
        updated_at: None,
    }).collect::<Vec<_>>()
}

#[component]
pub fn ChannelsPage() -> impl IntoView {
    let i18n = use_context::<I18nContext>().expect("i18n context not found");
    let i18n_stored = StoredValue::new(i18n);

    // Channel service
    let client = crate::api::create_client();
    let channel_service = ChannelService::new(client);
    let service_stored = StoredValue::new(channel_service);

    // Channels resource
    let channels = LocalResource::new(move || {
        let service = service_stored.get_value();
        async move {
            service.list().await.unwrap_or_else(|_| get_demo_channels())
        }
    });

    // Selected channel for configuration
    let (selected_channel, set_selected_channel) = signal::<Option<ChannelInfo>>(None);

    // QR code state for WeChat
    let (qr_code, set_qr_code) = signal::<Option<WeChatQrResponse>>(None);
    let (qr_loading, set_qr_loading) = signal(false);
    let (qr_error, set_qr_error) = signal::<Option<String>>(None);
    let (qr_status, set_qr_status) = signal::<Option<QrStatusResponse>>(None);
    let (qr_polling, set_qr_polling) = signal(false);

    // Config panel open state
    let (config_panel_open, set_config_panel_open) = signal(false);

    view! {
        <div class="channels-page">
            // Page Header
            <div class="page-header">
                <h2>{move || i18n_stored.get_value().t("channels-title")}</h2>
                <p>{move || i18n_stored.get_value().t("channels-subtitle")}</p>
            </div>

            // Channels Grid
            <Suspense fallback=|| view! { <InlineLoading /> }>
                {move || {
                    channels.get().map(|channel_list| {
                        view! {
                            <div class="channels-grid">
                                {channel_list.into_iter().map(|channel| {
                                    let channel_for_click = channel.clone();
                                    let is_enabled = channel.enabled;
                                    let status_class = match channel.status {
                                        ChannelStatus::Connected => "status-active",
                                        ChannelStatus::Error => "status-error",
                                        _ => "",
                                    };

                                    view! {
                                        <div
                                            class="channel-card"
                                            on:click=move |_| {
                                                set_selected_channel.set(Some(channel_for_click.clone()));
                                                set_config_panel_open.set(true);
                                            }
                                        >
                                            <div class={format!("channel-icon {}", channel.id)}>
                                                {channel.icon.clone()}
                                            </div>
                                            <div class="channel-info">
                                                <h3>{channel.name.clone()}</h3>
                                                <p>{channel.description.clone()}</p>
                                            </div>
                                            <div class="channel-status">
                                                <span class={format!("status-badge {}", status_class)}>
                                                    {if is_enabled {
                                                        i18n_stored.get_value().t("status-enabled")
                                                    } else {
                                                        i18n_stored.get_value().t("status-disabled")
                                                    }}
                                                </span>
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }
                    })
                }}
            </Suspense>

            // Configuration Panel (Slide-out)
            {move || {
                if config_panel_open.get() {
                    selected_channel.get().map(|channel| {
                        view! {
                            <>
                                // Overlay
                                <div
                                    class="overlay show"
                                    on:click=move |_| {
                                        set_qr_polling.set(false);
                                        set_config_panel_open.set(false);
                                    }
                                />
                                // Config Panel
                                <div class="config-panel open">
                                    <div class="config-panel-header">
                                        <h3>{format!("{} {}", channel.icon, channel.name)}</h3>
                                        <button
                                            class="close-btn"
                                            on:click=move |_| {
                                                set_qr_polling.set(false);
                                                set_config_panel_open.set(false);
                                            }
                                        >
                                            "✕"
                                        </button>
                                    </div>
                                    <div class="config-panel-body">
                                        // Channel Status Section
                                        <div class="config-section">
                                            <h4>{i18n_stored.get_value().t("channel-status")}</h4>
                                            <div class="status-display">
                                                <span class={format!("status-dot {}", match channel.status {
                                                    ChannelStatus::Connected => "connected",
                                                    ChannelStatus::Error => "error",
                                                    _ => "disconnected",
                                                })} />
                                                <span>{format!("{:?}", channel.status)}</span>
                                            </div>
                                            {channel.last_error.clone().map(|err| view! {
                                                <div class="error-message">{err}</div>
                                            })}
                                        </div>

                                        // WeChat QR Code Section
                                        {if channel.id == "wechat" {
                                            Some(view! {
                                                <div class="config-section">
                                                    <h4>{i18n_stored.get_value().t("wechat-login")}</h4>
                                                    <p class="config-hint">
                                                        {i18n_stored.get_value().t("wechat-login-hint")}
                                                    </p>

                                                    {move || if qr_loading.get() {
                                                        view! { <InlineLoading /> }.into_any()
                                                    } else if let Some(error) = qr_error.get() {
                                                        view! {
                                                            <div class="error-message">
                                                                {error}
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        qr_code.get().map(|qr| view! {
                                                            <div class="qr-code-container">
                                                                {qr.qrcode_img_content.map(|url| view! {
                                                                    <div class="qr-link-box">
                                                                        <a href={url.clone()} target="_blank" rel="noopener" class="qr-link">
                                                                            <span class="qr-link-icon">[扫码]</span>
                                                                            <span>"点击打开微信扫码页面"</span>
                                                                        </a>
                                                                        <p class="qr-hint">"请使用微信扫描页面中的二维码"</p>
                                                                    </div>
                                                                })}
                                                                <p class="qr-text">{format!("二维码: {}", qr.qrcode.clone())}</p>
                                                                <p class="qr-expiry">
                                                                    {i18n_stored.get_value().t("qr-expires-in")}
                                                                    {format!(" {}s", qr.expires_in)}
                                                                </p>
                                                                {move || qr_status.get().map(|status| {
                                                                    let (icon, text, class) = match status.status.as_str() {
                                                                        "confirmed" => ("✅", "扫码成功，登录完成", "status-success"),
                                                                        "scanned" => ("📱", "已扫码，等待确认", "status-pending"),
                                                                        "expired" => ("❌", "二维码已过期，请重新获取", "status-error"),
                                                                        _ => ("⏳", "等待扫码...", "status-pending"),
                                                                    };
                                                                    view! {
                                                                        <p class={format!("qr-status {}", class)}>
                                                                            {icon} " " {text}
                                                                        </p>
                                                                    }
                                                                })}
                                                            </div>
                                                        }).into_any()
                                                    }}

                                                    <button
                                                        class="btn-primary btn-block"
                                                        on:click=move |_| {
                                                            set_qr_loading.set(true);
                                                            set_qr_polling.set(false);
                                                            set_qr_status.set(None);
                                                            let service = service_stored.get_value();
                                                            spawn_local(async move {
                                                                match service.get_wechat_qr().await {
                                                                    Ok(qr) => {
                                                                        set_qr_code.set(Some(qr.clone()));
                                                                        set_qr_error.set(None);
                                                                        set_qr_polling.set(true);
                                                                        // Start polling QR status
                                                                        let poll_service = service_stored.get_value();
                                                                        spawn_local(async move {
                                                                            loop {
                                                                                gloo_timers::future::TimeoutFuture::new(2000).await;
                                                                                if !qr_polling.get() {
                                                                                    break;
                                                                                }
                                                                                match poll_service.check_wechat_qr(&qr.qrcode).await {
                                                                                    Ok(status) => {
                                                                                        let should_stop = status.status == "confirmed" || status.status == "expired";
                                                                                        set_qr_status.set(Some(status));
                                                                                        if should_stop {
                                                                                            set_qr_polling.set(false);
                                                                                            break;
                                                                                        }
                                                                                    }
                                                                                    Err(e) => {
                                                                                        set_qr_error.set(Some(format!("轮询二维码状态失败: {:?}", e)));
                                                                                        set_qr_polling.set(false);
                                                                                        break;
                                                                                    }
                                                                                }
                                                                            }
                                                                        });
                                                                    }
                                                                    Err(e) => {
                                                                        set_qr_error.set(Some(format!("获取二维码失败: {:?}", e)));
                                                                    }
                                                                }
                                                                set_qr_loading.set(false);
                                                            });
                                                        }
                                                    >
                                                        {if qr_code.get().is_some() {
                                                            i18n_stored.get_value().t("action-refresh-qr")
                                                        } else {
                                                            i18n_stored.get_value().t("action-get-qr")
                                                        }}
                                                    </button>
                                                </div>
                                            })
                                        } else {
                                            None
                                        }}

                                        // Configuration Form
                                        <div class="config-section">
                                            <h4>{i18n_stored.get_value().t("channel-config")}</h4>
                                            <div class="form-group">
                                                <label>{i18n_stored.get_value().t("config-base-url")}</label>
                                                <input
                                                    type="text"
                                                    placeholder="https://ilinkai.weixin.qq.com"
                                                    value={channel.config.as_ref().and_then(|c| c.base_url.clone()).unwrap_or_default()}
                                                />
                                            </div>
                                            <div class="form-group">
                                                <label>{i18n_stored.get_value().t("config-bot-token")}</label>
                                                <input
                                                    type="password"
                                                    placeholder="••••••••"
                                                    value={channel.config.as_ref().and_then(|c| c.bot_token.clone()).unwrap_or_default()}
                                                />
                                            </div>
                                            <div class="form-group">
                                                <label class="checkbox-label">
                                                    <input
                                                        type="checkbox"
                                                        checked={channel.config.as_ref().and_then(|c| c.auto_reconnect).unwrap_or(true)}
                                                    />
                                                    <span>{i18n_stored.get_value().t("config-auto-reconnect")}</span>
                                                </label>
                                            </div>
                                        </div>

                                        // Actions
                                        <div class="config-actions">
                                            <button
                                                class="btn-secondary"
                                                on:click=move |_| {
                                                    // Test connection
                                                }
                                            >
                                                {i18n_stored.get_value().t("action-test")}
                                            </button>
                                            <button
                                                class="btn-primary"
                                                on:click=move |_| {
                                                    set_config_panel_open.set(false);
                                                }
                                            >
                                                {i18n_stored.get_value().t("action-save")}
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            </>
                        }
                    })
                } else {
                    None
                }
            }}
        </div>
    }
}
