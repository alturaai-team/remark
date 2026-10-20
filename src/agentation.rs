use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capture {
    pub session_id: String,
    pub url: String,
    pub annotations: Vec<Value>,
}

fn select(_bytes: &[u8], session: &str, _start: u64, _end: u64) -> Result<Capture> {
    Ok(Capture {session_id: session.into(), url: String::new(), annotations: Vec::new()})
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
        assert_eq!(capture.annotations.len(), 2, "old and future annotations must not enter this capture");
        assert_eq!(capture.annotations[0]["sourceFile"], "src/Save.tsx:12");
        assert_eq!(capture.annotations[0]["boundingBox"]["y"], 40, "document coordinates must remain document coordinates");
        assert_eq!(capture.annotations[1]["elementPath"], "#panel");
        assert!(select(&bytes, "other", 1000, 2000).is_err(), "a different session must never be attached");
    }
}
