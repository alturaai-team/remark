use crate::model::{TranscriptData, TranscriptSegment};
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::Path;

pub fn connect(port: u16) -> Result<()> {
    let bytes = crate::local_api::request(
        port,
        "/",
        Some(&json!({
            "jsonrpc":"2.0", "id":1, "method":"tools/list"
        })),
        3,
    )?;
    let reply: Value = serde_json::from_slice(&bytes)?;
    let tools = reply
        .pointer("/result/tools")
        .and_then(Value::as_array)
        .context("Spokenly ni vrnil orodij")?;
    ensure!(
        tools.iter().any(|tool| tool["name"] == "transcribe_file"),
        "Spokenly ne podpira prepisa datotek"
    );
    let path = crate::handoff::default_config_path();
    crate::handoff::set_config_value_in(&path, "spokenlyPort", port.into())?;
    crate::handoff::set_config_value_in(&path, "transcriber", "spokenly".into())?;
    crate::handoff::set_config_value_in(&path, "autoSend", false.into())?;
    let mut experiments = crate::handoff::config().experiments;
    experiments.insert("agentContext".into(), true);
    crate::handoff::set_config_value_in(&path, "experiments", serde_json::to_value(experiments)?)
}

pub fn transcribe(wav: &Path, duration_ms: u64) -> Result<TranscriptData> {
    let port = crate::handoff::config()
        .spokenly_port
        .context("Vrata Spokenly niso nastavljena")?;
    let wav = wav.canonicalize().context("Zvočni posnetek ne obstaja")?;
    let body = json!({"jsonrpc":"2.0", "id":1, "method":"tools/call", "params":{
        "name":"transcribe_file", "arguments":{"file_path":wav, "format":"json"}
    }});
    parse_response(
        &crate::local_api::request(port, "/", Some(&body), 600)?,
        duration_ms,
    )
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Transcript {
    model_id: Value,
    segments: Vec<Segment>,
}

#[derive(Deserialize)]
struct Segment {
    text: String,
    start: f64,
    end: f64,
}

fn parse_response(bytes: &[u8], duration_ms: u64) -> Result<TranscriptData> {
    let reply: Value = serde_json::from_slice(bytes).context("Spokenly ni vrnil JSON")?;
    ensure!(reply.get("error").is_none(), "Spokenly je zavrnil zahtevo");
    let result = reply.get("result").context("Spokenly ni vrnil rezultata")?;
    ensure!(
        result.get("isError") != Some(&Value::Bool(true)),
        "Prepis Spokenly ni uspel"
    );
    let content = result
        .get("content")
        .and_then(Value::as_array)
        .context("Spokenly ni vrnil vsebine")?;
    let text = content
        .iter()
        .filter(|block| block.get("type").and_then(Value::as_str) == Some("text"))
        .filter_map(|block| block.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>();
    ensure!(text.len() == 1, "Spokenly ni vrnil enega prepisa");
    let transcript: Transcript =
        serde_json::from_str(text[0]).context("Prepis Spokenly nima veljavne oblike")?;
    let model = match &transcript.model_id {
        Value::Object(values) if values.len() == 1 => values
            .values()
            .next()
            .and_then(Value::as_str)
            .context("Spokenly ni navedel modela")?
            .to_owned(),
        Value::String(name) => name.clone(),
        _ => anyhow::bail!("Spokenly ni navedel modela"),
    };
    let mut segments = Vec::with_capacity(transcript.segments.len());
    for segment in transcript.segments {
        ensure!(
            segment.start.is_finite()
                && segment.end.is_finite()
                && segment.start >= 0.0
                && segment.end >= segment.start
                && (segment.end * 1000.0).round() <= duration_ms as f64,
            "Časi prepisa Spokenly so zunaj posnetka"
        );
        segments.push(TranscriptSegment {
            t0: (segment.start * 1000.0).round() as u64,
            t1: (segment.end * 1000.0).round() as u64,
            text: segment.text,
        });
    }
    ensure!(
        segments.windows(2).all(|s| s[0].t0 <= s[1].t0),
        "Segmenti Spokenly niso časovno urejeni"
    );
    Ok(TranscriptData {
        source: "spokenly".into(),
        model: Some(model),
        text: segments
            .iter()
            .map(|s| s.text.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
        segments,
        ..TranscriptData::default()
    })
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
        }}))
        .unwrap()
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
        assert_eq!(
            (transcript.segments[0].t0, transcript.segments[1].t1),
            (125, 3250)
        );
        assert!(
            transcript.words.is_empty(),
            "segment timestamps must not become invented word times"
        );
    }

    #[test]
    fn spokenly_rejects_errors_and_invalid_audio_times() {
        for bytes in [
            br#"{"error":{"code":-1,"message":"failed"}}"#.to_vec(),
            br#"{"result":{"isError":true,"content":[{"type":"text","text":"failed"}]}}"#.to_vec(),
            response(json!([{"text":"wrong","start":-1.0,"end":2.0}])),
            response(json!([{"text":"wrong","start":3.0,"end":2.0}])),
            response(json!([{"text":"wrong","start":0.0,"end":99999.0}])),
        ] {
            assert!(parse_response(&bytes, 4_000).is_err());
        }
    }
}
