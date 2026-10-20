use anyhow::{Context, Result, ensure};
use serde_json::Value;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn request(port: u16, path: &str, body: Option<&Value>, seconds: u32) -> Result<Vec<u8>> {
    ensure!(port != 0, "Lokalni vmesnik nima veljavnih vrat");
    let mut command = Command::new("/usr/bin/curl");
    command.args([
        "--silent",
        "--fail",
        "--noproxy",
        "*",
        "--proto",
        "=http",
        "--connect-timeout",
        "2",
        "--max-time",
        &seconds.to_string(),
        "--max-filesize",
        "8388608",
        &format!("http://127.0.0.1:{port}{path}"),
    ]);
    if body.is_some() {
        command.args([
            "--header",
            "Content-Type: application/json",
            "--data-binary",
            "@-",
        ]);
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("Lokalnega vmesnika ni mogoče zagnati")?;
    if let Some(body) = body {
        let mut stdin = child.stdin.take().context("Lokalni vmesnik nima vhoda")?;
        stdin.write_all(&serde_json::to_vec(body)?)?;
    }
    let output = child.wait_with_output()?;
    ensure!(
        output.status.success(),
        "Lokalni vmesnik na vratih {port} ni uspel"
    );
    Ok(output.stdout)
}
