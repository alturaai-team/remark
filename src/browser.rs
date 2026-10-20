use crate::model::{Keyframe, format_clock};
use crate::page::Event;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capture {
    pub started_at_ms: u64,
    pub annotations: Vec<Event>,
}

pub fn capture(start: u64, end: u64) -> Option<Capture> {
    if !crate::handoff::experiment(crate::handoff::AGENT_CONTEXT) {
        return None;
    }
    let annotations: Vec<Event> = crate::page::between(start, end)
        .into_iter()
        .filter(|event| {
            event.kind == "annotation"
                && event.focused
                && event.selection.is_some()
                && event.target.is_some()
        })
        .collect();
    if annotations.is_empty() {
        return None;
    }
    Some(Capture {
        started_at_ms: start,
        annotations,
    })
}

pub fn times(capture: &Capture) -> Vec<u64> {
    capture
        .annotations
        .iter()
        .map(|e| e.t.saturating_sub(capture.started_at_ms))
        .collect()
}

pub fn summary(capture: &Capture, frames: &[Keyframe]) -> String {
    let mut out = String::from("Označeni elementi Remark:\n");
    for event in &capture.annotations {
        let t = event.t.saturating_sub(capture.started_at_ms);
        let Some(selection) = &event.selection else {
            continue;
        };
        out.push_str(&format!(
            "- Oznaka {} ({}) {}\n  Selector: {}\n",
            selection.number,
            format_clock(t),
            selection.url,
            selection.selector_path.join(" > ⟨shadow⟩ > ")
        ));
        if let Some(target) = &event.target {
            if let Some(text) = &target.text {
                out.push_str(&format!("  Element: {text}\n"));
            }
            if let Some(component) = &target.component {
                out.push_str(&format!("  Komponenta: {component}\n"));
            }
            if let Some(source) = &target.source {
                out.push_str(&format!("  Izvor: {source}\n"));
            }
        }
        if let Some(frame) = frames.iter().min_by_key(|f| f.t.abs_diff(t)) {
            out.push_str(&format!("  Najbližja slika: #{}\n", frame.index));
        }
    }
    out
}

pub fn configure_extension(directory: &Path) -> Result<()> {
    let token = crate::handoff::ensure_ingest_token()?;
    let path = directory.join("connection.json");
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)?;
        file.write_all(&serde_json::to_vec(
            &serde_json::json!({"port":crate::phone::DEFAULT_INGEST_PORT,"token":token}),
        )?)?;
    }
    Ok(())
}
