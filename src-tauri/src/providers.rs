use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use futures_util::StreamExt;
use tauri::ipc::Channel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    #[serde(default)]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub extra_headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CommitStreamEvent {
    Chunk { text: String },
    Done { message: String },
    Error { message: String },
}

const DEFAULT_SYSTEM_PROMPT: &str = "You are a helpful assistant that generates informative git commit messages based on git diffs output. Skip preamble and remove all backticks surrounding the commit message.";

const INSTRUCTION: &str = "Based on the provided git diff, generate a concise and descriptive commit message.

The commit message should:
1. Has a short title (50-72 characters)
2. The commit message should adhere to the conventional commit format
3. Describe what was changed and why
4. Be clear and informative";

const DIFF_TRUNCATE_CHARS: usize = 5000;

fn prompt_parts(provider: &Provider, diff: &str, note: &str) -> (String, String) {
    let mut user_parts: Vec<String> = Vec::new();
    user_parts.push(INSTRUCTION.to_string());
    let trimmed_note = note.trim();
    if !trimmed_note.is_empty() {
        user_parts.push(format!("Notes from developer (ignore if not relevant): {trimmed_note}"));
    }
    let truncated = if diff.chars().count() > DIFF_TRUNCATE_CHARS {
        let mut s: String = diff.chars().take(DIFF_TRUNCATE_CHARS).collect();
        s.push_str("\n\n[Diff truncated due to size]");
        s
    } else {
        diff.to_string()
    };
    user_parts.push(truncated);
    let user_prompt = user_parts.join("\n\n");

    let system_prompt = provider
        .system_prompt
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(DEFAULT_SYSTEM_PROMPT)
        .to_string();

    (system_prompt, user_prompt)
}

pub async fn generate_commit_message(provider: &Provider, diff: &str, note: &str) -> Result<String, String> {
    let (system_prompt, user_prompt) = prompt_parts(provider, diff, note);

    let body = serde_json::json!({
        "model": provider.model,
        "stream": false,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_prompt }
        ]
    });

    let client = reqwest::Client::builder().build().map_err(|e| e.to_string())?;
    let url = build_chat_url(&provider.base_url);

    let mut req = client.post(&url).header("content-type", "application/json");
    if !provider.api_key.trim().is_empty() {
        req = req.bearer_auth(provider.api_key.trim());
    }
    for (k, v) in &provider.extra_headers {
        req = req.header(k, v);
    }

    let resp = req
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| format!("Failed to connect to {url}: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        let snippet: String = text.chars().take(400).collect();
        return Err(format!("API returned HTTP {status}: {snippet}"));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Response is not valid JSON: {e}"))?;
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| "Model returned empty response. Check selected provider and model.".to_string())?;

    Ok(extract_commit_message(content))
}

/// Streams an OpenAI-compatible SSE chat completion, emitting Chunk/Done/Error
/// events over the Tauri Channel as they arrive.
pub async fn stream_commit_message(
    provider: &Provider,
    diff: &str,
    note: &str,
    on_event: Channel<CommitStreamEvent>,
) -> Result<(), String> {
    let (system_prompt, user_prompt) = prompt_parts(provider, diff, note);

    let body = serde_json::json!({
        "model": provider.model,
        "stream": true,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_prompt }
        ]
    });

    let client = reqwest::Client::builder().build().map_err(|e| e.to_string())?;
    let url = build_chat_url(&provider.base_url);

    let mut req = client.post(&url).header("content-type", "application/json");
    if !provider.api_key.trim().is_empty() {
        req = req.bearer_auth(provider.api_key.trim());
    }
    for (k, v) in &provider.extra_headers {
        req = req.header(k, v);
    }

    let resp = req
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| format!("Failed to connect to {url}: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        let snippet: String = text.chars().take(400).collect();
        let msg = format!("API returned HTTP {status}: {snippet}");
        let _ = on_event.send(CommitStreamEvent::Error { message: msg.clone() });
        return Err(msg);
    }

    let mut buf = String::new();
    let mut acc = String::new();
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Error reading stream: {e}"))?;
        buf.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(pos) = buf.find('\n') {
            let line = buf[..pos].to_string();
            buf = buf[pos + 1..].to_string();
            let Some(data) = line.strip_prefix("data:") else { continue };
            let data = data.trim();
            if data.is_empty() || data == "[DONE]" {
                continue;
            }
            let Ok(v) = serde_json::from_str::<serde_json::Value>(data) else { continue };
            if let Some(err) = v.get("error") {
                let msg = format!("API error: {err}");
                let _ = on_event.send(CommitStreamEvent::Error { message: msg.clone() });
                return Err(msg);
            }
            if let Some(delta) = v["choices"][0]["delta"]["content"].as_str() {
                if !delta.is_empty() {
                    acc.push_str(delta);
                    let _ = on_event.send(CommitStreamEvent::Chunk { text: delta.to_string() });
                }
            }
        }
    }

    let message = extract_commit_message(&acc);
    let _ = on_event.send(CommitStreamEvent::Done { message: message.clone() });
    Ok(())
}

fn build_chat_url(base_url: &str) -> String {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.ends_with("/chat/completions") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/chat/completions")
    }
}

fn extract_commit_message(str: &str) -> String {
    // mirrors cline's extractCommitMessage: strip markdown fences / preamble
    let mut s = str.trim();
    if let Some(rest) = s.strip_prefix("```") {
        let after_newline = rest.find('\n').map(|i| &rest[i + 1..]).unwrap_or("");
        s = after_newline;
    }
    s.strip_suffix("```").unwrap_or(s).trim().to_string()
}