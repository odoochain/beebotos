//! LLM Configuration & Monitoring Page
//!
//! Displays global LLM configuration and real-time metrics from Gateway.

use crate::api::{LlmConfigService, LlmGlobalConfig, LlmMetricsResponse, LlmHealthResponse};
use crate::components::InlineLoading;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::view;
use leptos_meta::*;

#[component]
pub fn LlmConfigPage() -> impl IntoView {
    let client = crate::api::create_client();
    let llm_service = LlmConfigService::new(client);
    let service_stored = StoredValue::new(llm_service);

    let config: RwSignal<Option<LlmGlobalConfig>> = RwSignal::new(None);
    let metrics: RwSignal<Option<LlmMetricsResponse>> = RwSignal::new(None);
    let health: RwSignal<Option<LlmHealthResponse>> = RwSignal::new(None);
    let error: RwSignal<Option<String>> = RwSignal::new(None);
    let loading = RwSignal::new(true);

    let fetch_all = move || {
        let service = service_stored.get_value();
        loading.set(true);
        error.set(None);
        spawn_local(async move {
            match service.get_config().await {
                Ok(c) => config.set(Some(c)),
                Err(e) => error.set(Some(format!("Config: {}", e))),
            }
            match service.get_metrics().await {
                Ok(m) => metrics.set(Some(m)),
                Err(e) => {
                    let msg = error.get().unwrap_or_default();
                    error.set(Some(format!("{} Metrics: {}", msg, e)));
                }
            }
            match service.get_health().await {
                Ok(h) => health.set(Some(h)),
                Err(e) => {
                    let msg = error.get().unwrap_or_default();
                    error.set(Some(format!("{} Health: {}", msg, e)));
                }
            }
            loading.set(false);
        });
    };

    let fetch_stored = StoredValue::new(fetch_all);

    // Initial fetch
    Effect::new(move |_| {
        fetch_stored.get_value()();
    });

    // Auto-refresh metrics every 10s
    Effect::new(move |_| {
        spawn_local(async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(10_000).await;
                let service = service_stored.get_value();
                if let Ok(m) = service.get_metrics().await {
                    metrics.set(Some(m));
                }
                if let Ok(h) = service.get_health().await {
                    health.set(Some(h));
                }
            }
        });
    });

    view! {
        <Title text="LLM Configuration - BeeBotOS" />
        <div class="page llm-config-page">
            <div class="page-header">
                <h1>"LLM Configuration"</h1>
                <p class="page-description">"Global LLM settings and real-time monitoring"</p>
            </div>

            {move || if loading.get() {
                view! { <InlineLoading /> }.into_any()
            } else if let Some(err) = error.get() {
                view! {
                    <div class="error-state">
                        <div class="error-icon">"⚠️"</div>
                        <p>{err}</p>
                        <button class="btn btn-primary" on:click=move |_| fetch_stored.get_value()()>
                            "Retry"
                        </button>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="llm-config-grid">
                        // Global Config Card
                        {config.get().map(|cfg| view! {
                            <section class="card llm-section">
                                <h2>"Global Configuration"</h2>
                                <div class="info-grid">
                                    <InfoRow label="Default Provider" value=cfg.default_provider />
                                    <InfoRow label="Max Tokens" value=cfg.max_tokens.to_string() />
                                    <InfoRow label="Request Timeout" value=format!("{}s", cfg.request_timeout) />
                                    <InfoRow label="Cost Optimization" value=if cfg.cost_optimization { "Enabled" } else { "Disabled" }.to_string() />
                                    <InfoRow label="Fallback Chain" value=cfg.fallback_chain.join(", ") />
                                </div>
                                <div class="form-group">
                                    <label>"System Prompt"</label>
                                    <textarea readonly class="system-prompt">{cfg.system_prompt}</textarea>
                                </div>
                            </section>
                        })}

                        // Provider Cards
                        {config.get().map(|cfg| view! {
                            <section class="card llm-section">
                                <h2>"Providers"</h2>
                                <div class="provider-cards">
                                    {cfg.providers.into_iter().map(|p| {
                                        let health_status = health.get()
                                            .and_then(|h| h.providers.iter().find(|ph| ph.name == p.name).cloned());
                                        view! {
                                            <div class="provider-card">
                                                <div class="provider-header">
                                                    <h3>{p.name.clone()}</h3>
                                                    {health_status.map(|h| view! {
                                                        <span class=format!("health-badge {}", if h.healthy { "healthy" } else { "unhealthy" })>
                                                            {if h.healthy { "● Healthy".to_string() } else { format!("● {} failures", h.consecutive_failures) }}
                                                        </span>
                                                    })}
                                                </div>
                                                <div class="info-grid">
                                                    <InfoRow label="Model" value=p.model />
                                                    <InfoRow label="Base URL" value=p.base_url />
                                                    <InfoRow label="API Key" value=p.api_key_masked />
                                                    <InfoRow label="Temperature" value=format!("{:.2}", p.temperature) />
                                                    <InfoRow label="Context Window" value=p.context_window.map(|c| c.to_string()).unwrap_or_else(|| "Default".to_string()) />
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </section>
                        })}

                        // Metrics Card
                        {metrics.get().map(|m| view! {
                            <section class="card llm-section">
                                <h2>"Real-time Metrics"</h2>
                                <p class="timestamp">{format!("Last updated: {}", m.timestamp)}</p>
                                <div class="metrics-grid">
                                    <MetricCard
                                        label="Total Requests"
                                        value=m.summary.total_requests.to_string()
                                        delta=Some(format!("{:.1}% success", m.summary.success_rate_percent))
                                    />
                                    <MetricCard
                                        label="Successful"
                                        value=m.summary.successful_requests.to_string()
                                        delta=None
                                    />
                                    <MetricCard
                                        label="Failed"
                                        value=m.summary.failed_requests.to_string()
                                        delta=None
                                    />
                                    <MetricCard
                                        label="Total Tokens"
                                        value=m.tokens.total_tokens.to_string()
                                        delta=Some(format!("{} in / {} out", m.tokens.input_tokens, m.tokens.output_tokens))
                                    />
                                </div>
                                <h3>"Latency"</h3>
                                <div class="latency-bars">
                                    <LatencyBar label="Avg" value=m.latency.average_ms max=1000.0 />
                                    <LatencyBar label="P50" value=m.latency.p50_ms max=1000.0 />
                                    <LatencyBar label="P95" value=m.latency.p95_ms max=1000.0 />
                                    <LatencyBar label="P99" value=m.latency.p99_ms max=1000.0 />
                                </div>
                            </section>
                        })}
                    </div>
                }.into_any()
            }}
        </div>
    }
}

#[component]
fn InfoRow(#[prop(into)] label: String, #[prop(into)] value: String) -> impl IntoView {
    view! {
        <div class="info-row">
            <span class="info-label">{label}</span>
            <span class="info-value">{value}</span>
        </div>
    }
}

#[component]
fn MetricCard(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    delta: Option<String>,
) -> impl IntoView {
    view! {
        <div class="metric-card">
            <div class="metric-value">{value}</div>
            <div class="metric-label">{label}</div>
            {delta.map(|d| view! { <div class="metric-delta">{d}</div> })}
        </div>
    }
}

#[component]
fn LatencyBar(
    #[prop(into)] label: String,
    value: f64,
    max: f64,
) -> impl IntoView {
    let pct = (value / max * 100.0).min(100.0);
    let color_class = if pct < 30.0 {
        "latency-good"
    } else if pct < 70.0 {
        "latency-warning"
    } else {
        "latency-danger"
    };

    view! {
        <div class="latency-bar">
            <span class="latency-label">{label}</span>
            <span class="latency-value">{format!("{:.0}ms", value)}</span>
            <div class="latency-track">
                <div class=format!("latency-fill {}", color_class) style=format!("width: {}%", pct)></div>
            </div>
        </div>
    }
}
