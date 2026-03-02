# Resources Reference

The server exposes resources under the `cncf://` URI scheme.

| URI | Description |
|-----|-------------|
| `cncf://landscape/overview` | Landscape-wide statistics. |
| `cncf://categories/all` | All categories and subcategories. |
| `cncf://projects/{name}` | Project detail by name (template). |
| `cncf://categories/{category}` | Category listing (template). |

Use MCP `resources/read` with the desired URI. Subscriptions are supported via `resources/subscribe`; the server tracks subscriptions (push notifications may be extended in future).
