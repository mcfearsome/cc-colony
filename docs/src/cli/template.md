# colony template

Manage agent templates for reusable configurations.

## Synopsis

```bash
colony template <SUBCOMMAND>
```

## Subcommands

### list
List all available templates (both installed and built-in).

```bash
colony template list
```

**Example Output**:
```
Available Templates

  code-reviewer 1.0.0 (custom)
    Code quality and best practices review agent
    Role: Code Reviewer

  security-auditor 1.0.0 (custom)
    OWASP Top 10 focused security auditing agent
    Role: Security Auditor
```

### show
Show detailed information about a specific template.

```bash
colony template show <NAME>
```

**Arguments**:
- `NAME` - Template name

**Example**:
```bash
colony template show code-reviewer
```

**Output**:
```
Template: code-reviewer

Version: 1.0.0
Source: custom

Description:
  Code quality and best practices review agent

Author: Colony Community
License: MIT

Agent Configuration:
  Role: Code Reviewer
  Focus: Review code for quality, best practices, and potential issues
  Model: claude-sonnet-4-20250514

Compatible Repository Types:
  - source
  - application
```

### install
Install a built-in template to `.colony/templates/`.

```bash
colony template install <NAME>
```

**Arguments**:
- `NAME` - Built-in template name

**Example**:
```bash
colony template install code-reviewer
```

**Output**:
```
✓ Installed template: code-reviewer

Template location: .colony/templates/code-reviewer

You can now use this template when creating agents:
  colony agent create --template code-reviewer --id my-agent
```

**Error if already installed**:
```
Error: Template 'code-reviewer' is already installed
```

### builtin
List all built-in templates available for installation.

```bash
colony template builtin
```

**Example Output**:
```
Built-in Templates

  code-reviewer
    Code quality and best practices review agent
    Role: Code Reviewer

  security-auditor
    OWASP Top 10 focused security auditing agent
    Role: Security Auditor

  test-engineer
    Automated testing and QA specialist
    Role: Test Engineer

  documentation-writer
    Technical documentation specialist
    Role: Documentation Writer

  data-analyst
    Data analysis and insights agent
    Role: Data Analyst

Use 'colony template install <name>' to install a template
Use 'colony template show <name>' to see details (after installing)
```

## Template Storage

Templates are stored in:
```
.colony/templates/
  ├── code-reviewer/
  │   └── template.yaml
  ├── security-auditor/
  │   └── template.yaml
  └── custom-template/
      └── template.yaml
```

## Using Templates

### In colony.yml Configuration

```yaml
agents:
  - id: reviewer
    worktree_branch: review/auto
    template: code-reviewer

  - id: security
    worktree_branch: security/scan
    template: security-auditor
```

### With Custom Startup Prompt

```yaml
agents:
  - id: reviewer
    worktree_branch: review/auto
    template: code-reviewer
    startup_prompt: |
      Override the template's default prompt.
      Focus on TypeScript and React best practices.
```

## Creating Custom Templates

Create a custom template directory:

```bash
mkdir -p .colony/templates/my-template
```

Create `template.yaml`:

```yaml
name: my-template
version: 1.0.0
description: My custom agent template
author: Your Name
license: MIT

agent:
  role: Custom Role
  focus: Specific responsibilities
  model: claude-sonnet-4-20250514

  startup_prompt: |
    Your custom instructions here.

requirements:
  repo_types:
    - source
```

Use your custom template:

```yaml
agents:
  - id: custom-agent
    worktree_branch: feature/work
    template: my-template
```

## Built-in Template Details

### code-reviewer
- **Role**: Code Reviewer
- **Focus**: Quality, best practices, potential issues
- **Model**: claude-sonnet-4-20250514
- **Use Case**: Automated code review

### security-auditor
- **Role**: Security Auditor
- **Focus**: OWASP Top 10, vulnerabilities, security best practices
- **Model**: claude-sonnet-4-20250514
- **Use Case**: Security scanning and auditing

### test-engineer
- **Role**: Test Engineer
- **Focus**: Unit tests, integration tests, test coverage
- **Model**: claude-sonnet-4-20250514
- **Use Case**: Writing comprehensive test suites

### documentation-writer
- **Role**: Documentation Writer
- **Focus**: API docs, user guides, README files
- **Model**: claude-sonnet-4-20250514
- **Use Case**: Maintaining technical documentation

### data-analyst
- **Role**: Data Analyst
- **Focus**: Data processing, analysis, insights
- **Model**: claude-sonnet-4-20250514
- **Use Case**: Data analysis and visualization

## Examples

### Install and Use Template

```bash
# Install the code-reviewer template
colony template install code-reviewer

# Verify installation
colony template show code-reviewer

# Use in configuration
cat > colony.yml <<EOF
agents:
  - id: reviewer
    worktree_branch: review/auto
    template: code-reviewer
EOF

# Start colony
colony start
```

### List All Templates

```bash
# List built-in (not installed)
colony template builtin

# Install a few
colony template install code-reviewer
colony template install security-auditor

# List all (installed + custom)
colony template list
```

### Create Custom Template

```bash
# Create template directory
mkdir -p .colony/templates/frontend-specialist

# Create template.yaml
cat > .colony/templates/frontend-specialist/template.yaml <<EOF
name: frontend-specialist
version: 1.0.0
description: React and TypeScript frontend specialist

agent:
  role: Frontend Developer
  focus: React components, TypeScript, and styling
  model: claude-sonnet-4-20250514

  startup_prompt: |
    You are a frontend specialist focusing on:
    - React component development
    - TypeScript type safety
    - CSS-in-JS styling
    - Component testing with Jest/React Testing Library
    - Accessibility (WCAG 2.1 AA)
EOF

# List templates (will include custom)
colony template list

# Use in config
cat >> colony.yml <<EOF
  - id: frontend
    worktree_branch: feature/ui
    template: frontend-specialist
EOF
```

## Exit Codes

- `0` - Success
- `1` - Template not found
- `1` - Template already installed
- `1` - Invalid template format

## See Also

- [Templates Concept](../concepts/templates.md) - Learn about templates
- [Custom Templates Guide](../advanced/custom-templates.md) - Advanced template creation
- [Configuration](../getting-started/configuration.md) - Using templates in config
