//! # HTML QC report rendering
//!
//! ## Scientific scope
//!
//! Human-readable QC artifacts are useful for quickly inspecting orbit-
//! determination output outside a notebook or plotting stack. This module
//! renders a compact HTML summary from the workspace QC JSON structure.
//!
//! The renderer is intentionally presentation-only. It preserves the
//! upstream statistics verbatim and adds no new scientific interpretation.
//!
//! ## Technical scope
//!
//! The main entry point is `render_html`, which accepts a
//! `serde_json::Value` compatible with the workspace QC schema and returns
//! a self-contained HTML string. The output is static and dependency-free
//! so it can be archived as a standalone artifact.
//!
//! It does not compute statistics, fetch assets, or depend on browser-side
//! JavaScript.
//!
//! ## References
//!
//! - Fielding, R., Nottingham, M., & Reschke, J. (2022). HTTP Semantics.
//!   RFC 9110.
//! - Bray, T. (2017). The JavaScript Object Notation (JSON) Data
//!   Interchange Format. RFC 8259.
use std::fmt::Write;

/// Render a `qc.json`-shaped JSON value to HTML.
pub fn render_html(qc: &serde_json::Value) -> String {
    let mut s = String::new();
    s.push_str("<!doctype html>\n<html><head><meta charset=\"utf-8\">");
    s.push_str("<title>spod — QC report</title><style>");
    s.push_str("body{font-family:system-ui,sans-serif;max-width:880px;margin:2em auto;padding:0 1em;color:#222}");
    s.push_str("h1,h2{border-bottom:1px solid #ccc;padding-bottom:.25em}");
    s.push_str("table{border-collapse:collapse;margin:1em 0}");
    s.push_str("th,td{border:1px solid #ddd;padding:.35em .8em;text-align:right}");
    s.push_str("th:first-child,td:first-child{text-align:left}");
    s.push_str("code{background:#f4f4f4;padding:.05em .3em;border-radius:3px}");
    s.push_str("</style></head><body>");
    s.push_str("<h1>spod — QC report</h1>");

    if let Some(run_id) = qc.get("run_id").and_then(|v| v.as_str()) {
        let _ = write!(s, "<p>Run ID: <code>{}</code></p>", html_escape(run_id));
    }
    if let Some(version) = qc.get("schema_version").and_then(|v| v.as_str()) {
        let _ = write!(
            s,
            "<p>Schema version: <code>{}</code></p>",
            html_escape(version)
        );
    }

    if let Some(residuals) = qc.get("residuals") {
        s.push_str("<h2>Residuals</h2>");
        render_residuals(&mut s, residuals);
    }

    if let Some(extra) = qc.as_object() {
        for (k, v) in extra {
            if matches!(k.as_str(), "run_id" | "schema_version" | "residuals") {
                continue;
            }
            let _ = write!(s, "<h2>{}</h2>", html_escape(k));
            let _ = write!(
                s,
                "<pre><code>{}</code></pre>",
                html_escape(&serde_json::to_string_pretty(v).unwrap_or_default())
            );
        }
    }

    s.push_str("</body></html>");
    s
}

fn render_residuals(s: &mut String, r: &serde_json::Value) {
    if let Some(overall) = r.get("overall") {
        s.push_str("<h3>Overall</h3>");
        render_stats_table(s, overall);
    }
    if let Some(by_group) = r.get("by_group").and_then(|v| v.as_object()) {
        s.push_str("<h3>By group</h3>");
        s.push_str("<table><tr><th>group</th><th>n</th><th>mean</th><th>rms</th><th>std</th><th>min</th><th>max</th></tr>");
        for (k, v) in by_group {
            let _ = write!(
                s,
                "<tr><td>{}</td><td>{}</td><td>{:.4e}</td><td>{:.4e}</td><td>{:.4e}</td><td>{:.4e}</td><td>{:.4e}</td></tr>",
                html_escape(k),
                v.get("n").and_then(|x| x.as_u64()).unwrap_or(0),
                f(v, "mean"),
                f(v, "rms"),
                f(v, "std"),
                f(v, "min"),
                f(v, "max"),
            );
        }
        s.push_str("</table>");
    }
}

fn render_stats_table(s: &mut String, v: &serde_json::Value) {
    s.push_str("<table><tr><th>statistic</th><th>value</th></tr>");
    for k in ["n", "mean", "std", "rms", "min", "max"] {
        let val = match v.get(k) {
            Some(j) if j.is_u64() => format!("{}", j.as_u64().unwrap()),
            Some(j) if j.is_f64() => format!("{:.6e}", j.as_f64().unwrap()),
            Some(j) => j.to_string(),
            None => continue,
        };
        let _ = write!(s, "<tr><td>{}</td><td>{}</td></tr>", k, html_escape(&val));
    }
    s.push_str("</table>");
}

fn f(v: &serde_json::Value, key: &str) -> f64 {
    v.get(key).and_then(|x| x.as_f64()).unwrap_or(0.0)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn renders_minimal_html() {
        let qc = json!({
            "run_id": "demo",
            "schema_version": "1.0.0",
            "residuals": {
                "overall": {"n": 3, "mean": 0.0, "std": 0.1, "rms": 0.1, "min": -0.1, "max": 0.1},
                "by_group": {"code": {"n": 2, "mean": 0.0, "std": 0.05, "rms": 0.05, "min": -0.05, "max": 0.05}}
            }
        });
        let h = render_html(&qc);
        assert!(h.starts_with("<!doctype html>"));
        assert!(h.contains("demo"));
        assert!(h.contains("By group"));
    }
}
