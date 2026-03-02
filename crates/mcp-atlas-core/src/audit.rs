//! Structured audit logging for tool calls (Phase 4). No PII; machine-parseable.

use serde_json::Value;
use sha2::{Digest, Sha256};

/// Hash of tool arguments for audit (deterministic, no PII). Same args => same hash.
pub fn params_hash(args: &Value) -> String {
    let mut hasher = Sha256::new();
    hasher.update(serde_json::to_string(args).unwrap_or_default().as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Implementations emit one structured event per tool call.
pub trait AuditLogger: Send + Sync {
    fn log_tool_call(&self, tool: &str, params_hash: &str, status: &str, latency_ms: u64);
}

/// Writes one JSON line per tool call to stderr. Fields: event, tool, params_hash, status, latency_ms.
#[derive(Debug, Default)]
pub struct StderrAuditLogger;

impl AuditLogger for StderrAuditLogger {
    fn log_tool_call(&self, tool: &str, params_hash: &str, status: &str, latency_ms: u64) {
        let line = serde_json::json!({
            "event": "tool_call",
            "tool": tool,
            "params_hash": params_hash,
            "status": status,
            "latency_ms": latency_ms
        });
        eprintln!("{}", line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn params_hash_deterministic() {
        let args = serde_json::json!({ "query": "k8s", "limit": 10 });
        let h1 = params_hash(&args);
        let h2 = params_hash(&args);
        assert_eq!(h1, h2);
        assert!(!h1.is_empty());
    }
}
