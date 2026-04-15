use crate::api::{AssetInfo, TransactionInfo, TransactionStatus, TransactionType, TreasuryInfo};
use crate::state::use_app_state;
use leptos::prelude::*;
use leptos::view;
use leptos_meta::*;
use leptos_router::components::A;

/// Format a number with thousand separators
fn format_with_commas(num: impl ToString, suffix: &str) -> String {
    let num_str = num.to_string();
    let parts: Vec<&str> = num_str.split('.').collect();
    let int_part = parts[0];
    let frac_part = if parts.len() > 1 {
        Some(parts[1])
    } else {
        None
    };

    let mut result = String::new();
    let mut count = 0;

    for c in int_part.chars().rev() {
        if count > 0 && count % 3 == 0 {
            result.push(',');
        }
        result.push(c);
        count += 1;
    }

    let mut formatted = result.chars().rev().collect::<String>();
    if let Some(frac) = frac_part {
        formatted.push('.');
        formatted.push_str(frac);
    }

    if !suffix.is_empty() {
        formatted.push(' ');
        formatted.push_str(suffix);
    }

    formatted
}

/// Format a float with thousand separators and 2 decimal places
fn format_usd(value: f64) -> String {
    if value <= 0.0 {
        return "-".to_string();
    }
    let formatted = format!("{:.2}", value);
    let parts: Vec<&str> = formatted.split('.').collect();
    let int_part = parts[0];
    let frac_part = parts[1];

    let mut result = String::new();
    let mut count = 0;

    for c in int_part.chars().rev() {
        if count > 0 && count % 3 == 0 {
            result.push(',');
        }
        result.push(c);
        count += 1;
    }

    let mut formatted = result.chars().rev().collect::<String>();
    formatted.push('.');
    formatted.push_str(frac_part);

    formatted
}

#[component]
pub fn TreasuryPage() -> impl IntoView {
    let app_state = use_app_state();

    // Fetch treasury data - use LocalResource for CSR
    let treasury = LocalResource::new(move || {
        let service = app_state.treasury_service();
        let loading = app_state.loading();
        async move {
            loading.treasury.set(true);
            let result = service.get_info().await;
            loading.treasury.set(false);
            result
        }
    });

    view! {
        <Title text="Treasury - BeeBotOS" />
        <div class="page treasury-page">
            <div class="page-header">
                <div class="breadcrumb-nav">
                    <A href="/dao">"DAO"</A>
                    <span>"/"</span>
                    <span>"Treasury"</span>
                </div>
                <h1>"DAO Treasury"</h1>
                <p class="page-description">"Manage community funds with transparent, on-chain governance"</p>
            </div>

            <Suspense fallback=|| view! { <TreasuryLoading/> }>
                {move || {
                    Suspend::new(async move {
                        match treasury.await {
                            Ok(data) => view! { <TreasuryView data=data/> }.into_any(),
                            Err(e) => view! { <TreasuryError message=e.to_string()/> }.into_any(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn TreasuryView(data: TreasuryInfo) -> impl IntoView {
    view! {
        <div class="treasury-content">
            <section class="treasury-overview">
                <div class="total-balance-card">
                    <div class="balance-header">
                        <span class="balance-label">"Total Treasury Balance"</span>
                        <span class="live-indicator">"● Live"</span>
                    </div>
                    <div class="balance-value">
                        {format_with_commas(data.total_balance, &data.token_symbol)}
                    </div>
                    <div class="balance-actions">
                        <button class="btn btn-primary">"Deposit"</button>
                        <button class="btn btn-secondary">"Withdraw"</button>
                        <button class="btn btn-secondary">"Transfer"</button>
                    </div>
                </div>
            </section>

            <div class="treasury-grid">
                <section class="card assets-section">
                    <div class="section-header">
                        <h2>"Assets"</h2>
                        <span class="asset-count">{format!("{} tokens", data.assets.len())}</span>
                    </div>

                    {move || if data.assets.is_empty() {
                        view! { <AssetsEmpty/> }.into_any()
                    } else {
                        view! {
                            <div class="assets-list">
                                {data.assets.clone().into_iter().map(|asset| view! {
                                    <AssetRow asset=asset/>
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }}
                </section>

                <section class="card transactions-section">
                    <div class="section-header">
                        <h2>"Recent Transactions"</h2>
                        <A href="/dao/treasury/transactions" attr:class="btn btn-text">
                            "View All →"
                        </A>
                    </div>

                    {move || if data.recent_transactions.is_empty() {
                        view! { <TransactionsEmpty/> }.into_any()
                    } else {
                        view! {
                            <div class="transactions-list">
                                {data.recent_transactions.clone().into_iter().map(|tx| view! {
                                    <TransactionRow tx=tx/>
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }}
                </section>
            </div>

            <section class="card treasury-info">
                <h3>"About the Treasury"</h3>
                <div class="info-grid">
                    <div class="info-item">
                        <span class="info-icon">"🔒"</span>
                        <div>
                            <h4>"Multi-Sig Protected"</h4>
                            <p>"All withdrawals require multiple signatures from DAO council members"</p>
                        </div>
                    </div>
                    <div class="info-item">
                        <span class="info-icon">"📊"</span>
                        <div>
                            <h4>"Transparent"</h4>
                            <p>"All transactions are recorded on-chain and publicly verifiable"</p>
                        </div>
                    </div>
                    <div class="info-item">
                        <span class="info-icon">"⚡"</span>
                        <div>
                            <h4>"Governance Controlled"</h4>
                            <p>"Major allocations require community vote through DAO proposals"</p>
                        </div>
                    </div>
                </div>
            </section>
        </div>
    }
}

#[component]
fn AssetRow(#[prop(into)] asset: AssetInfo) -> impl IntoView {
    view! {
        <div class="asset-row">
            <div class="asset-info">
                <div class="asset-token">{asset.token.clone()}</div>
                <div class="asset-balance">{format_with_commas(&asset.balance, "")}</div>
            </div>
            <div class="asset-value">
                {if asset.value_usd > 0.0 {
                    format!("${}", format_usd(asset.value_usd))
                } else {
                    "-".to_string()
                }}
            </div>
        </div>
    }
}

#[component]
fn TransactionRow(#[prop(into)] tx: TransactionInfo) -> impl IntoView {
    let status_class = match tx.status {
        TransactionStatus::Completed => "status-completed",
        TransactionStatus::Pending => "status-pending",
        TransactionStatus::Failed => "status-failed",
    };

    let type_icon = match tx.tx_type {
        TransactionType::Deposit => "⬇️",
        TransactionType::Withdrawal => "⬆️",
        TransactionType::Transfer => "↔️",
        TransactionType::Swap => "🔄",
    };

    view! {
        <div class="transaction-row">
            <div class="transaction-icon">{type_icon}</div>
            <div class="transaction-details">
                <div class="transaction-type">{format!("{:?}", tx.tx_type)}</div>
                <div class="transaction-meta">
                    <span class="transaction-time">{tx.timestamp}</span>
                    <span class=format!("transaction-status {}", status_class)>
                        {format!("{:?}", tx.status)}
                    </span>
                </div>
            </div>
            <div class="transaction-amount">
                {format!("{:+} {}", tx.amount, tx.token)}
            </div>
        </div>
    }
}

#[component]
fn AssetsEmpty() -> impl IntoView {
    view! {
        <div class="empty-state-small">
            <p class="text-muted">"No assets in treasury"</p>
            <button class="btn btn-primary btn-sm">"Make First Deposit"</button>
        </div>
    }
}

#[component]
fn TransactionsEmpty() -> impl IntoView {
    view! {
        <div class="empty-state-small">
            <p class="text-muted">"No recent transactions"</p>
        </div>
    }
}

#[component]
fn TreasuryLoading() -> impl IntoView {
    view! {
        <div class="treasury-skeleton">
            <div class="total-balance-card skeleton">
                <div class="skeleton-label"></div>
                <div class="skeleton-value"></div>
            </div>
            <div class="treasury-grid">
                <div class="card skeleton">
                    <div class="skeleton-header"></div>
                    <div class="skeleton-line"></div>
                    <div class="skeleton-line"></div>
                </div>
                <div class="card skeleton">
                    <div class="skeleton-header"></div>
                    <div class="skeleton-line"></div>
                    <div class="skeleton-line"></div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn TreasuryError(#[prop(into)] message: String) -> impl IntoView {
    view! {
        <div class="error-state">
            <div class="error-icon">"⚠️"</div>
            <h3>"Failed to load treasury"</h3>
            <p>{message}</p>
            <button
                class="btn btn-primary"
                on:click=move |_| { let _ = window().location().reload(); }
            >
                "Retry"
            </button>
        </div>
    }
}
