use crate::katex;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, KeyboardEvent, MouseEvent};
use yew::prelude::*;

fn api_base() -> &'static str {
    // Backend по умолчанию на порту 3000; trunk (фронт) — на 8080
    option_env!("API_BASE").unwrap_or("http://localhost:3000")
}

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

#[derive(Clone, Debug, Serialize)]
struct StartRenderRequest {
    variation_ids: Vec<String>,
    symmetry: usize,
    gamma: f64,
    width: usize,
    height: usize,
}

#[derive(Clone, Debug, Deserialize)]
struct StartRenderResponse {
    job_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppState {
    variations: Vec<VariationDto>,
    selected: Vec<String>,
    loading: bool,
    error: Option<String>,
    symmetry: usize,
    gamma: f64,
    width: usize,
    height: usize,
    last_job_id: Option<String>,
    /// Data URL готовой картинки (когда поллинг получил 200)
    last_render_image: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            variations: Vec::new(),
            selected: Vec::new(),
            loading: false,
            error: None,
            symmetry: 4,
            gamma: 2.2,
            width: 1920,
            height: 1080,
            last_job_id: None,
            last_render_image: None,
        }
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let state = use_state(AppState::default);
    let state_clone = state.clone();

    // Поллинг результата рендера
    let state_for_poll = state.clone();
    use_effect_with(
        state.last_job_id.clone(),
        move |job_id_opt| {
            let job_id = match job_id_opt {
                Some(id) if !id.is_empty() => id.clone(),
                _ => return,
            };
            let state = state_for_poll.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let poll_interval_ms = 2000;
                loop {
                    let url = format!("{}/api/render/{}/result", api_base(), job_id);
                    match Request::get(&url).send().await {
                        Ok(resp) => {
                            if resp.status() == 200 {
                                if let Ok(bytes) = resp.binary().await {
                                    let arr = js_sys::Uint8Array::from(bytes.as_slice());
                                    let array = js_sys::Array::new();
                                    array.push(&arr.buffer());
                                    let opts = web_sys::BlobPropertyBag::new();
                                    opts.set_type("image/png");
                                    if let Ok(blob) = web_sys::Blob::new_with_u8_array_sequence_and_options(
                                        &array.into(),
                                        &opts,
                                    ) {
                                        if let Ok(url_obj) = web_sys::Url::create_object_url_with_blob(&blob) {
                                            state.set(AppState {
                                                last_render_image: Some(url_obj),
                                                ..(*state).clone()
                                            });
                                            return;
                                        }
                                    }
                                }
                            }
                            // 202 или ошибка — продолжаем поллить
                        }
                        Err(_) => {}
                    }
                    gloo_timers::future::TimeoutFuture::new(poll_interval_ms).await;
                }
            });
        },
    );

    use_effect_with((), move |_| {
        let state = state_clone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            state.set(AppState {
                loading: true,
                ..(*state).clone()
            });

            let url = format!("{}/api/variations", api_base());
            match Request::get(url.as_str()).send().await {
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

    let on_symmetry_change = {
        let state = state.clone();
        Callback::from(move |v: usize| {
            let current = (*state).clone();
            state.set(AppState {
                symmetry: v,
                ..current
            });
        })
    };

    let on_gamma_change = {
        let state = state.clone();
        Callback::from(move |v: f64| {
            let current = (*state).clone();
            state.set(AppState {
                gamma: v,
                ..current
            });
        })
    };

    let on_width_change = {
        let state = state.clone();
        Callback::from(move |v: usize| {
            let current = (*state).clone();
            state.set(AppState {
                width: v,
                ..current
            });
        })
    };

    let on_height_change = {
        let state = state.clone();
        Callback::from(move |v: usize| {
            let current = (*state).clone();
            state.set(AppState {
                height: v,
                ..current
            });
        })
    };

    let on_start_render = {
        let state = state.clone();
        Callback::from(move |_| {
            let state = state.clone();
            let current = (*state).clone();
            wasm_bindgen_futures::spawn_local(async move {
                let body = StartRenderRequest {
                    variation_ids: current.selected.clone(),
                    symmetry: current.symmetry,
                    gamma: current.gamma,
                    width: current.width,
                    height: current.height,
                };
                let url = format!("{}/api/render/start", api_base());
                let result = match Request::post(url.as_str()).json(&body) {
                    Ok(req) => req.send().await,
                    Err(e) => {
                        state.set(AppState {
                            error: Some(format!("Request failed: {}", e)),
                            ..(*state).clone()
                        });
                        return;
                    }
                };
                match result {
                    Ok(resp) if resp.ok() => {
                        if let Ok(data) = resp.json::<StartRenderResponse>().await {
                            state.set(AppState {
                                error: None,
                                last_job_id: Some(data.job_id.clone()),
                                last_render_image: None,
                                ..(*state).clone()
                            });
                        }
                    }
                    Ok(resp) => {
                        state.set(AppState {
                            error: Some(format!("HTTP {}", resp.status())),
                            ..(*state).clone()
                        });
                    }
                    Err(e) => {
                        state.set(AppState {
                            error: Some(format!("Request failed: {}", e)),
                            ..(*state).clone()
                        });
                    }
                }
            });
        })
    };

    app_view(
        &*state,
        on_toggle,
        on_symmetry_change,
        on_gamma_change,
        on_width_change,
        on_height_change,
        on_start_render,
    )
}

fn app_view(
    state: &AppState,
    on_toggle: Callback<String>,
    on_symmetry_change: Callback<usize>,
    on_gamma_change: Callback<f64>,
    on_width_change: Callback<usize>,
    on_height_change: Callback<usize>,
    on_start_render: Callback<()>,
) -> Html {
    html! {
        <div class="app">
            <header class="header">
                <h1 class="title">{"Fractal Flame"}</h1>
                <p class="subtitle">{"Select variations for generation (click to toggle)"}</p>
                <div class="preview-controls">
                    <label>
                        {"Symmetry: "}
                        <input
                            type="number"
                            min="1"
                            max="24"
                            value={state.symmetry.to_string()}
                            onchange={move |e: Event| {
                                let input = e.target_dyn_into::<HtmlInputElement>();
                                if let Some(input) = input {
                                    if let Ok(v) = input.value().parse::<usize>() {
                                        on_symmetry_change.emit(v);
                                    }
                                }
                            }}
                        />
                    </label>
                    <label>
                        {"Gamma: "}
                        <input
                            type="number"
                            min="0.1"
                            max="5.0"
                            step="0.1"
                            value={state.gamma.to_string()}
                            onchange={move |e: Event| {
                                let input = e.target_dyn_into::<HtmlInputElement>();
                                if let Some(input) = input {
                                    if let Ok(v) = input.value().parse::<f64>() {
                                        on_gamma_change.emit(v);
                                    }
                                }
                            }}
                        />
                    </label>
                    <label>
                        {"Width: "}
                        <input
                            type="number"
                            min="64"
                            max="4096"
                            value={state.width.to_string()}
                            onchange={move |e: Event| {
                                let input = e.target_dyn_into::<HtmlInputElement>();
                                if let Some(input) = input {
                                    if let Ok(v) = input.value().parse::<usize>() {
                                        on_width_change.emit(v);
                                    }
                                }
                            }}
                        />
                    </label>
                    <label>
                        {"Height: "}
                        <input
                            type="number"
                            min="64"
                            max="4096"
                            value={state.height.to_string()}
                            onchange={move |e: Event| {
                                let input = e.target_dyn_into::<HtmlInputElement>();
                                if let Some(input) = input {
                                    if let Ok(v) = input.value().parse::<usize>() {
                                        on_height_change.emit(v);
                                    }
                                }
                            }}
                        />
                    </label>
                </div>
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
                        {state.variations.iter().map(|v| variation_card(v, &state.selected, state.symmetry, state.gamma, &on_toggle)).collect::<Html>()}
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
                            <button
                                class="start-render-btn"
                                onclick={move |_| on_start_render.emit(())}
                            >
                                {"Start render"}
                            </button>
                            if let Some(ref job_id) = state.last_job_id {
                                <JobIdDisplay
                                    job_id={job_id.clone()}
                                    image_url={state.last_render_image.clone()}
                                />
                            }
                        </div>
                    }
                }
            </main>
        </div>
    }
}

#[derive(Clone, Properties, PartialEq)]
struct JobIdDisplayProps {
    job_id: String,
    image_url: Option<String>,
}

#[function_component(JobIdDisplay)]
fn job_id_display(props: &JobIdDisplayProps) -> Html {
    let copied = use_state(|| false);
    let fullscreen_open = use_state(|| false);
    let overlay_ref = NodeRef::default();

    {
        let overlay_ref = overlay_ref.clone();
        use_effect_with(*fullscreen_open, move |is_open| {
            if *is_open {
                if let Some(el) = overlay_ref.get() {
                    let _ = el.dyn_ref::<web_sys::HtmlElement>().map(|e| e.focus());
                }
            }
        });
    }

    let on_id_click = {
        let job_id = props.job_id.clone();
        let copied = copied.clone();
        Callback::from(move |_| {
            let job_id = job_id.clone();
            let copied = copied.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(window) = web_sys::window() {
                    let clipboard = window.navigator().clipboard();
                    let promise = clipboard.write_text(&job_id);
                    if wasm_bindgen_futures::JsFuture::from(promise).await.is_ok() {
                        copied.set(true);
                        let copied_reset = copied.clone();
                        gloo_timers::future::TimeoutFuture::new(1500).await;
                        copied_reset.set(false);
                    }
                }
            });
        })
    };

    html! {
        <div class="job-id-display">
            <div class="job-id-header">
                <span class="job-id-label">{"Picture ID: "}</span>
                <span
                    class={if *copied { "job-id-copyable copied" } else { "job-id-copyable" }}
                    onclick={on_id_click}
                    title="Click to copy"
                >
                    <code class="job-id-value">{&props.job_id}</code>
                    <span class="job-id-copy-icon" aria-hidden="true">
                        {if *copied { "✓" } else { "⧉" }}
                    </span>
                </span>
            </div>
            if props.image_url.is_none() {
                <p class="render-status">{"Rendering... (polling)"}</p>
            } else if let Some(ref url) = props.image_url {
                <div class="render-result-wrapper">
                    <div class="render-result">
                        <img
                            src={url.clone()}
                            alt="Render result"
                            class="render-result-img"
                            onclick={{
                                let fullscreen_open = fullscreen_open.clone();
                                Callback::from(move |_| fullscreen_open.set(true))
                            }}
                        />
                    </div>
                    <div class="render-result-actions">
                        <button
                            class="fullscreen-btn"
                            onclick={{
                                let fullscreen_open = fullscreen_open.clone();
                                Callback::from(move |_| fullscreen_open.set(true))
                            }}
                            title="Fullscreen"
                        >
                            <span class="fullscreen-btn-icon">{"⛶"}</span>
                            {"Fullscreen"}
                        </button>
                        <a
                            href={url.clone()}
                            download="fractal-flame.png"
                            class="download-btn"
                            title="Download image"
                        >
                            <span class="download-btn-icon">{"↓"}</span>
                            {"Download"}
                        </a>
                    </div>
                    if *fullscreen_open {
                        <div
                            ref={overlay_ref.clone()}
                            class="fullscreen-overlay"
                            tabindex="-1"
                            onclick={{
                                let fullscreen_open = fullscreen_open.clone();
                                Callback::from(move |_| fullscreen_open.set(false))
                            }}
                            onkeydown={{
                                let fullscreen_open = fullscreen_open.clone();
                                Callback::from(move |e: KeyboardEvent| {
                                    if e.key() == "Escape" {
                                        fullscreen_open.set(false);
                                    }
                                })
                            }}
                        >
                            <img
                                src={url.clone()}
                                alt="Render result"
                                class="fullscreen-img"
                                onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
                            />
                            <button
                                class="fullscreen-close"
                                onclick={{
                                    let fullscreen_open = fullscreen_open.clone();
                                    Callback::from(move |_| fullscreen_open.set(false))
                                }}
                                title="Close"
                            >
                                {"✕"}
                            </button>
                        </div>
                    }
                </div>
            }
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

#[derive(Clone, Properties, PartialEq)]
struct PreviewImageProps {
    src: String,
    alt: String,
}

#[function_component(PreviewImage)]
fn preview_image(props: &PreviewImageProps) -> Html {
    let loaded = use_state(|| false);
    let loaded_for_effect = loaded.clone();

    use_effect_with(props.src.clone(), move |_| {
        loaded_for_effect.set(false);
    });

    let on_load = {
        let loaded = loaded.clone();
        Callback::from(move |_| loaded.set(true))
    };

    html! {
        <div class="card-preview">
            <div class={if *loaded { "shimmer shimmer-hidden" } else { "shimmer" }} />
            <img
                src={props.src.clone()}
                alt={props.alt.clone()}
                onload={on_load}
                class={if *loaded { "preview-img" } else { "preview-img preview-img-loading" }}
            />
        </div>
    }
}

fn variation_card(
    v: &VariationDto,
    selected: &[String],
    symmetry: usize,
    gamma: f64,
    on_toggle: &Callback<String>,
) -> Html {
    let is_selected = selected.contains(&v.id);
    let id = v.id.clone();
    let on_click = {
        let on_toggle = on_toggle.clone();
        Callback::from(move |_| on_toggle.emit(id.clone()))
    };

    let preview_url = format!(
        "{}/api/variations/{}/preview?symmetry={}&gamma={}",
        api_base(),
        v.id,
        symmetry,
        gamma
    );

    html! {
        <div
            class={if is_selected { "variation-card selected" } else { "variation-card" }}
            onclick={on_click}
        >
            <PreviewImage src={preview_url} alt={format!("Preview of {}", v.name)} />
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
