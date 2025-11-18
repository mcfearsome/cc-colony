# Templates

Templates provide reusable agent configurations for common roles and tasks.

## What are Templates?

Templates are pre-configured agent setups that include:
- **Role definition**: What the agent does
- **Focus areas**: Specific responsibilities
- **Model selection**: Which Claude model to use
- **Startup prompts**: Initial instructions
- **MCP server configs**: Tool integrations (optional)

## Built-in Templates

Colony includes 5 built-in templates:

### code-reviewer
Reviews code for quality, best practices, and potential issues.

**Focus**:
- Code quality and style
- Best practices adherence
- Performance considerations
- Maintainability

**Use case**: Automated code review before merging

### security-auditor
OWASP Top 10 focused security auditing.

**Focus**:
- Security vulnerabilities
- Authentication/authorization issues
- Input validation
- Dependency vulnerabilities

**Use case**: Security scanning and vulnerability detection

### test-engineer
Automated testing and QA specialist.

**Focus**:
- Unit test coverage
- Integration testing
- Test quality and maintainability
- Edge case identification

**Use case**: Writing comprehensive test suites

### documentation-writer
Technical documentation specialist.

**Focus**:
- API documentation
- User guides
- Code comments
- README files

**Use case**: Maintaining up-to-date documentation

### data-analyst
Data analysis and insights agent.

**Focus**:
- Data processing
- Statistical analysis
- Visualization
- Insights generation

**Use case**: Analyzing data and generating reports

## Using Templates

### List Available Templates

```bash
# List built-in templates
colony template builtin

# List installed templates
colony template list
```

### Install a Template

```bash
# Install a built-in template
colony template install code-reviewer

# View template details
colony template show code-reviewer
```

### Use in Configuration

```yaml
agents:
  - id: reviewer
    worktree_branch: review/auto
    template: code-reviewer  # Use installed template
```

### Override Template Settings

You can override any template setting:

```yaml
agents:
  - id: reviewer
    worktree_branch: review/auto
    template: code-reviewer

    # Override the startup prompt
    startup_prompt: |
      Review code with extra focus on:
      - TypeScript type safety
      - React best practices
      - Performance optimization
```

## Template Structure

Templates are stored as YAML files in `.colony/templates/`:

```yaml
# .colony/templates/my-template/template.yaml
name: my-custom-template
version: 1.0.0
author: Your Name
description: Custom agent template
license: MIT

agent:
  role: Custom Role
  focus: Specific focus area
  model: claude-sonnet-4-20250514

  startup_prompt: |
    Your custom startup prompt here.

  mcp_servers:
    filesystem:
      command: npx
      args: [-y, "@modelcontextprotocol/server-filesystem", "."]

requirements:
  repo_types:
    - source
    - application
```

## Creating Custom Templates

### Step 1: Create Template Directory

```bash
mkdir -p .colony/templates/my-template
```

### Step 2: Create template.yaml

```yaml
# .colony/templates/my-template/template.yaml
name: my-template
version: 1.0.0
description: My custom agent template

agent:
  role: My Role
  focus: My focus area
  model: claude-sonnet-4-20250514

  startup_prompt: |
    Custom instructions for this agent.
```

### Step 3: Use Your Template

```yaml
agents:
  - id: my-agent
    worktree_branch: feature/work
    template: my-template
```

## Template Best Practices

### Clear Role Definition
```yaml
agent:
  role: Frontend Developer
  focus: React components, TypeScript, and styling
```

### Specific Instructions
```yaml
agent:
  startup_prompt: |
    Focus on:
    1. Component reusability
    2. Type safety with TypeScript
    3. Accessibility (WCAG 2.1 AA)
    4. Performance optimization
```

### Appropriate Model Selection
```yaml
agent:
  # Use Sonnet for most tasks (balanced performance/cost)
  model: claude-sonnet-4-20250514

  # Or Opus for complex reasoning tasks
  model: claude-opus-4-20250514

  # Or Haiku for simple, fast tasks
  model: claude-3-5-haiku-20241022
```

### Repository Type Requirements
```yaml
requirements:
  repo_types:
    - source        # Source code repositories
    - application   # Full applications
    - library       # Libraries and packages
    - documentation # Documentation projects
```

## Template Examples

### Full-Stack Developer
```yaml
name: fullstack-dev
version: 1.0.0
description: Full-stack development specialist

agent:
  role: Full-Stack Developer
  focus: End-to-end feature development
  model: claude-sonnet-4-20250514

  startup_prompt: |
    You are a full-stack developer responsible for:
    - Frontend (React/TypeScript)
    - Backend (Node.js/Python)
    - Database design and queries
    - API development
    - Testing and documentation

requirements:
  repo_types:
    - application
```

### DevOps Engineer
```yaml
name: devops-engineer
version: 1.0.0
description: DevOps and infrastructure specialist

agent:
  role: DevOps Engineer
  focus: CI/CD, deployment, and infrastructure
  model: claude-sonnet-4-20250514

  startup_prompt: |
    Focus on:
    - CI/CD pipeline configuration
    - Docker and container orchestration
    - Infrastructure as code
    - Monitoring and logging
    - Performance optimization

  mcp_servers:
    docker:
      command: npx
      args: [-y, "@modelcontextprotocol/server-docker"]

requirements:
  repo_types:
    - application
    - infrastructure
```

## Sharing Templates

### Export Template
```bash
# Template is just a directory
tar -czf my-template.tar.gz .colony/templates/my-template
```

### Import Template
```bash
# Extract to templates directory
cd .colony/templates
tar -xzf /path/to/my-template.tar.gz
```

### Share via Git
```bash
# Commit templates directory
git add .colony/templates/
git commit -m "Add custom templates"
git push
```

## Next Steps

- [Custom Templates Guide](../advanced/custom-templates.md) - Advanced template creation
- [CLI Reference](../cli/template.md) - Template commands
- [Configuration](../getting-started/configuration.md) - Using templates in config
