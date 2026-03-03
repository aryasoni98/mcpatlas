//! `get_issue_context` tool — fetch structured context for a GitHub issue
//! to enable AI-assisted resolution with minimal tokens.

use std::sync::Arc;

use anyhow::{Context, Result};
use serde_json::{Value, json};

use mcp_atlas_data::github::{fetch_github_issue, parse_github_url};
use mcp_atlas_data::models::Maturity;

use super::args;
use crate::server::AppState;

/// Maximum issue number we accept (prevent absurd values).
const MAX_ISSUE_NUMBER: u64 = 1_000_000_000;

/// Maximum body length included in the compact brief (chars).
const MAX_BODY_CHARS: usize = 500;

/// Handle `get_issue_context` — fetch a GitHub issue and return a compact
/// resolution brief with CNCF project context when available.
pub async fn handle_get_issue_context(state: &Arc<AppState>, args_val: &Value) -> Result<Value> {
    let client = state
        .github_client
        .as_ref()
        .context("GitHub client not configured. Set GITHUB_TOKEN environment variable.")?;

    let repo_input = args::parse_string_arg(args_val, "repo", "");
    if repo_input.is_empty() {
        anyhow::bail!("Missing required parameter: repo");
    }
    args::validate_string_len(&repo_input, args::MAX_QUERY_LEN)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let issue_number = args_val
        .get("issue")
        .and_then(|v| v.as_u64())
        .context("Missing or invalid required parameter: issue (must be a positive integer)")?;
    if issue_number == 0 || issue_number > MAX_ISSUE_NUMBER {
        anyhow::bail!("Issue number must be between 1 and {MAX_ISSUE_NUMBER}");
    }

    let (owner, repo) = parse_repo_input(&repo_input)?;

    let issue = fetch_github_issue(client, &owner, &repo, issue_number).await?;

    let suggested_files = extract_file_paths(issue.body.as_deref().unwrap_or(""));
    let issue_type = classify_issue(&issue.labels);
    let truncated_body = truncate_body(issue.body.as_deref().unwrap_or(""), MAX_BODY_CHARS);
    let cncf_match = find_cncf_project(state, &owner, &repo);

    let mut brief = format!(
        "## {owner}/{repo}#{issue_number}\n\
         **Title:** {title}\n\
         **Type:** {issue_type} | **State:** {state}\n\
         **Labels:** {labels}\n\
         **Author:** {author} | **Created:** {created}\n\
         \n\
         ### Summary\n\
         {body}\n",
        title = issue.title,
        state = issue.state,
        labels = if issue.labels.is_empty() {
            "(none)".to_string()
        } else {
            issue.labels.join(", ")
        },
        author = issue.user.as_deref().unwrap_or("unknown"),
        created = &issue.created_at[..10.min(issue.created_at.len())],
        body = truncated_body,
    );

    if !suggested_files.is_empty() {
        brief.push_str("\n### Suggested Files\n");
        for f in &suggested_files {
            brief.push_str(&format!("- {f}\n"));
        }
    }

    if let Some((project, maturity)) = &cncf_match {
        let mat_str = match maturity {
            Some(Maturity::Graduated) => "Graduated",
            Some(Maturity::Incubating) => "Incubating",
            Some(Maturity::Sandbox) => "Sandbox",
            Some(Maturity::Archived) => "Archived",
            _ => "Unknown",
        };
        brief.push_str(&format!("\n### CNCF Project\n{project} ({mat_str})\n"));
    }

    brief.push_str(&format!("\n**Issue URL:** {}\n", issue.html_url));

    Ok(json!({
        "content": [{ "type": "text", "text": brief }],
        "_meta": {
            "repo": format!("{owner}/{repo}"),
            "issue": issue_number,
            "is_cncf_project": cncf_match.is_some(),
            "suggested_files_count": suggested_files.len(),
        }
    }))
}

/// Parse a repo input that can be "owner/repo" or a full GitHub URL.
fn parse_repo_input(input: &str) -> Result<(String, String)> {
    let trimmed = input.trim();

    if trimmed.contains("github.com") {
        let url = trimmed
            .strip_suffix("/issues")
            .or_else(|| {
                // Strip /issues/NNN suffix if the user included it
                trimmed.rfind("/issues/").map(|i| &trimmed[..i])
            })
            .unwrap_or(trimmed);
        parse_github_url(url)
            .context("Could not parse GitHub URL — expected https://github.com/owner/repo")
    } else if trimmed.contains('/') {
        let parts: Vec<&str> = trimmed.splitn(2, '/').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            Ok((parts[0].to_string(), parts[1].to_string()))
        } else {
            anyhow::bail!("Invalid repo format: expected 'owner/repo'")
        }
    } else {
        anyhow::bail!("Invalid repo format: expected 'owner/repo' or a GitHub URL")
    }
}

/// Extract file paths from issue body text using common patterns.
pub fn extract_file_paths(body: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for line in body.lines() {
        let trimmed = line.trim();
        for word in trimmed.split_whitespace() {
            let candidate = word.trim_matches(|c: char| {
                c == '`' || c == '\'' || c == '"' || c == ',' || c == '(' || c == ')'
            });
            if is_likely_file_path(candidate) && seen.insert(candidate.to_string()) {
                paths.push(candidate.to_string());
            }
        }
    }

    paths
}

/// Heuristic: a token looks like a file path if it contains a `/` and ends
/// with a known extension or contains a dotted filename component.
fn is_likely_file_path(s: &str) -> bool {
    if s.len() < 3 || s.len() > 256 {
        return false;
    }
    if !s.contains('/') && !s.contains('.') {
        return false;
    }
    if s.starts_with("http://") || s.starts_with("https://") || s.starts_with("mailto:") {
        return false;
    }

    let extensions = [
        ".rs",
        ".go",
        ".py",
        ".js",
        ".ts",
        ".tsx",
        ".jsx",
        ".java",
        ".rb",
        ".c",
        ".cpp",
        ".h",
        ".hpp",
        ".cs",
        ".swift",
        ".kt",
        ".scala",
        ".yaml",
        ".yml",
        ".json",
        ".toml",
        ".xml",
        ".html",
        ".css",
        ".scss",
        ".md",
        ".txt",
        ".sh",
        ".bash",
        ".zsh",
        ".dockerfile",
        ".proto",
        ".sql",
        ".graphql",
        ".tf",
        ".hcl",
        ".conf",
        ".cfg",
        ".ini",
        ".env",
        ".lock",
        ".mod",
        ".sum",
        ".cue",
        ".rego",
    ];
    let lower = s.to_lowercase();
    if extensions.iter().any(|ext| lower.ends_with(ext)) {
        return true;
    }

    let known_files = [
        "Dockerfile",
        "Makefile",
        "Cargo.toml",
        "go.mod",
        "package.json",
        "CMakeLists.txt",
        "Jenkinsfile",
        "Vagrantfile",
        "Rakefile",
    ];
    let last_component = s.rsplit('/').next().unwrap_or(s);
    if known_files.contains(&last_component) {
        return true;
    }

    s.contains('/') && s.contains('.')
}

/// Classify issue type from GitHub labels.
fn classify_issue(labels: &[String]) -> &'static str {
    let lower: Vec<String> = labels.iter().map(|l| l.to_lowercase()).collect();
    if lower
        .iter()
        .any(|l| l.contains("bug") || l.contains("defect"))
    {
        "bug"
    } else if lower
        .iter()
        .any(|l| l.contains("feature") || l.contains("enhancement") || l.contains("proposal"))
    {
        "feature"
    } else if lower.iter().any(|l| l.contains("doc")) {
        "docs"
    } else if lower.iter().any(|l| l.contains("test")) {
        "test"
    } else if lower
        .iter()
        .any(|l| l.contains("chore") || l.contains("cleanup") || l.contains("refactor"))
    {
        "chore"
    } else if lower
        .iter()
        .any(|l| l.contains("security") || l.contains("cve") || l.contains("vuln"))
    {
        "security"
    } else {
        "other"
    }
}

/// Truncate body text to max chars, appending "..." if truncated.
fn truncate_body(body: &str, max_chars: usize) -> String {
    let trimmed = body.trim();
    if trimmed.len() <= max_chars {
        trimmed.to_string()
    } else {
        let mut end = max_chars;
        while end > 0 && !trimmed.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &trimmed[..end])
    }
}

/// Try to match a GitHub owner/repo against loaded CNCF projects.
fn find_cncf_project(
    state: &AppState,
    owner: &str,
    repo: &str,
) -> Option<(String, Option<Maturity>)> {
    let target_lower = format!("github.com/{owner}/{repo}").to_lowercase();

    state.projects.iter().find_map(|p| {
        p.repo_url.as_ref().and_then(|url| {
            let url_lower = url.to_lowercase().trim_end_matches('/').to_string();
            if url_lower.contains(&target_lower) {
                Some((p.name.clone(), p.maturity.clone()))
            } else {
                None
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_file_paths() {
        let body = r#"
            Please update `layouts/docs/release-series.html` and
            the file i18n/en/en.toml to include CHANGELOG links.
            See https://example.com/docs for reference.
        "#;
        let paths = extract_file_paths(body);
        assert!(paths.contains(&"layouts/docs/release-series.html".to_string()));
        assert!(paths.contains(&"i18n/en/en.toml".to_string()));
        assert!(!paths.iter().any(|p| p.contains("https://")));
    }

    #[test]
    fn test_extract_file_paths_dedup() {
        let body = "Fix src/main.rs and also src/main.rs again";
        let paths = extract_file_paths(body);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "src/main.rs");
    }

    #[test]
    fn test_extract_no_paths() {
        let body = "This issue has no file references, just text.";
        let paths = extract_file_paths(body);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_classify_issue() {
        assert_eq!(classify_issue(&["kind/bug".into()]), "bug");
        assert_eq!(classify_issue(&["kind/feature".into()]), "feature");
        assert_eq!(classify_issue(&["kind/documentation".into()]), "docs");
        assert_eq!(classify_issue(&["area/testing".into()]), "test");
        assert_eq!(classify_issue(&["security".into()]), "security");
        assert_eq!(classify_issue(&["priority/p1".into()]), "other");
        assert_eq!(classify_issue(&[]), "other");
    }

    #[test]
    fn test_truncate_body() {
        let short = "Hello world";
        assert_eq!(truncate_body(short, 500), "Hello world");

        let long = "a".repeat(600);
        let truncated = truncate_body(&long, 500);
        assert!(truncated.ends_with("..."));
        assert!(truncated.len() <= 503);
    }

    #[test]
    fn test_parse_repo_input_slash_format() {
        let (owner, repo) = parse_repo_input("kubernetes/website").unwrap();
        assert_eq!(owner, "kubernetes");
        assert_eq!(repo, "website");
    }

    #[test]
    fn test_parse_repo_input_url_format() {
        let (owner, repo) = parse_repo_input("https://github.com/prometheus/prometheus").unwrap();
        assert_eq!(owner, "prometheus");
        assert_eq!(repo, "prometheus");
    }

    #[test]
    fn test_parse_repo_input_url_with_issues() {
        let (owner, repo) =
            parse_repo_input("https://github.com/kubernetes/website/issues/54739").unwrap();
        assert_eq!(owner, "kubernetes");
        assert_eq!(repo, "website");
    }

    #[test]
    fn test_parse_repo_input_invalid() {
        assert!(parse_repo_input("").is_err());
        assert!(parse_repo_input("justarepo").is_err());
    }

    #[test]
    fn test_is_likely_file_path() {
        assert!(is_likely_file_path("src/main.rs"));
        assert!(is_likely_file_path("layouts/docs/release-series.html"));
        assert!(is_likely_file_path("Cargo.toml"));
        assert!(is_likely_file_path("deploy/helm/values.yaml"));
        assert!(!is_likely_file_path("https://example.com/page"));
        assert!(!is_likely_file_path("ab"));
        assert!(!is_likely_file_path("hello"));
    }
}
