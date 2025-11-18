---
name: colony-config
description: Guide for creating and configuring colony.yml files. Use when setting up multi-agent colonies or modifying agent configurations.
---

# Colony Configuration Skill

## Overview

This skill helps you create and configure `colony.yml` files for multi-agent colonies. A well-designed colony configuration ensures agents work together effectively, have the right tools, and understand their roles.

## File Location

The `colony.yml` file must be in the **root of your Git repository**.

## Basic Structure

```yaml
# Optional: Colony name (defaults to directory name)
name: my-project

# Required: List of agents
agents:
  - id: agent-1
    role: Role Description
    focus: Specific task focus
    model: claude-sonnet-4-20250514
    # ... additional optional fields

  - id: agent-2
    role: Another Role
    focus: Different focus
    model: claude-opus-4-20250514

# Optional: MCP Executor configuration
executor:
  enabled: true
  agent_id: mcp-executor
  mcp_servers:
    # ... MCP server configs
  languages:
    - typescript
    - python
```

## Required Fields

### Agent Configuration

Every agent must have these fields:

```yaml
- id: unique-agent-id        # Must be unique, alphanumeric + hyphens/underscores
  role: Human Readable Role  # Descriptive role name
  focus: Specific task area  # What the agent focuses on
```

### Field Validation Rules

- **id**:
  - Must be unique across all agents
  - Only alphanumeric characters, hyphens (-), and underscores (_)
  - Cannot be empty
  - Used for directory names and messaging
  - Examples: `backend-1`, `frontend_dev`, `test-engineer`

- **role**:
  - Human-readable description
  - Examples: "Backend Engineer", "Security Auditor", "Test Specialist"

- **focus**:
  - Specific task or area of responsibility
  - Examples: "API endpoints and server logic", "React components and UI"

## Optional Fields

### Model Selection

```yaml
model: claude-opus-4-20250514
```

**Available Models:**
- `claude-opus-4-20250514` - Most capable, slower, higher cost
- `claude-sonnet-4-20250514` - Balanced, **default**
- `claude-haiku-4-20250514` - Fastest, most economical

**When to use each:**
- **Opus**: Complex reasoning, architecture decisions, security audits
- **Sonnet**: General development, most use cases
- **Haiku**: Simple tasks, testing, documentation

### Working Directory

```yaml
# Option 1: Use git worktree (recommended, default behavior)
# Agent works in .colony/worktrees/{worktree-name}
# No directory field needed

# Option 2: Custom directory
directory: ~/projects/my-backend

# Option 3: Shared worktree
worktree: shared-backend  # Multiple agents can share same worktree
```

**Working Directory Options:**

1. **Git Worktree (Default)**: Agents work in isolated worktrees
   ```yaml
   - id: backend-1
     role: Backend Engineer
     focus: API development
     # No directory/worktree specified = uses worktree named "backend-1"
   ```

2. **Custom Directory**: Agent works in a specific directory
   ```yaml
   - id: backend-1
     role: Backend Engineer
     focus: API development
     directory: ~/projects/backend-repo
   ```

3. **Shared Worktree**: Multiple agents work in same worktree
   ```yaml
   - id: reviewer-1
     role: Code Reviewer
     focus: Backend reviews
     worktree: shared-review

   - id: reviewer-2
     role: Code Reviewer
     focus: Frontend reviews
     worktree: shared-review  # Same worktree as reviewer-1
   ```

### Environment Variables

```yaml
env:
  NODE_ENV: development
  DEBUG: "api:*"
  DATABASE_URL: postgresql://localhost:5432/dev_db
```

**Use cases:**
- Development vs production settings
- Feature flags
- Debug configurations
- API keys (though consider using secrets management)
- Database connection strings

**Important:**
- Variables are shell-escaped for security
- Each agent can have different environment variables
- Variables are exported before running Claude Code

### MCP Servers

```yaml
mcp_servers:
  filesystem:
    command: npx
    args:
      - -y
      - "@modelcontextprotocol/server-filesystem"
      - /path/to/allowed/directory
    env:
      OPTIONAL_VAR: value

  git:
    command: uvx
    args:
      - mcp-server-git
      - --repository
      - /path/to/repo
```

**MCP Server Structure:**
- **Server name** (key): Identifier for the server
- **command**: Executable to run
- **args** (optional): Command-line arguments
- **env** (optional): Environment variables for the server

**Common MCP Servers:**
- `@modelcontextprotocol/server-filesystem` - File system access
- `@modelcontextprotocol/server-git` - Git operations
- `@modelcontextprotocol/server-github` - GitHub API
- `@modelcontextprotocol/server-postgres` - PostgreSQL database
- `@modelcontextprotocol/server-sqlite` - SQLite database
- `@modelcontextprotocol/server-puppeteer` - Browser automation
- `@modelcontextprotocol/server-aws` - AWS operations

### Custom Instructions

```yaml
# Option 1: Add to default colony prompt
instructions: |
  Your custom instructions here...

  These are appended after the standard colony prompt
  (role, focus, messaging system).

# Option 2: Replace entire startup prompt
startup_prompt: |
  Complete custom startup prompt...

  This replaces the entire default colony prompt.
  You have full control over the agent's initial instructions.
```

**When to use:**
- **instructions**: Add specialized behavior while keeping colony features
- **startup_prompt**: Complete control for specialized or non-colony agents

**Note:** If both are provided, only `startup_prompt` is used.

## MCP Executor Configuration

The MCP executor is a specialized agent for running complex multi-tool MCP workflows:

```yaml
executor:
  enabled: true                    # Enable/disable executor
  agent_id: mcp-executor           # Executor identifier (default: mcp-executor)
  mcp_servers:                     # Same format as agent mcp_servers
    filesystem:
      command: npx
      args:
        - -y
        - "@modelcontextprotocol/server-filesystem"
        - /tmp
  languages:                       # Supported execution languages
    - typescript                   # Requires Deno
    - python                       # Requires Python 3.8+
```

**Benefits:**
- Reduces token usage by up to 98%
- Centralizes MCP operations
- Other agents submit tasks via messaging
- Dedicated execution environment

## Complete Example

```yaml
# Production-ready colony configuration
name: my-web-app

agents:
  # Backend development
  - id: backend-api
    role: Backend API Developer
    focus: RESTful API endpoints and business logic
    model: claude-opus-4-20250514
    env:
      NODE_ENV: development
      DATABASE_URL: postgresql://localhost:5432/dev_db
      REDIS_URL: redis://localhost:6379
    mcp_servers:
      filesystem:
        command: npx
        args:
          - -y
          - "@modelcontextprotocol/server-filesystem"
          - ./backend/src
      postgres:
        command: npx
        args:
          - -y
          - "@modelcontextprotocol/server-postgres"
        env:
          POSTGRES_CONNECTION_STRING: postgresql://localhost:5432/dev_db
    instructions: |
      ## API Development Standards
      - Follow REST principles
      - All endpoints must have error handling
      - Write integration tests for each endpoint
      - Document using OpenAPI 3.0 spec

      ## Before committing:
      1. Run test suite: `npm test`
      2. Check linting: `npm run lint`
      3. Update API documentation
      4. Notify frontend team of contract changes

  # Frontend development
  - id: frontend-ui
    role: Frontend UI Developer
    focus: React components and user interface
    model: claude-sonnet-4-20250514
    env:
      NODE_ENV: development
      API_URL: http://localhost:3000
    mcp_servers:
      puppeteer:
        command: npx
        args:
          - -y
          - "@modelcontextprotocol/server-puppeteer"
    instructions: |
      ## UI Development Guidelines
      - Follow component library design system
      - Ensure accessibility (WCAG 2.1 AA)
      - Write Storybook stories for components
      - Test with Playwright for critical flows

      ## Coordinate with backend-api:
      Check for API changes before implementing new features

  # Database management
  - id: database-admin
    role: Database Administrator
    focus: Schema design and optimization
    model: claude-sonnet-4-20250514
    worktree: backend-api  # Share worktree with backend
    env:
      DATABASE_URL: postgresql://localhost:5432/dev_db
    mcp_servers:
      postgres:
        command: npx
        args:
          - -y
          - "@modelcontextprotocol/server-postgres"
        env:
          POSTGRES_CONNECTION_STRING: postgresql://localhost:5432/dev_db
    instructions: |
      ## Database Management
      - Design normalized schemas
      - Create migrations for all schema changes
      - Optimize slow queries (> 100ms)
      - Ensure proper indexing

      ## Before schema changes:
      ```bash
      ./colony_message.sh send backend-api "Planning schema change: [description]. Please review."
      ```

  # Testing and QA
  - id: test-engineer
    role: Test Engineer
    focus: Automated testing and quality assurance
    model: claude-sonnet-4-20250514
    env:
      TEST_ENV: integration
      DATABASE_URL: postgresql://localhost:5432/test_db
    instructions: |
      ## Testing Mission
      Maintain 90% test coverage across the codebase

      ## Test Types
      - Unit tests: Fast, isolated, many
      - Integration tests: API contracts, some
      - E2E tests: Critical user paths, few

      ## Workflow
      1. Monitor messages for new features
      2. Write tests before or alongside development
      3. Run full test suite before approving changes
      4. Report failures immediately to responsible agents

  # Security auditing
  - id: security-auditor
    role: Security Auditor
    focus: Security vulnerability detection
    model: claude-opus-4-20250514
    instructions: |
      ## Security Audit Mission
      Follow OWASP Top 10 guidelines

      ## Focus Areas
      - SQL injection
      - XSS vulnerabilities
      - Authentication/authorization flaws
      - Dependency vulnerabilities

      ## When you find issues:
      1. Document severity (Critical/High/Medium/Low)
      2. Provide proof-of-concept or test case
      3. Recommend specific fixes
      4. Notify responsible agent immediately

  # Code review
  - id: code-reviewer
    role: Code Reviewer
    focus: Code quality and best practices
    model: claude-opus-4-20250514
    startup_prompt: |
      # Code Review Specialist

      You are a code review expert ensuring high code quality.

      ## Review Criteria
      - Code clarity and readability
      - Proper error handling
      - Test coverage
      - Documentation completeness
      - Performance considerations
      - Security best practices

      ## Review Process
      1. Read recent commits (git log)
      2. Review changed files
      3. Check test coverage
      4. Verify documentation updates
      5. Provide constructive feedback

      ## Feedback Format
      - Be specific: Reference file and line numbers
      - Be constructive: Suggest improvements
      - Be kind: Focus on code, not the author

      ## Colony Communication
      ```bash
      ./colony_message.sh send <agent-id> "Code review feedback: [summary]"
      ```

      Begin reviewing recent changes!

# MCP Executor for complex workflows
executor:
  enabled: true
  agent_id: mcp-executor
  mcp_servers:
    filesystem:
      command: npx
      args:
        - -y
        - "@modelcontextprotocol/server-filesystem"
        - .
    postgres:
      command: npx
      args:
        - -y
        - "@modelcontextprotocol/server-postgres"
      env:
        POSTGRES_CONNECTION_STRING: postgresql://localhost:5432/dev_db
  languages:
    - typescript
    - python
```

## Design Patterns

### Pattern 1: Specialized Teams

Organize agents by functional area:

```yaml
agents:
  # Backend team
  - id: backend-api
    role: Backend API Developer
    focus: REST API endpoints

  - id: backend-workers
    role: Backend Worker Developer
    focus: Background jobs and queues

  # Frontend team
  - id: frontend-web
    role: Web Frontend Developer
    focus: Web application UI

  - id: frontend-mobile
    role: Mobile Frontend Developer
    focus: React Native mobile app

  # Infrastructure team
  - id: devops-cloud
    role: Cloud Infrastructure Engineer
    focus: AWS infrastructure and deployment

  - id: devops-cicd
    role: CI/CD Engineer
    focus: Build and deployment pipelines
```

### Pattern 2: Development Workflow

Agents for each stage of development:

```yaml
agents:
  - id: developer
    role: Feature Developer
    focus: Implement new features

  - id: tester
    role: Test Engineer
    focus: Write and run tests

  - id: reviewer
    role: Code Reviewer
    focus: Review code quality

  - id: documenter
    role: Documentation Specialist
    focus: Maintain documentation
```

### Pattern 3: Shared Worktrees

Multiple agents collaborating in same codebase:

```yaml
agents:
  - id: developer-1
    role: Developer
    focus: User authentication
    worktree: main-dev

  - id: developer-2
    role: Developer
    focus: Payment processing
    worktree: main-dev  # Same worktree

  - id: reviewer
    role: Code Reviewer
    focus: Review all changes
    worktree: main-dev  # Same worktree
```

### Pattern 4: Tool-Specific Agents

Agents with specialized MCP tools:

```yaml
agents:
  # Database specialist with DB tools
  - id: db-admin
    role: Database Administrator
    focus: Schema and query optimization
    mcp_servers:
      postgres:
        command: npx
        args: ["-y", "@modelcontextprotocol/server-postgres"]

  # DevOps specialist with cloud tools
  - id: devops
    role: DevOps Engineer
    focus: Infrastructure management
    mcp_servers:
      aws:
        command: npx
        args: ["-y", "@modelcontextprotocol/server-aws"]

  # Browser testing specialist
  - id: qa-automation
    role: QA Automation Engineer
    focus: Browser-based testing
    mcp_servers:
      puppeteer:
        command: npx
        args: ["-y", "@modelcontextprotocol/server-puppeteer"]
```

## Best Practices

### DO:
- ✅ Use descriptive agent IDs (e.g., `backend-api` not `agent1`)
- ✅ Assign clear, non-overlapping responsibilities
- ✅ Provide specific focus areas
- ✅ Configure appropriate models for task complexity
- ✅ Use shared worktrees when agents need to collaborate closely
- ✅ Set environment variables for agent-specific configuration
- ✅ Configure MCP servers based on agent needs
- ✅ Add custom instructions for specialized behaviors
- ✅ Keep the executor enabled for complex MCP workflows
- ✅ Document your configuration choices

### DON'T:
- ❌ Use generic agent IDs (e.g., `agent1`, `agent2`)
- ❌ Give agents overlapping responsibilities (causes conflicts)
- ❌ Leave focus areas vague (e.g., "do backend stuff")
- ❌ Use Opus for all agents (unnecessary cost)
- ❌ Put sensitive credentials directly in colony.yml (use env vars or secrets)
- ❌ Configure all agents with all MCP servers (only what they need)
- ❌ Create too many agents (start small, 3-5 agents)
- ❌ Forget to validate the configuration before starting

## Validation

Before starting your colony, validate the configuration:

```bash
# Check for syntax errors
colony start --dry-run

# Or manually validate
cat colony.yml | yq eval
```

**Common validation errors:**
- Duplicate agent IDs
- Invalid agent ID characters
- Missing required fields (id, role, focus)
- Empty agents list
- Invalid YAML syntax

## Configuration Workflow

1. **Plan your colony**
   - What's the project goal?
   - What roles are needed?
   - How will agents collaborate?

2. **Create colony.yml**
   ```bash
   # Start with example
   cp colony.example.yml colony.yml
   ```

3. **Configure agents**
   - Set IDs, roles, and focus areas
   - Choose appropriate models
   - Configure working directories
   - Add environment variables if needed

4. **Add MCP servers**
   - Identify tools each agent needs
   - Configure server commands and args
   - Set server-specific environment variables

5. **Customize instructions**
   - Add specialized instructions
   - Or create custom startup prompts
   - Reference relevant skills and documentation

6. **Configure executor** (optional)
   - Enable if you have complex MCP workflows
   - Configure necessary MCP servers
   - Set supported languages

7. **Validate**
   ```bash
   colony start --dry-run
   ```

8. **Start and iterate**
   ```bash
   colony start
   ```
   - Monitor agent behavior
   - Adjust configuration as needed
   - Refine instructions based on performance

## Quick Start Templates

### Minimal Colony (2 agents)

```yaml
agents:
  - id: developer
    role: Full-Stack Developer
    focus: Feature development
    model: claude-sonnet-4-20250514

  - id: reviewer
    role: Code Reviewer
    focus: Code quality and testing
    model: claude-sonnet-4-20250514
```

### Web Application Colony

```yaml
agents:
  - id: backend
    role: Backend Developer
    focus: API and server logic
    model: claude-opus-4-20250514

  - id: frontend
    role: Frontend Developer
    focus: React UI components
    model: claude-sonnet-4-20250514

  - id: database
    role: Database Engineer
    focus: Schema and queries
    model: claude-sonnet-4-20250514
    worktree: backend  # Share with backend

  - id: tester
    role: Test Engineer
    focus: Automated testing
    model: claude-sonnet-4-20250514
```

### Security-Focused Colony

```yaml
agents:
  - id: developer
    role: Developer
    focus: Feature implementation
    model: claude-sonnet-4-20250514

  - id: security-auditor
    role: Security Auditor
    focus: Vulnerability detection
    model: claude-opus-4-20250514
    instructions: |
      Review all code changes for security vulnerabilities.
      Follow OWASP Top 10 guidelines.
      Report findings immediately.

  - id: penetration-tester
    role: Penetration Tester
    focus: Security testing
    model: claude-opus-4-20250514
    mcp_servers:
      puppeteer:
        command: npx
        args: ["-y", "@modelcontextprotocol/server-puppeteer"]
```

## Troubleshooting

### Agent won't start
- Check agent ID is valid (alphanumeric, hyphens, underscores only)
- Verify required fields (id, role, focus) are present
- Check YAML syntax is valid

### MCP servers not working
- Verify command is installed and in PATH
- Check args are correct for the MCP server
- Ensure environment variables are set correctly
- Check MCP server logs in agent's .claude/logs/

### Agents conflicting
- Review focus areas for overlap
- Consider using shared worktrees if intentional collaboration
- Add coordination instructions
- Use different branches or directories

### Configuration changes not applying
- Stop colony: `colony stop`
- Restart colony: `colony start`
- Check for typos in agent IDs

## Additional Resources

- **Example file**: `colony.example.yml` - Comprehensive examples
- **Startup prompts**: `.claude/skills/startup-prompts.md` - Writing effective prompts
- **Colony messaging**: `.claude/skills/colony-message.md` - Agent communication
- **MCP executor**: `.claude/skills/mcp-executor/` - Complex workflows

---

## Getting Started

Ready to create your colony?

1. Copy the example: `cp colony.example.yml colony.yml`
2. Edit `colony.yml` with your agent configuration
3. Validate: `colony start --dry-run`
4. Launch: `colony start`

Your agents will begin working together toward your project goals!
