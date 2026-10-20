use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capture {
    pub session_id: String,
    pub started_at_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub annotations: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub port: u16,
    pub session_id: String,
}

pub fn sessions(port: u16) -> Result<Value> {
    Ok(serde_json::from_slice(&crate::local_api::request(
        port,
        "/sessions",
        None,
        3,
    )?)?)
}

pub fn connect(port: u16, session: &str) -> Result<()> {
    let bytes = crate::local_api::request(port, &session_path(session)?, None, 3)?;
    select(&bytes, session, 0, 0)?;
    let path = crate::handoff::default_config_path();
    crate::handoff::set_config_value_in(
        &path,
        "agentation",
        serde_json::to_value(Config {
            port,
            session_id: session.into(),
        })?,
    )?;
    let mut experiments = crate::handoff::config().experiments;
    experiments.insert("agentation".into(), true);
    crate::handoff::set_config_value_in(&path, "experiments", serde_json::to_value(experiments)?)
}

pub fn capture(start: u64, end: u64) -> Option<Capture> {
    if !crate::handoff::experiment("agentation") {
        return None;
    }
    let config = crate::handoff::config().agentation?;
    let result = (|| {
        let bytes =
            crate::local_api::request(config.port, &session_path(&config.session_id)?, None, 3)?;
        select(&bytes, &config.session_id, start, end)
    })();
    Some(match result {
        Ok(capture) => capture,
        Err(error) => Capture {
            session_id: config.session_id,
            started_at_ms: start,
            url: None,
            annotations: Vec::new(),
            error: Some(format!("Pripomb Agentation ni mogoče prebrati: {error}")),
        },
    })
}

fn session_path(session: &str) -> Result<String> {
    ensure!(
        !session.is_empty()
            && session
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b)),
        "Seja Agentation nima veljavnega ID"
    );
    Ok(format!("/sessions/{session}"))
}

#[derive(Deserialize)]
struct Session {
    id: String,
    url: String,
    annotations: Vec<Value>,
}

fn select(bytes: &[u8], session: &str, start: u64, end: u64) -> Result<Capture> {
    let response: Session =
        serde_json::from_slice(bytes).context("Agentation ni vrnil veljavne seje")?;
    ensure!(response.id == session, "Agentation je vrnil drugo sejo");
    ensure!(start <= end, "Čas posnetka ni veljaven");
    let mut annotations = Vec::new();
    for annotation in response.annotations {
        let timestamp = annotation
            .get("timestamp")
            .and_then(Value::as_u64)
            .context("Pripomba nima časa")?;
        if timestamp < start || timestamp > end {
            continue;
        }
        for field in ["id", "comment", "element", "elementPath"] {
            ensure!(
                annotation.get(field).and_then(Value::as_str).is_some(),
                "Pripomba nima polja {field}"
            );
        }
        if let Some(id) = annotation.get("sessionId") {
            ensure!(id.as_str() == Some(session), "Pripomba pripada drugi seji");
        }
        annotations.push(annotation);
    }
    annotations.sort_by_key(|a| a["timestamp"].as_u64());
    Ok(Capture {
        session_id: session.into(),
        started_at_ms: start,
        url: Some(response.url),
        annotations,
        error: None,
    })
}

pub fn times(capture: &Capture) -> Vec<u64> {
    capture
        .annotations
        .iter()
        .filter_map(|a| a["timestamp"].as_u64())
        .map(|t| t.saturating_sub(capture.started_at_ms))
        .collect()
}

pub fn summary(capture: &Capture, frames: &[crate::model::Keyframe]) -> String {
    let mut out = format!("Pripombe Agentation (seja {}):\n", capture.session_id);
    if let Some(error) = &capture.error {
        out.push_str(&format!("{error}\n"));
        return out;
    }
    if let Some(url) = &capture.url {
        out.push_str(&format!("Stran: {url}\n"));
    }
    for (annotation, t) in capture.annotations.iter().zip(times(capture)) {
        out.push_str(&format!("- {}", crate::model::format_clock(t)));
        if let Some(frame) = frames.iter().min_by_key(|f| f.t.abs_diff(t)) {
            out.push_str(&format!(" [najbližja slika #{}]", frame.index));
        }
        let context: serde_json::Map<String, Value> = [
            "id",
            "comment",
            "element",
            "elementPath",
            "reactComponents",
            "sourceFile",
            "url",
            "selectedText",
        ]
        .into_iter()
        .filter_map(|field| {
            annotation
                .get(field)
                .map(|value| (field.into(), value.clone()))
        })
        .collect();
        out.push_str(&format!(" {}\n", Value::Object(context)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn agentation_selects_only_this_recording_and_preserves_component_context() {
        let bytes = serde_json::to_vec(&json!({"id":"chosen", "url":"http://localhost:3000/inbox", "annotations":[
            {"id":"old","timestamp":999,"comment":"old","element":"button","elementPath":"#old"},
            {"id":"one","timestamp":1100,"comment":"Popravi gumb","element":"button","elementPath":"#save","sourceFile":"src/Save.tsx:12","reactComponents":"App > Save","boundingBox":{"x":12,"y":40,"width":80,"height":20}},
            {"id":"two","timestamp":1400,"comment":"Premakni","element":"div","elementPath":"#panel"},
            {"id":"future","timestamp":2001,"comment":"later","element":"div","elementPath":"#later"}
        ]})).unwrap();
        let capture = select(&bytes, "chosen", 1000, 2000).unwrap();
        assert_eq!(
            capture.annotations.len(),
            2,
            "old and future annotations must not enter this capture"
        );
        assert_eq!(capture.annotations[0]["sourceFile"], "src/Save.tsx:12");
        assert_eq!(times(&capture), vec![100, 400], "annotation times must use the recording clock");
        let restored: Capture = serde_json::from_slice(&serde_json::to_vec(&capture).unwrap()).unwrap();
        assert_eq!(restored, capture, "saved captures must retain annotation geometry and component identity");
        assert_eq!(
            capture.annotations[0]["boundingBox"]["y"], 40,
            "document coordinates must remain document coordinates"
        );
        assert_eq!(capture.annotations[1]["elementPath"], "#panel");
        assert!(
            select(&bytes, "other", 1000, 2000).is_err(),
            "a different session must never be attached"
        );
    }
}
