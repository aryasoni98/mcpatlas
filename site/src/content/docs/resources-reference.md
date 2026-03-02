# Resources reference

The server exposes MCP resources under the `cncf://` URI scheme.

## URI templates

| URI | Description |
|-----|-------------|
| `cncf://landscape/overview` | Landscape-wide statistics. |
| `cncf://categories/all` | All categories and subcategories. |
| `cncf://projects/{name}` | Project detail by name (template). |
| `cncf://categories/{category}` | Category listing (template). |

## Usage

Clients request these URIs via the MCP `resources/read` method. The server resolves the template, loads the data, and returns the content (e.g. JSON or text). Use project and category names that exist in the landscape (e.g. from `list_categories` or `search_projects`).

## Related

- [Tools reference](/docs/tools-reference)  `get_project`, `list_categories`, and search tools.
- [Prompts](/docs/prompts)  Pre-built prompts that use tools and resources.
