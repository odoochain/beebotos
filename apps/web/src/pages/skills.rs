use crate::api::{SkillCategory, SkillInfo};
use crate::state::use_app_state;
use leptos::prelude::*;
use leptos::view;
use leptos_meta::*;

#[component]
pub fn SkillsPage() -> impl IntoView {
    let app_state = use_app_state();
    let search_query = RwSignal::new(String::new());
    let selected_category = RwSignal::new(None::<SkillCategory>);

    // Fetch skills - use LocalResource for CSR
    let skills = LocalResource::new({
        let app_state = app_state.clone();
        move || {
            let service = app_state.skill_service();
            let app_state = app_state.clone();
            async move {
                app_state.loading().skills.set(true);
                let result = service.list().await;
                app_state.loading().skills.set(false);
                result
            }
        }
    });

    view! {
        <Title text="Skills - BeeBotOS" />
        <div class="page skills-page">
            <div class="page-header">
                <div>
                    <h1>"Skill Marketplace"</h1>
                    <p class="page-description">"Browse and install community-built skills to extend your agents"</p>
                </div>
            </div>

            <section class="skills-controls">
                <div class="search-bar">
                    <input
                        type="text"
                        placeholder="Search skills..."
                        prop:value=search_query
                        on:input=move |e| search_query.set(event_target_value(&e))
                    />
                    <span class="search-icon">"🔍"</span>
                </div>

                <div class="category-filters">
                    <CategoryFilter
                        label="All"
                        is_active={
                            let selected = selected_category;
                            move || selected.get().is_none()
                        }
                        on_click={
                            let selected = selected_category;
                            move || selected.set(None)
                        }
                    />
                </div>
            </section>

            <Suspense fallback=|| view! { <SkillsLoading/> }>
                {move || {
                    Suspend::new(async move {
                        match skills.await {
                            Ok(data) => {
                                let filtered: Vec<_> = data.into_iter()
                                    .filter(|s| {
                                        let matches_search = search_query.with(|q| {
                                            q.is_empty() ||
                                            s.name.to_lowercase().contains(&q.to_lowercase()) ||
                                            s.description.to_lowercase().contains(&q.to_lowercase())
                                        });
                                        let matches_category = selected_category.with(|c| {
                                            c.as_ref().map(|cat| std::mem::discriminant(cat) == std::mem::discriminant(&s.category)).unwrap_or(true)
                                        });
                                        matches_search && matches_category
                                    })
                                    .collect();

                                if filtered.is_empty() {
                                    view! { <SkillsEmpty/> }.into_any()
                                } else {
                                    view! {
                                        <SkillsGrid skills=filtered/>
                                    }.into_any()
                                }
                            }
                            Err(e) => view! { <SkillsError message=e.to_string()/> }.into_any(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn CategoryFilter(
    #[prop(into)] label: String,
    is_active: impl Fn() -> bool + Clone + Send + Sync + 'static,
    on_click: impl Fn() + Clone + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <button
            class=move || format!("category-filter {}", if is_active() { "active" } else { "" })
            on:click=move |_| on_click()
        >
            {label}
        </button>
    }
}

#[component]
fn SkillsGrid(skills: Vec<SkillInfo>) -> impl IntoView {
    view! {
        <div class="skills-grid">
            {skills.into_iter().map(|skill| {
                view! {
                    <SkillCard skill=skill/>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
fn SkillCard(#[prop(into)] skill: SkillInfo) -> impl IntoView {
    let category_icon = match skill.category {
        SkillCategory::Trading => "📈",
        SkillCategory::Data => "📊",
        SkillCategory::Social => "💬",
        SkillCategory::Automation => "⚙️",
        SkillCategory::Analysis => "🔍",
        SkillCategory::Other => "📦",
    };

    let is_installed = skill.installed;

    view! {
        <div class="card skill-card">
            <div class="skill-header">
                <div class="skill-icon">{category_icon}</div>
                <div class="skill-meta">
                    <h3>{skill.name}</h3>
                    <div class="skill-stats">
                        <span class="skill-version">{format!("v{}", skill.version)}</span>
                        <span class="skill-rating">
                            "⭐ "{format!("{:.1}", skill.rating)}
                        </span>
                        <span class="skill-downloads">
                            "⬇️ "{format!("{}", skill.downloads)}
                        </span>
                    </div>
                </div>
                {move || if is_installed {
                    view! {
                        <span class="installed-badge">"✓ Installed"</span>
                    }.into_any()
                } else {
                    view! { <></> }.into_any()
                }}
            </div>

            <p class="skill-description">{skill.description.clone()}</p>

            <div class="skill-footer">
                <span class="skill-author">{format!("by {}", skill.author)}</span>
                <div class="skill-actions">
                    <button class="btn btn-secondary btn-sm">"Details"</button>
                    {move || if is_installed {
                        view! {
                            <button class="btn btn-danger btn-sm">"Uninstall"</button>
                        }.into_any()
                    } else {
                        view! {
                            <button
                                class="btn btn-primary btn-sm"
                                on:click=move |_| {
                                    leptos::task::spawn_local(async move {
                                        gloo_timers::future::TimeoutFuture::new(500).await;
                                    });
                                }
                            >
                                "Install"
                            </button>
                        }.into_any()
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
fn SkillsLoading() -> impl IntoView {
    view! {
        <div class="skills-grid">
            <div class="card skill-card skeleton">
                <div class="skeleton-header"></div>
                <div class="skeleton-line"></div>
                <div class="skeleton-line"></div>
            </div>
            <div class="card skill-card skeleton">
                <div class="skeleton-header"></div>
                <div class="skeleton-line"></div>
                <div class="skeleton-line"></div>
            </div>
            <div class="card skill-card skeleton">
                <div class="skeleton-header"></div>
                <div class="skeleton-line"></div>
                <div class="skeleton-line"></div>
            </div>
            <div class="card skill-card skeleton">
                <div class="skeleton-header"></div>
                <div class="skeleton-line"></div>
                <div class="skeleton-line"></div>
            </div>
        </div>
    }
}

#[component]
fn SkillsEmpty() -> impl IntoView {
    view! {
        <div class="empty-state">
            <div class="empty-icon">"📦"</div>
            <h3>"No skills found"</h3>
            <p>"Try adjusting your search or filters"</p>
        </div>
    }
}

#[component]
fn SkillsError(#[prop(into)] message: String) -> impl IntoView {
    view! {
        <div class="error-state">
            <div class="error-icon">"⚠️"</div>
            <h3>"Failed to load skills"</h3>
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
