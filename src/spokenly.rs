use anyhow::{Result, bail};
use crate::model::TranscriptData;

fn parse_response(_bytes: &[u8], _duration_ms: u64) -> Result<TranscriptData> {
    bail!("Spokenly prepis ni podprt")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn response(segments: serde_json::Value) -> Vec<u8> {
        serde_json::to_vec(&json!({"jsonrpc":"2.0", "id":1, "result": {
            "content": [{"type":"text", "text": json!({
                "modelId":{"predefined":"parakeetTDT06V2"}, "segments":segments
            }).to_string()}]
        }})).unwrap()
    }

    #[test]
    fn spokenly_segments_keep_measured_times_without_inventing_words() {
        let bytes = response(json!([
            {"text":"Prvi gumb.","start":0.125,"end":1.5},
            {"text":"Drugi gumb.","start":2.0,"end":3.25}
        ]));
        let transcript = parse_response(&bytes, 4_000).unwrap();
        assert_eq!(transcript.source, "spokenly");
        assert_eq!(transcript.model.as_deref(), Some("parakeetTDT06V2"));
        assert_eq!(transcript.text, "Prvi gumb. Drugi gumb.");
        assert_eq!((transcript.segments[0].t0, transcript.segments[1].t1), (125, 3250));
        assert!(transcript.words.is_empty(), "segment timestamps must not become invented word times");
    }

    #[test]
    fn spokenly_rejects_errors_and_invalid_audio_times() {
        for bytes in [
            br#"{"error":{"code":-1,"message":"failed"}}"#.to_vec(),
            br#"{"result":{"isError":true,"content":[{"type":"text","text":"failed"}]}}"#.to_vec(),
            response(json!([{"text":"wrong","start":-1.0,"end":2.0}])),
            response(json!([{"text":"wrong","start":3.0,"end":2.0}])),
            response(json!([{"text":"wrong","start":0.0,"end":99999.0}]))
        ] {
            assert!(parse_response(&bytes, 4_000).is_err());
        }
    }
}
