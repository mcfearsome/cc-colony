# Plugin Development

Guide for developing Colony plugins.

## Plugin Manifest

```yaml
name: my-plugin
version: 1.0.0
plugin_type: backend  # backend, ui, or tool
entrypoint: src/main.rs

hooks:
  - on_agent_start
  - on_task_complete
```

## Creating a Plugin

1. Create plugin directory
2. Write plugin.yaml manifest
3. Implement plugin logic
4. Enable in colony.yml

(Detailed guide coming soon)
