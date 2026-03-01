use crate::katex;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{EventSource, HtmlInputElement, KeyboardEvent, MessageEvent, MouseEvent};
use yew::prelude::*;

fn api_base() -> &'static str {
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

#[derive(Clone, Debug, Deserialize)]
struct SseProgressData {
    #[allow(dead_code)]
    status: String,
    progress: Option<usize>,
    total: Option<usize>,
    intermediate_version: Option<u64>,
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
    last_render_image: Option<String>,
    render_progress: Option<usize>,
    render_total: Option<usize>,
    intermediate_url: Option<String>,
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
            render_progress: None,
            render_total: None,
            intermediate_url: None,
        }
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let state = use_state(AppState::default);
    let state_clone = state.clone();

    let state_for_sse = state.clone();
    use_effect_with(
        state.last_job_id.clone(),
        move |job_id_opt: &Option<String>| {
            let es: Option<EventSource> = (|| {
                let job_id = job_id_opt.as_ref().filter(|id| !id.is_empty())?;

                let sse_url = format!("{}/api/render/{}/progress", api_base(), job_id);
                let es = EventSource::new(&sse_url).ok()?;

                let on_progress = {
                    let state = state_for_sse.clone();
                    let job_id = job_id.clone();
                    Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
                        if let Some(data) = event.data().as_string() {
                            if let Ok(info) = serde_json::from_str::<SseProgressData>(&data) {
                                let mut next = (*state).clone();
                                next.render_progress = info.progress;
                                next.render_total = info.total;
                                if let Some(v) = info.intermediate_version {
                                    if v > 0 {
                                        next.intermediate_url = Some(format!(
                                            "{}/api/render/{}/intermediate?v={}",
                                            api_base(),
                                            job_id,
                                            v
                                        ));
                                    }
                                }
                                state.set(next);
                            }
                        }
                    })
                };
                es.add_event_listener_with_callback(
                    "progress",
                    on_progress.as_ref().unchecked_ref(),
                )
                .ok();
                on_progress.forget();

                let on_completed = {
                    let state = state_for_sse.clone();
                    let job_id = job_id.clone();
                    let es_ref = es.clone();
                    Closure::<dyn FnMut(MessageEvent)>::new(move |_event: MessageEvent| {
                        es_ref.close();
                        let state = state.clone();
                        let job_id = job_id.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            let url =
                                format!("{}/api/render/{}/result", api_base(), job_id);
                            if let Ok(resp) = Request::get(&url).send().await {
                                if resp.status() == 200 {
                                    if let Ok(bytes) = resp.binary().await {
                                        let arr =
                                            js_sys::Uint8Array::from(bytes.as_slice());
                                        let array = js_sys::Array::new();
                                        array.push(&arr.buffer());
                                        let opts = web_sys::BlobPropertyBag::new();
                                        opts.set_type("image/png");
                                        if let Ok(blob) =
                                            web_sys::Blob::new_with_u8_array_sequence_and_options(
                                                &array.into(),
                                                &opts,
                                            )
                                        {
                                            if let Ok(url_obj) =
                                                web_sys::Url::create_object_url_with_blob(
                                                    &blob,
                                                )
                                            {
                                                let mut next = (*state).clone();
                                                next.last_render_image = Some(url_obj);
                                                next.render_progress = next.render_total;
                                                state.set(next);
                                            }
                                        }
                                    }
                                }
                            }
                        });
                    })
                };
                es.add_event_listener_with_callback(
                    "completed",
                    on_completed.as_ref().unchecked_ref(),
                )
                .ok();
                on_completed.forget();

                let on_failed = {
                    let state = state_for_sse.clone();
                    let es_ref = es.clone();
                    Closure::<dyn FnMut(MessageEvent)>::new(move |_event: MessageEvent| {
                        es_ref.close();
                        let mut next = (*state).clone();
                        next.error = Some("Render failed".to_string());
                        next.render_progress = None;
                        next.render_total = None;
                        state.set(next);
                    })
                };
                es.add_event_listener_with_callback(
                    "failed",
                    on_failed.as_ref().unchecked_ref(),
                )
                .ok();
                on_failed.forget();

                Some(es)
            })();

            move || {
                if let Some(es) = es {
                    es.close();
                }
            }
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

    let on_select_all = {
        let state = state.clone();
        Callback::from(move |_| {
            let current = (*state).clone();
            let all_ids: Vec<String> = current.variations.iter().map(|v| v.id.clone()).collect();
            let selected = if current.selected.len() == all_ids.len() {
                vec![]
            } else {
                all_ids
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
                                render_progress: None,
                                render_total: None,
                                intermediate_url: None,
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
        on_select_all,
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
    on_select_all: Callback<()>,
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
                <FetchByPictureId />
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
                    <div class="variations-actions">
                        <button
                            class="select-all-btn"
                            onclick={move |_| on_select_all.emit(())}
                        >
                            {if !state.variations.is_empty() && state.selected.len() == state.variations.len() {
                                "Deselect all"
                            } else {
                                "Select all"
                            }}
                        </button>
                    </div>
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
                                    render_progress={state.render_progress}
                                    render_total={state.render_total}
                                    intermediate_url={state.intermediate_url.clone()}
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
    #[prop_or_default]
    render_progress: Option<usize>,
    #[prop_or_default]
    render_total: Option<usize>,
    #[prop_or_default]
    intermediate_url: Option<String>,
}

#[function_component(JobIdDisplay)]
fn job_id_display(props: &JobIdDisplayProps) -> Html {
    let copied = use_state(|| false);
    let fullscreen_open = use_state(|| false);
    let overlay_ref = NodeRef::default();

    let ready_intermediate = use_state(|| Option::<String>::None);
    {
        let ready = ready_intermediate.clone();
        use_effect_with(props.intermediate_url.clone(), move |url: &Option<String>| {
            if url.is_none() {
                ready.set(None);
            }
        });
    }
    let on_intermediate_load = {
        let ready = ready_intermediate.clone();
        let pending_url = props.intermediate_url.clone();
        Callback::from(move |_| {
            ready.set(pending_url.clone());
        })
    };

    let is_completed = props.image_url.is_some();
    let display_url: Option<String> = if let Some(ref final_url) = props.image_url {
        Some(final_url.clone())
    } else {
        (*ready_intermediate).clone()
    };

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

    let img_onclick = if is_completed {
        let fs = fullscreen_open.clone();
        Some(Callback::from(move |_: MouseEvent| fs.set(true)))
    } else {
        None
    };

    let progress_html = if !is_completed {
        if let (Some(progress), Some(total)) = (props.render_progress, props.render_total) {
            html! {
                <div class="render-progress">
                    <div class="progress-bar-container">
                        <div
                            class="progress-bar-fill"
                            style={format!(
                                "width: {}%",
                                if total > 0 { (progress as f64 / total as f64 * 100.0).min(100.0) } else { 0.0 }
                            )}
                        />
                    </div>
                    <p class="render-status">
                        {format!(
                            "Rendering: {:.1}%  ({}/{})",
                            if total > 0 { progress as f64 / total as f64 * 100.0 } else { 0.0 },
                            progress,
                            total
                        )}
                    </p>
                </div>
            }
        } else {
            html! { <p class="render-status">{"Starting render..."}</p> }
        }
    } else {
        html! {}
    };

    let preloader_html = if !is_completed {
        if let Some(ref iurl) = props.intermediate_url {
            html! {
                <img
                    src={iurl.clone()}
                    alt=""
                    class="intermediate-preloader"
                    onload={on_intermediate_load.reform(|_: web_sys::Event| ())}
                />
            }
        } else {
            html! {}
        }
    } else {
        html! {}
    };

    let image_html = if let Some(ref url) = display_url {
        html! {
            <div class="render-result">
                <img
                    src={url.clone()}
                    alt={if is_completed { "Render result" } else { "Intermediate render" }}
                    class={if is_completed { "render-result-img" } else { "render-result-img intermediate-img" }}
                    onclick={img_onclick}
                />
            </div>
        }
    } else {
        html! {}
    };

    let actions_html = if let Some(ref url) = props.image_url {
        html! { <>
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
        </> }
    } else {
        html! {}
    };

    let render_body = html! { <>
        {progress_html}
        {preloader_html}
        {image_html}
        {actions_html}
    </> };

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
            {render_body}
        </div>
    }
}

#[function_component(FetchByPictureId)]
fn fetch_by_picture_id() -> Html {
    let id_input = use_state(|| String::new());
    let loaded_job_id = use_state(|| Option::<String>::None);
    let loading = use_state(|| false);
    let image_url = use_state(|| Option::<String>::None);
    let error = use_state(|| Option::<String>::None);

    let on_load = {
        let id_input = id_input.clone();
        let loaded_job_id = loaded_job_id.clone();
        let loading = loading.clone();
        let image_url = image_url.clone();
        let error = error.clone();
        Callback::from(move |_| {
            let id = (*id_input).trim().to_string();
            if id.is_empty() {
                error.set(Some("Enter Picture ID".to_string()));
                return;
            }
            loading.set(true);
            error.set(None);
            image_url.set(None);
            let loaded_job_id = loaded_job_id.clone();
            let loading = loading.clone();
            let image_url = image_url.clone();
            let error = error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("{}/api/render/{}/result", api_base(), id);
                match Request::get(&url).send().await {
                    Ok(resp) => {
                        loading.set(false);
                        if resp.status() == 200 {
                            loaded_job_id.set(Some(id.clone()));
                            if let Ok(bytes) = resp.binary().await {
                                let arr = js_sys::Uint8Array::from(bytes.as_slice());
                                let array = js_sys::Array::new();
                                array.push(&arr.buffer());
                                let opts = web_sys::BlobPropertyBag::new();
                                opts.set_type("image/png");
                                if let Ok(blob) =
                                    web_sys::Blob::new_with_u8_array_sequence_and_options(
                                        &array.into(),
                                        &opts,
                                    )
                                {
                                    if let Ok(url_obj) =
                                        web_sys::Url::create_object_url_with_blob(&blob)
                                    {
                                        image_url.set(Some(url_obj));
                                        error.set(None);
                                    }
                                }
                            }
                        } else if resp.status() == 202 {
                            error.set(Some("Rendering in progress, try again later".to_string()));
                        } else {
                            error.set(Some(format!("HTTP {}", resp.status())));
                        }
                    }
                    Err(e) => {
                        loading.set(false);
                        error.set(Some(format!("Request failed: {}", e)));
                    }
                }
            });
        })
    };

    let id_input_clone = id_input.clone();
    html! {
        <div class="fetch-by-id">
            <div class="fetch-by-id-row">
            <label class="fetch-by-id-label">
                {"Open by Picture ID: "}
                <input
                    type="text"
                    class="fetch-by-id-input"
                    placeholder="Paste Picture ID..."
                    value={(*id_input).clone()}
                    oninput={move |e: web_sys::InputEvent| {
                        let input = e.target_dyn_into::<HtmlInputElement>();
                        if let Some(input) = input {
                            id_input_clone.set(input.value());
                        }
                    }}
                />
            </label>
            <button
                class="fetch-by-id-btn"
                onclick={on_load}
                disabled={*loading}
            >
                {if *loading { "Loading..." } else { "Load" }}
            </button>
            </div>
            if let Some(ref err) = *error {
                <span class="fetch-by-id-error">{err}</span>
            }
            if let (Some(ref job_id), Some(ref url)) = (loaded_job_id.as_ref(), image_url.as_ref()) {
                <JobIdDisplay
                    job_id={(*job_id).clone()}
                    image_url={Some((*url).clone())}
                />
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
