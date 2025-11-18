# Plugins

Plugins extend Colony functionality.

## Plugin Types

### Backend Plugins
Extend core functionality:
- Custom state backends
- Integration points
- Data processors

### UI Plugins
Enhance the TUI:
- Custom panels
- Visualizations
- Dashboards

### Tool Plugins
Add capabilities:
- MCP server integrations
- External tools
- Custom commands

## Plugin Structure

```
.colony/plugins/my-plugin/
├── plugin.yaml
└── src/
    └── main.rs
```

See [Plugin Development](../advanced/plugin-development.md) for creating plugins.
