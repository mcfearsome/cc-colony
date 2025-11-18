# Configuration

Colony uses a `colony.yml` file to configure agents, state, and behavior.

## Basic Structure

```yaml
agents:
  - id: agent-1
    worktree_branch: feature/agent-1
    startup_prompt: "Your role and instructions"

state:
  backend: git
  location: .colony/state
  schema: beads

tmux:
  session_name: colony
  shell: /bin/bash
```

## Agent Configuration

### Required Fields

```yaml
agents:
  - id: unique-agent-id          # Unique identifier
    worktree_branch: branch-name # Git branch for this agent
```

### Optional Fields

```yaml
agents:
  - id: my-agent
    worktree_branch: feature/work

    # Template to use (optional)
    template: code-reviewer

    # Startup prompt (overrides template)
    startup_prompt: |
      Your custom instructions here.
      Can be multi-line.

    # MCP servers (optional)
    mcp_servers:
      filesystem:
        command: npx
        args: [-y, "@modelcontextprotocol/server-filesystem", "/path"]

      github:
        command: npx
        args: [-y, "@modelcontextprotocol/server-github"]
        env:
          GITHUB_TOKEN: ${GITHUB_TOKEN}
```

## State Configuration

### Git Backend (Recommended)

```yaml
state:
  backend: git
  location: .colony/state
  schema: beads

  git:
    auto_commit: true
    auto_push: false
    commit_message_prefix: "[colony]"
```

### In-Memory Backend (Testing)

```yaml
state:
  backend: memory
  schema: beads
```

## Tmux Configuration

```yaml
tmux:
  # Session name (default: "colony")
  session_name: colony

  # Shell to use (default: /bin/bash)
  shell: /bin/bash

  # Window layout (default: "tiled")
  layout: tiled

  # Custom tmux configuration
  config:
    status: "on"
    mouse: "on"
```

## Template Usage

Using built-in templates:

```yaml
agents:
  - id: reviewer
    worktree_branch: review/auto
    template: code-reviewer  # Built-in template

  - id: security
    worktree_branch: security/scan
    template: security-auditor

    # Override template's startup prompt
    startup_prompt: |
      Focus specifically on OWASP Top 10 vulnerabilities.
```

List available templates:
```bash
colony template builtin
```

## Environment Variables

Colony supports environment variable substitution:

```yaml
agents:
  - id: api-dev
    worktree_branch: feature/api
    mcp_servers:
      database:
        command: npx
        args: [-y, "@modelcontextprotocol/server-postgres"]
        env:
          DATABASE_URL: ${DATABASE_URL}
          API_KEY: ${API_KEY}
```

Set environment variables before starting:
```bash
export DATABASE_URL="postgresql://..."
export API_KEY="secret"
colony start
```

## Multiple Agent Examples

### Parallel Development
```yaml
agents:
  - id: frontend
    worktree_branch: feature/ui
    startup_prompt: "Focus on React components and UI"

  - id: backend
    worktree_branch: feature/api
    startup_prompt: "Focus on API endpoints and database"

  - id: reviewer
    worktree_branch: review/all
    template: code-reviewer
```

### Testing Pipeline
```yaml
agents:
  - id: dev
    worktree_branch: feature/new-feature
    startup_prompt: "Implement the feature"

  - id: unit-tests
    worktree_branch: test/unit
    template: test-engineer
    startup_prompt: "Write unit tests"

  - id: integration-tests
    worktree_branch: test/integration
    template: test-engineer
    startup_prompt: "Write integration tests"

  - id: security
    worktree_branch: security/scan
    template: security-auditor
```

### Documentation Team
```yaml
agents:
  - id: dev
    worktree_branch: feature/work

  - id: docs
    worktree_branch: docs/auto
    template: documentation-writer
    startup_prompt: |
      Document new features as they are implemented.
      Keep docs in sync with code changes.
```

## Advanced Configuration

### Custom Logging

```yaml
logging:
  level: info  # debug, info, warn, error
  format: json # text, json
  output: .colony/logs

  filters:
    - pattern: "^(INFO|WARN)"
      destination: .colony/logs/filtered.log
```

### Workflow Integration

```yaml
workflows:
  # Path to workflow definitions
  definitions: .colony/workflows

  # Auto-start workflows on init
  auto_start:
    - code-review-pipeline
    - daily-security-scan
```

### Plugin Configuration

```yaml
plugins:
  # Plugins directory
  directory: .colony/plugins

  # Auto-enable specific plugins
  enabled:
    - metrics-collector
    - custom-notifier
```

## Configuration Validation

Validate your configuration:

```bash
# Colony automatically validates on start
colony start

# Or explicitly check
colony init --validate
```

## Next Steps

- [Agents](../concepts/agents.md) - Learn about agent concepts
- [State Management](../concepts/state.md) - Understand shared state
- [Templates](../concepts/templates.md) - Use and create templates
