use serde_json::{Value, json};

/// List all available MCP prompt templates.
pub fn prompts_list() -> Value {
    json!({
        "prompts": [
            {
                "name": "evaluate_tool",
                "description": "Evaluate a CNCF tool for a specific use case. Returns a structured analysis covering maturity, community health, alternatives, and case studies.",
                "arguments": [
                    { "name": "tool_name", "description": "Name of the CNCF project to evaluate", "required": true },
                    { "name": "use_case", "description": "The use case to evaluate the tool for", "required": true }
                ]
            },
            {
                "name": "plan_migration",
                "description": "Create a migration plan between two CNCF tools. Covers key differences, breaking changes, and step-by-step guidance.",
                "arguments": [
                    { "name": "from", "description": "Source project to migrate from", "required": true },
                    { "name": "to", "description": "Target project to migrate to", "required": true }
                ]
            },
            {
                "name": "review_stack",
                "description": "Review a cloud-native architecture stack. Identifies gaps, redundancies, and recommends CNCF alternatives.",
                "arguments": [
                    { "name": "stack_description", "description": "Description of the current cloud-native stack", "required": true }
                ]
            },
            {
                "name": "onboard_contributor",
                "description": "Generate an onboarding guide for a new contributor to a CNCF project. Includes project overview, setup steps, and good first issues.",
                "arguments": [
                    { "name": "project", "description": "CNCF project to contribute to", "required": true },
                    { "name": "experience_level", "description": "Contributor experience (beginner, intermediate, advanced)", "required": false }
                ]
            }
        ]
    })
}

/// Get a prompt by name, filling in the provided arguments.
pub fn get_prompt(name: &str, arguments: &Value) -> anyhow::Result<Value> {
    match name {
        "evaluate_tool" => {
            let tool_name = arguments
                .get("tool_name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing argument: tool_name"))?;
            let use_case = arguments
                .get("use_case")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing argument: use_case"))?;

            Ok(json!({
                "description": format!("Evaluate {} for: {}", tool_name, use_case),
                "messages": [
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!(
                                "Evaluate the CNCF project **{tool_name}** for the following use case: **{use_case}**.\n\n\
                                Please use the available MCP tools to gather data and provide a structured analysis covering:\n\n\
                                1. **Project Overview** — Use `get_project` to fetch project details\n\
                                2. **Maturity & Health** — Use `get_health_score` to assess project health\n\
                                3. **Alternatives** — Use `find_alternatives` to identify competing projects\n\
                                4. **Comparison** — Use `compare_projects` to compare with top alternatives\n\
                                5. **Recommendation** — Based on all data, provide a clear recommendation\n\n\
                                Be specific and data-driven in your analysis."
                            )
                        }
                    }
                ]
            }))
        }
        "plan_migration" => {
            let from = arguments
                .get("from")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing argument: from"))?;
            let to = arguments
                .get("to")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing argument: to"))?;

            Ok(json!({
                "description": format!("Migration plan: {} → {}", from, to),
                "messages": [
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!(
                                "Create a detailed migration plan from **{from}** to **{to}**.\n\n\
                                Please use the available MCP tools:\n\
                                1. Use `get_project` to get details on both {from} and {to}\n\
                                2. Use `compare_projects` to compare them side-by-side\n\
                                3. Use `get_health_score` on both to assess their current state\n\n\
                                Then provide:\n\
                                - **Key differences** between the two projects\n\
                                - **Breaking changes** to be aware of\n\
                                - **Step-by-step migration guide**\n\
                                - **Risk assessment** and mitigation strategies\n\
                                - **Estimated effort** (small/medium/large)"
                            )
                        }
                    }
                ]
            }))
        }
        "review_stack" => {
            let stack = arguments
                .get("stack_description")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing argument: stack_description"))?;

            Ok(json!({
                "description": format!("Stack review: {}", &stack[..stack.len().min(50)]),
                "messages": [
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!(
                                "Review this cloud-native architecture stack:\n\n{stack}\n\n\
                                Please use the available MCP tools:\n\
                                1. Use `search_projects` to find CNCF alternatives for each component\n\
                                2. Use `list_categories` to identify any missing layers\n\
                                3. Use `get_health_score` on mentioned projects to assess health\n\n\
                                Provide:\n\
                                - **Gap analysis** — missing components for a production-ready stack\n\
                                - **Redundancy check** — overlapping tools that could be consolidated\n\
                                - **CNCF alternatives** — graduated/incubating replacements for non-CNCF tools\n\
                                - **Overall assessment** — maturity rating of the stack"
                            )
                        }
                    }
                ]
            }))
        }
        "onboard_contributor" => {
            let project = arguments
                .get("project")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing argument: project"))?;
            let level = arguments
                .get("experience_level")
                .and_then(|v| v.as_str())
                .unwrap_or("beginner");

            Ok(json!({
                "description": format!("Onboarding guide for {} ({})", project, level),
                "messages": [
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!(
                                "Generate an onboarding guide for a **{level}** contributor who wants to contribute to **{project}**.\n\n\
                                Please use the available MCP tools:\n\
                                1. Use `get_project` to get project details and repository URL\n\
                                2. Use `get_health_score` to understand project activity level\n\
                                3. Use `list_categories` to understand where {project} fits in the CNCF landscape\n\n\
                                Provide:\n\
                                - **Project overview** — what it does and why it matters\n\
                                - **Development setup** — how to clone, build, and run tests\n\
                                - **Contribution workflow** — how to find issues, create PRs, and get reviews\n\
                                - **Architecture overview** — key components and code organization\n\
                                - **Community resources** — Slack channels, mailing lists, meetings"
                            )
                        }
                    }
                ]
            }))
        }
        _ => anyhow::bail!("Unknown prompt: {name}"),
    }
}
