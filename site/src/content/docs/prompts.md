# Prompts

Pre-built MCP prompts for common workflows.

## Available prompts

| Prompt | Description |
|--------|-------------|
| `evaluate_tool` | Structured analysis of a CNCF project for a given use case. |
| `plan_migration` | Migration plan between two CNCF tools. |
| `review_stack` | Gap and redundancy analysis of a cloud-native stack. |
| `onboard_contributor` | Onboarding guide for new contributors to a project. |

## Usage

Invoke these via your client’s prompt API (e.g. list prompts, then run with the required arguments). Each prompt may take project names, use case, or other parameters; see your client’s prompt schema or the server’s `prompts/list` response.

## Related

- [Resources reference](/docs/resources-reference)  `cncf://` resources used by prompts.
- [Tools reference](/docs/tools-reference)  Tools used inside prompts.
