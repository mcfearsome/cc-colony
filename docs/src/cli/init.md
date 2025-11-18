# colony init

Initialize a new colony configuration with an interactive wizard.

## Synopsis

```bash
colony init
```

## Description

The `init` command launches an interactive wizard that guides you through creating a `colony.yml` configuration file. The wizard asks questions about your desired setup and generates a complete configuration.

## Interactive Wizard

The wizard will guide you through:

1. **Agent Count** - How many agents to create (default: 2)
2. **Agent Configuration** - For each agent:
   - Agent ID (unique identifier)
   - Template selection (5 built-in templates + manual)
   - Role customization
   - Focus area customization
   - Claude model selection
   - Optional startup prompt override
3. **Colony Naming** - Optional colony name

## Template Selection

Choose from 5 built-in templates:

- **Code Reviewer** - Quality and best practices review
- **Security Auditor** - OWASP Top 10 security scanning
- **Test Engineer** - Automated testing and QA
- **Documentation Writer** - Technical documentation
- **Data Analyst** - Data analysis and insights
- **None** - Manual configuration

## Model Selection

Choose from three Claude models:

- **claude-sonnet-4-20250514** - Balanced performance (recommended)
- **claude-opus-4-20250514** - Most capable, best for complex tasks
- **claude-3-5-haiku-20241022** - Fast and efficient

## Examples

### Basic Initialization

```bash
$ colony init

Colony Configuration Wizard
──────────────────────────────────────────────────

This wizard will help you create a colony.yml configuration.
Press Ctrl+C at any time to cancel.

? How many agents do you want to create? (2)
```

### Quick Setup with Defaults

Press Enter to accept all defaults:

```bash
$ colony init
# Accept defaults throughout
✓ Created colony.yml
✓ Initialized task queue directories
```

This creates 2 agents with sensible defaults.

### Custom Configuration

Customize each agent:

```bash
$ colony init

? How many agents do you want to create? 3

Agent 1 Configuration
? Agent ID: backend
? Choose a template: None - I'll configure manually
? Agent role: Backend Engineer
? Agent focus: API endpoints and database
? Use a different Claude model? Yes
? Select Claude model: claude-opus-4-20250514 (most capable)
```

## Prerequisites

- Must be run inside a Git repository
- Git must be initialized (`git init` if needed)

## Output

After completion, the wizard:
1. Creates `colony.yml` in the current directory
2. Initializes `.colony/` directory structure
3. Displays the generated configuration
4. Shows next steps

## Generated Configuration Example

```yaml
name: my-project
agents:
  - id: backend
    role: Backend Engineer
    focus: API endpoints and database
    model: claude-opus-4-20250514
    worktree: agent/backend
  - id: frontend
    role: Frontend Developer
    focus: React components and UI
    model: claude-sonnet-4-20250514
    worktree: agent/frontend
```

## Next Steps

After initialization:

```bash
# Start the colony
colony start

# Monitor with TUI
colony tui

# Check status
colony status
```

## Troubleshooting

### "colony must be run inside a Git repository"

Initialize Git first:
```bash
git init
colony init
```

## See Also

- [Configuration Guide](../getting-started/configuration.md) - Detailed config reference
- [Quick Start](../getting-started/quick-start.md) - Get started quickly
- [Templates](../concepts/templates.md) - Learn about templates
