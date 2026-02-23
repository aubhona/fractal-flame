use crate::katex;
use gloo_net::http::Request;
use serde::Deserialize;
use yew::prelude::*;

const API_URL: &str = "http://localhost:4245/api/variations";

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct VariationDto {
    pub id: String,
    pub name: String,
    pub formula_latex: String,
}

#[derive(Clone, Debug, Deserialize)]
struct VariationsResponse {
    variations: Vec<VariationDto>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AppState {
    variations: Vec<VariationDto>,
    selected: Vec<String>,
    loading: bool,
    error: Option<String>,
}

#[function_component(App)]
pub fn app() -> Html {
    let state = use_state(AppState::default);
    let state_clone = state.clone();

    use_effect_with((), move |_| {
        let state = state_clone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            state.set(AppState {
                loading: true,
                ..(*state).clone()
            });

            match Request::get(API_URL).send().await {
                Ok(resp) => {
                    if resp.ok() {
                        match resp.json::<VariationsResponse>().await {
                            Ok(data) => {
                                state.set(AppState {
                                    variations: data.variations,
                                    loading: false,
                                    error: None,
                                    ..(*state).clone()
                                });
                            }
                            Err(e) => {
                                state.set(AppState {
                                    loading: false,
                                    error: Some(format!("Failed to parse response: {}", e)),
                                    ..(*state).clone()
                                });
                            }
                        }
                    } else {
                        state.set(AppState {
                            loading: false,
                            error: Some(format!("HTTP {}", resp.status())),
                            ..(*state).clone()
                        });
                    }
                }
                Err(e) => {
                    state.set(AppState {
                        loading: false,
                        error: Some(format!("Request failed: {}. Is the backend running?", e)),
                        ..(*state).clone()
                    });
                }
            }
        });
    });

    let on_toggle = {
        let state = state.clone();
        Callback::from(move |id: String| {
            let current = (*state).clone();
            let selected = if current.selected.contains(&id) {
                current
                    .selected
                    .iter()
                    .filter(|s| *s != &id)
                    .cloned()
                    .collect()
            } else {
                let mut s = current.selected.clone();
                s.push(id);
                s
            };
            state.set(AppState {
                selected,
                ..current
            });
        })
    };

    app_view(&*state, on_toggle)
}

fn app_view(state: &AppState, on_toggle: Callback<String>) -> Html {
    html! {
        <div class="app">
            <header class="header">
                <h1 class="title">{"Fractal Flame"}</h1>
                <p class="subtitle">{"Select variations for generation (click to toggle)"}</p>
            </header>

            <main class="main">
                if state.loading {
                    <div class="loading">
                        <div class="spinner"></div>
                        <p>{"Loading variations..."}</p>
                    </div>
                } else if let Some(ref err) = state.error {
                    <div class="error">
                        <span class="error-icon">{"⚠"}</span>
                        <p>{err}</p>
                    </div>
                } else {
                    <div class="variations-grid">
                        {state.variations.iter().map(|v| variation_card(v, &state.selected, &on_toggle)).collect::<Html>()}
                    </div>

                    if !state.selected.is_empty() {
                        <div class="formula-panel selected-panel">
                            <h3>
                                {"Selected: "}
                                <span class="selected-count">{state.selected.len()}</span>
                                {" variation(s)"}
                            </h3>
                            <div class="selected-list">
                                {state.selected.iter().filter_map(|id| {
                                    state.variations.iter().find(|v| &v.id == id)
                                }).map(|v| html! {
                                    <div key={v.id.clone()} class="selected-item">
                                        <span class="selected-name">{&v.name}</span>
                                        <div class="selected-formula">
                                            <FormulaDisplay formula={v.formula_latex.clone()} />
                                        </div>
                                    </div>
                                }).collect::<Html>()}
                            </div>
                        </div>
                    }
                }
            </main>
        </div>
    }
}

#[derive(Clone, Properties, PartialEq)]
struct FormulaDisplayProps {
    formula: String,
}

#[function_component(FormulaDisplay)]
fn formula_display(props: &FormulaDisplayProps) -> Html {
    let html = katex::render_to_html(&props.formula);
    let parsed = Html::from_html_unchecked(html.into());
    html! {
        <div class="katex-container">{parsed}</div>
    }
}

fn variation_card(v: &VariationDto, selected: &[String], on_toggle: &Callback<String>) -> Html {
    let is_selected = selected.contains(&v.id);
    let id = v.id.clone();
    let on_click = {
        let on_toggle = on_toggle.clone();
        Callback::from(move |_| on_toggle.emit(id.clone()))
    };

    html! {
        <div
            class={if is_selected { "variation-card selected" } else { "variation-card" }}
            onclick={on_click}
        >
            <div class="card-header">
                <span class="card-name">{&v.name}</span>
                <span class="card-id">{&v.id}</span>
            </div>
            <div class="card-formula">
                <FormulaDisplay formula={v.formula_latex.clone()} />
            </div>
        </div>
    }
}
