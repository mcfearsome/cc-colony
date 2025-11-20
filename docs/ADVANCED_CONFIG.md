# Advanced Colony Configuration

## Enhanced colony.yml Schema

Colony supports advanced configuration for custom layouts, capability discovery, and tool management.

## Layout Configuration

Control tmux window and pane arrangement:

```yaml
layout:
  type: custom  # Options: custom, tiled, vertical, horizontal, default

  windows:
    - name: development
      panes:
        - type: agent
          agent_id: frontend-1
          size: 60%

        - type: tool
          command: "nvim"
          title: "Editor"
          size: 40%
```

## Agent Capabilities

Define what tools and MCP servers each agent can access:

```yaml
agents:
  - id: frontend-1
    capabilities:
      tools:
        - nvim
        - ollama
        - gh
      mcp_servers:
        - filesystem
        - git
      pane_tools:
        - nvim
        - ollama
```

## Global Capabilities

Available to all agents by default:

```yaml
capabilities:
  tools: [bash, git, gh, curl, jq]
  mcp_servers: [filesystem, git]
  pane_tools: [nvim, ollama, tmux]
```

## Tool Discovery

Agents receive capability information via:
1. Environment variable: `COLONY_AVAILABLE_TOOLS`
2. Startup prompt documentation
3. Query at runtime: `echo $COLONY_AVAILABLE_TOOLS`

## Future Enhancements

This system enables:
- Custom multi-window layouts
- Tool pane templates
- Capability-based security
- Dynamic tool discovery
- Resource optimization

See `colony.example.yml` for full example.
