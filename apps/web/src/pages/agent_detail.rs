use crate::api::{AgentInfo, AgentStatus};
use crate::state::use_app_state;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::view;
use leptos_meta::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

#[component]
pub fn AgentDetail() -> impl IntoView {
    let params = use_params_map();
    let app_state = use_app_state();
    let app_state_stored = StoredValue::new(app_state);

    let agent_id = move || params.with(|p| p.get("id").unwrap_or_default());

    let agent_data: RwSignal<Option<AgentInfo>> = RwSignal::new(None);
    let agent_error: RwSignal<Option<String>> = RwSignal::new(None);

    // Fetch agent details
    let fetch_agent = move || {
        let app_state = app_state_stored.get_value();
        let id = agent_id();
        spawn_local(async move {
            if id.is_empty() {
                agent_error.set(Some("Agent ID is required".to_string()));
            } else {
                let service = app_state.agent_service();
                match service.get(&id).await {
                    Ok(data) => agent_data.set(Some(data)),
                    Err(e) => agent_error.set(Some(e.to_string())),
                }
            }
        });
    };

    // Store for reuse
    let fetch_agent_stored = StoredValue::new(fetch_agent);

    // Initial fetch
    fetch_agent_stored.get_value()();

    view! {
        <Title text="Agent Details - BeeBotOS" />
        <div class="page agent-detail-page">
            {move || {
                if let Some(error) = agent_error.get() {
                    view! { <AgentDetailError message=error/> }.into_any()
                } else if let Some(agent) = agent_data.get() {
                    let agent_id_start = agent.id.clone();
                    let agent_id_stop = agent.id.clone();
                    view! {
                        <AgentDetailView
                            agent=agent
                            on_start={
                                move || {
                                    let app_state = app_state_stored.get_value();
                                    let id = agent_id_start.clone();
                                    spawn_local(async move {
                                        let service = app_state.agent_service();
                                        let _ = service.start(&id).await;
                                        fetch_agent_stored.get_value()();
                                    });
                                }
                            }
                            on_stop={
                                move || {
                                    let app_state = app_state_stored.get_value();
                                    let id = agent_id_stop.clone();
                                    spawn_local(async move {
                                        let service = app_state.agent_service();
                                        let _ = service.stop(&id).await;
                                        fetch_agent_stored.get_value()();
                                    });
                                }
                            }
                        />
                    }.into_any()
                } else {
                    view! { <AgentDetailLoading/> }.into_any()
                }
            }}
        </div>
    }
}

#[component]
fn AgentDetailView(
    #[prop(into)] agent: AgentInfo,
    on_start: impl Fn() + Clone + 'static,
    on_stop: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let status_class = match agent.status {
        AgentStatus::Running => "status-running",
        AgentStatus::Stopped | AgentStatus::Idle => "status-idle",
        AgentStatus::Error => "status-error",
        AgentStatus::Pending => "status-pending",
    };

    let is_running = agent.status == AgentStatus::Running;

    // Use Rc<RefCell> for callbacks (not Send/Sync but works in single-threaded WASM)
    let on_start = std::rc::Rc::new(std::cell::RefCell::new(on_start));
    let on_stop = std::rc::Rc::new(std::cell::RefCell::new(on_stop));

    view! {
        <div class="agent-detail-header">
            <div class="breadcrumb">
                <A href="/agents">"Agents"</A>
                <span>"/"</span>
                <span>{agent.name.clone()}</span>
            </div>
            <div class="agent-title-section">
                <div class="agent-title">
                    <h1>{agent.name}</h1>
                    <span class=format!("status-badge {}", status_class)>
                        {format!("{:?}", agent.status)}
                    </span>
                </div>
                <div class="agent-actions">
                    {if is_running {
                        let on_stop = on_stop.clone();
                        view! {
                            <button class="btn btn-warning" on:click=move |_| on_stop.borrow_mut()()>
                                "⏹ Stop"
                            </button>
                        }.into_any()
                    } else {
                        let on_start = on_start.clone();
                        view! {
                            <button class="btn btn-success" on:click=move |_| on_start.borrow_mut()()>
                                "▶ Start"
                            </button>
                        }.into_any()
                    }}
                    <button class="btn btn-secondary">"Edit"</button>
                    <button class="btn btn-danger">"Delete"</button>
                </div>
            </div>
        </div>

        <div class="agent-detail-grid">
            <div class="agent-detail-main">
                <section class="card">
                    <h2>"Overview"</h2>
                    {agent.description.clone().map(|desc| view! {
                        <p class="agent-description">{desc}</p>
                    })}
                    <div class="agent-info-grid">
                        <InfoItem label="Agent ID" value=agent.id.clone() />
                        <InfoItem
                            label="Created"
                            value=agent.created_at.clone().unwrap_or_else(|| "Unknown".to_string())
                        />
                        <InfoItem
                            label="Updated"
                            value=agent.updated_at.clone().unwrap_or_else(|| "Unknown".to_string())
                        />
                        <InfoItem
                            label="Tasks Completed"
                            value=agent.task_count.map(|t| t.to_string()).unwrap_or_else(|| "0".to_string())
                        />
                        <InfoItem
                            label="Uptime"
                            value=agent.uptime_percent.map(|u| format!("{:.1}%", u)).unwrap_or_else(|| "N/A".to_string())
                        />
                    </div>
                </section>

                <section class="card">
                    <h2>"Capabilities"</h2>
                    <div class="capabilities-list">
                        {if agent.capabilities.is_empty() {
                            view! { <p class="text-muted">"No capabilities configured"</p> }.into_any()
                        } else {
                            view! {
                                <div class="capabilities-list">
                                    {agent.capabilities.iter().map(|cap| view! {
                                        <div class="capability-item">
                                            <span class="capability-icon">"⚡"</span>
                                            <span class="capability-name">{cap.clone()}</span>
                                        </div>
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }}
                    </div>
                </section>

                <section class="card">
                    <h2>"Recent Activity"</h2>
                    <AgentActivityLog />
                </section>
            </div>

            <div class="agent-detail-sidebar">
                <section class="card">
                    <h3>"Quick Stats"</h3>
                    <div class="quick-stats">
                        <StatItem label="Status" value=format!("{:?}", agent.status) />
                        <StatItem label="Tasks" value=agent.task_count.unwrap_or(0).to_string() />
                        <StatItem
                            label="Uptime"
                            value=agent.uptime_percent.map(|u| format!("{:.1}%", u)).unwrap_or_else(|| "N/A".to_string())
                        />
                    </div>
                </section>

                <section class="card">
                    <h3>"Actions"</h3>
                    <div class="action-list">
                        <button class="btn btn-secondary btn-block">"View Logs"</button>
                        <button class="btn btn-secondary btn-block">"Configure"</button>
                        <button class="btn btn-secondary btn-block">"Clone Agent"</button>
                        <button class="btn btn-secondary btn-block">"Export Config"</button>
                    </div>
                </section>
            </div>
        </div>
    }
}

#[component]
fn InfoItem(#[prop(into)] label: String, #[prop(into)] value: String) -> impl IntoView {
    view! {
        <div class="info-item">
            <span class="info-label">{label}</span>
            <span class="info-value">{value}</span>
        </div>
    }
}

#[component]
fn StatItem(#[prop(into)] label: String, #[prop(into)] value: String) -> impl IntoView {
    view! {
        <div class="stat-item">
            <span class="stat-label">{label}</span>
            <span class="stat-value">{value}</span>
        </div>
    }
}

#[component]
fn AgentDetailLoading() -> impl IntoView {
    view! {
        <div class="agent-detail-loading">
            <div class="skeleton-header"></div>
            <div class="skeleton-grid">
                <div class="skeleton-card">
                    <div class="skeleton-line"></div>
                    <div class="skeleton-line"></div>
                </div>
                <div class="skeleton-card">
                    <div class="skeleton-line"></div>
                    <div class="skeleton-line"></div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn AgentDetailError(#[prop(into)] message: String) -> impl IntoView {
    view! {
        <div class="error-state">
            <div class="error-icon">"⚠️"</div>
            <h2>"Agent Not Found"</h2>
            <p>{message}</p>
            <A href="/agents" attr:class="btn btn-primary">
                "Back to Agents"
            </A>
        </div>
    }
}

#[component]
fn AgentActivityLog() -> impl IntoView {
    view! {
        <div class="activity-log">
            <div class="activity-item">
                <span class="activity-time">"2024-03-22 14:30"</span>
                <span class="activity-action">"Agent started"</span>
            </div>
            <div class="activity-item">
                <span class="activity-time">"2024-03-22 12:15"</span>
                <span class="activity-action">"Task completed: Data sync"</span>
            </div>
            <div class="activity-item">
                <span class="activity-time">"2024-03-22 10:00"</span>
                <span class="activity-action">"Agent initialized"</span>
            </div>
        </div>
    }
}
