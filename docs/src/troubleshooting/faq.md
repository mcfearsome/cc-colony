# Frequently Asked Questions

## General

### What is Colony?

Colony is a multi-agent orchestration system for Claude Code. It allows you to run multiple Claude Code instances in parallel, each in their own isolated environment, with shared state management and coordination.

### Why use Colony?

- **Parallel Development**: Multiple agents work simultaneously
- **Specialization**: Each agent focuses on specific tasks
- **No Conflicts**: Git worktrees prevent merge conflicts
- **Coordin**ation**: Shared state enables agent communication
- **Scalable**: Add more agents as needed

### What are the system requirements?

- Rust 1.70+ (for building)
- tmux 2.0+
- Git
- Claude Code CLI
- Linux or macOS (Windows via WSL)

## Configuration

### How many agents should I create?

Start with 2-3 agents. Add more based on:
- Complexity of your project
- Available system resources
- Number of parallel workstreams needed

### Can agents share code?

Yes! Agents work on separate branches but can:
- Read from any branch
- Merge changes between branches
- Share via main/master branch
- Communicate through shared state

### How do I customize agent behavior?

Use `startup_prompt` in colony.yml:

```yaml
agents:
  - id: my-agent
    startup_prompt: |
      Custom instructions here
      Multiple lines supported
```

## Usage

### How do agents communicate?

Through shared state:
- **Tasks**: Assign and track work
- **Messages**: Direct communication
- **Memory**: Shared context and knowledge
- **Workflows**: Coordinated multi-step processes

### Can I stop and resume a colony?

Yes:
```bash
# Stop
colony stop

# Resume later
colony start
```

State persists in `.colony/` directory.

### How do I monitor agents?

Three ways:
1. **TUI**: `colony tui` - Real-time dashboard
2. **Status**: `colony status` - Quick check
3. **Logs**: `colony logs` - Detailed output

## State Management

### Where is state stored?

In `.colony/state/` directory:
- Git repository (JSONL files)
- SQLite database (for queries)
- Automatically synced

### How do I backup state?

```bash
# State is git-backed
cd .colony/state
git push origin main

# Or copy the directory
cp -r .colony/state/ backup/
```

### Can multiple people use the same colony?

Yes! Use git to sync:
```bash
# Person A
colony state push

# Person B
colony state pull
```

## Templates

### What templates are available?

Five built-in templates:
- code-reviewer
- security-auditor
- test-engineer
- documentation-writer
- data-analyst

### Can I create custom templates?

Yes! Create in `.colony/templates/`:

```yaml
name: my-template
version: 1.0.0
description: My custom template

agent:
  role: My Role
  focus: My focus area
  model: claude-sonnet-4-20250514
```

### Do I need to use templates?

No. Templates are optional. You can configure agents manually.

## Workflows

### What are workflows?

Multi-step processes with dependencies:
- Define steps
- Specify agent assignments
- Set dependencies
- Configure retry policies

### How do I create a workflow?

Create YAML file in `.colony/workflows/`:

```yaml
name: my-workflow
steps:
  - id: step1
    agent: agent-1
    action: do_something
  - id: step2
    agent: agent-2
    depends_on: [step1]
```

## Troubleshooting

### Agent is stuck or not responding

```bash
# Check logs
colony logs agent-id

# Restart agent
colony stop agent-id
colony start  # Restarts all agents
```

### Git worktree conflicts

Each agent has its own branch. Conflicts only occur when merging. Resolve like normal git conflicts.

### Performance issues

- Use faster Claude models (Haiku)
- Reduce number of agents
- Check system resources
- Monitor with metrics

## Advanced

### Can I use different Claude models per agent?

Yes:

```yaml
agents:
  - id: simple-agent
    model: claude-3-5-haiku-20241022  # Fast
  - id: complex-agent
    model: claude-opus-4-20250514     # Powerful
```

### Can I add custom MCP servers?

Yes:

```yaml
agents:
  - id: my-agent
    mcp_servers:
      custom-tool:
        command: node
        args: [server.js]
        env:
          API_KEY: ${API_KEY}
```

### How do I integrate with CI/CD?

```bash
# In CI pipeline
colony init --template test-engineer
colony start --no-attach
colony workflow run test-pipeline
colony logs --json > results.json
```

### Can I run colony in Docker?

Theoretically yes, but requires:
- tmux in container
- Git repository mounted
- Claude Code CLI available

(Docker support is experimental)

## Contributing

### How can I contribute?

- Report bugs on GitHub
- Submit pull requests
- Write documentation
- Share templates
- Help others in issues

### Where is the source code?

GitHub: [yourusername/cc-colony](https://github.com/yourusername/cc-colony)

## Getting More Help

- [Troubleshooting Guide](./common-issues.md)
- [Documentation](../introduction.md)
- [GitHub Issues](https://github.com/yourusername/cc-colony/issues)
- [Discord Community](https://discord.gg/colony) (coming soon)
