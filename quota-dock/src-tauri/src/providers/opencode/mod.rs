use std::process::Stdio;
use serde::Deserialize;
use tokio::process::Command;

use crate::contracts::{LocalTokens, ModelTokenUsage};
use crate::quota::normalization::epoch_ms_to_iso;

#[derive(Debug, Deserialize)]
struct StatsJson {
    range: Option<RangeData>,
    tokens: Option<TokenData>,
    models: Option<Vec<ModelStatEntry>>,
}

#[derive(Debug, Deserialize)]
struct RangeData {
    from: Option<i64>,
    to: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct TokenData {
    input: Option<i64>,
    output: Option<i64>,
    reasoning: Option<i64>,
    cache: Option<CacheData>,
}

#[derive(Debug, Deserialize)]
struct CacheData {
    read: Option<i64>,
    write: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ModelStatEntry {
    model: Option<ModelMeta>,
    tokens: Option<TokenData>,
}

#[derive(Debug, Deserialize)]
struct ModelMeta {
    id: Option<String>,
}

pub fn parse_opencode_stats_json(json_str: &str) -> Result<LocalTokens, String> {
    let stats: StatsJson = serde_json::from_str(json_str)
        .map_err(|e| format!("Falha ao interpretar JSON de opencode stats: {}", e))?;

    let from_ms = stats.range.as_ref().and_then(|r| r.from).unwrap_or(0);
    let to_ms = stats.range.as_ref().and_then(|r| r.to).unwrap_or(0);

    let range_start = epoch_ms_to_iso(from_ms).unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let range_end = epoch_ms_to_iso(to_ms).unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    let (input, output, reasoning, cache_read, cache_write, total) = if let Some(t) = stats.tokens {
        let inp = t.input.unwrap_or(0);
        let out = t.output.unwrap_or(0);
        let reas = t.reasoning.unwrap_or(0);
        let cread = t.cache.as_ref().and_then(|c| c.read).unwrap_or(0);
        let cwrite = t.cache.as_ref().and_then(|c| c.write).unwrap_or(0);
        let tot = inp + out + reas + cread + cwrite;
        (
            Some(inp),
            Some(out),
            Some(reas),
            Some(cread),
            Some(cwrite),
            Some(tot),
        )
    } else {
        (None, None, None, None, None, None)
    };

    let mut model_usages = Vec::new();
    if let Some(entries) = stats.models {
        for entry in entries {
            let model_id = entry
                .model
                .as_ref()
                .and_then(|m| m.id.clone())
                .unwrap_or_else(|| "desconhecido".to_string());

            let model_tot = entry.tokens.as_ref().map(|t| {
                let inp = t.input.unwrap_or(0);
                let out = t.output.unwrap_or(0);
                let reas = t.reasoning.unwrap_or(0);
                let cread = t.cache.as_ref().and_then(|c| c.read).unwrap_or(0);
                let cwrite = t.cache.as_ref().and_then(|c| c.write).unwrap_or(0);
                inp + out + reas + cread + cwrite
            });

            model_usages.push(ModelTokenUsage {
                model_id,
                tokens: model_tot,
            });
        }
    }

    Ok(LocalTokens {
        schema_version: 1,
        origin: "opencode_stats".to_string(),
        range_start,
        range_end,
        input,
        output,
        reasoning,
        cache_read,
        cache_write,
        total,
        models: model_usages,
        completeness: "complete".to_string(),
    })
}

pub async fn query_opencode_local_tokens(range: &str) -> Result<LocalTokens, String> {
    let days_arg = match range {
        "today" => "0",
        "7d" => "7",
        "30d" => "30",
        _ => "7",
    };

    let output = Command::new("opencode")
        .args(["stats", "--days", days_arg, "--cost", "--models", "--json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Falha ao executar 'opencode stats': {}", e))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("'opencode stats' retornou erro: {}", err_msg));
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    parse_opencode_stats_json(&stdout_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_opencode_stats_json() {
        let fixture = include_str!("../../../../tests/fixtures/opencode_stats_output.json");
        let parsed = parse_opencode_stats_json(fixture).expect("Should parse");

        assert_eq!(parsed.schema_version, 1);
        assert_eq!(parsed.origin, "opencode_stats");
        assert_eq!(parsed.input, Some(184075));
        assert_eq!(parsed.output, Some(2543));
        assert_eq!(parsed.reasoning, Some(4616));
        assert_eq!(parsed.cache_read, Some(1796173));
        assert_eq!(parsed.cache_write, Some(0));
        assert_eq!(parsed.total, Some(184075 + 2543 + 4616 + 1796173));
        assert_eq!(parsed.models.len(), 1);
        assert_eq!(parsed.models[0].model_id, "antigravity-gemini-3.8-flash");
    }
}
