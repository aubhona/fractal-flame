use js_sys::{Object, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::window;

/// Renders LaTeX formula to HTML string via KaTeX. Returns raw formula on error.
pub fn render_to_html(formula: &str) -> String {
    let window = match window() {
        Some(w) => w,
        None => {
            gloo_console::error!("KaTeX: no window");
            return fallback_html(formula);
        }
    };

    let katex = match Reflect::get(&window, &JsValue::from_str("katex")) {
        Ok(k) if !k.is_undefined() && !k.is_null() => k,
        _ => {
            gloo_console::error!("KaTeX: not loaded on window. Add script before app.");
            return fallback_html(formula);
        }
    };

    let options = Object::new();
    let _ = Reflect::set(&options, &"displayMode".into(), &true.into());
    let _ = Reflect::set(&options, &"throwOnError".into(), &false.into());

    let render_fn = match Reflect::get(&katex, &JsValue::from_str("renderToString")) {
        Ok(f) => f,
        Err(e) => {
            gloo_console::error!(&format!("KaTeX: renderToString not found: {:?}", e));
            return fallback_html(formula);
        }
    };

    let render = match render_fn.dyn_ref::<js_sys::Function>() {
        Some(f) => f,
        None => {
            gloo_console::error!("KaTeX: renderToString is not a function");
            return fallback_html(formula);
        }
    };

    match render.call2(&katex, &JsValue::from_str(formula), &options) {
        Ok(result) => result
            .as_string()
            .unwrap_or_else(|| {
                gloo_console::error!("KaTeX: renderToString returned non-string");
                fallback_html(formula)
            }),
        Err(e) => {
            gloo_console::error!(&format!("KaTeX: renderToString error: {:?}", e));
            fallback_html(formula)
        }
    }
}

fn fallback_html(formula: &str) -> String {
    format!(
        r#"<span class="formula-fallback" title="{}">{}</span>"#,
        html_escape(formula),
        html_escape(formula)
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
