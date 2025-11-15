---
name: startup-prompts
description: Guide for writing effective startup prompts for Colony agents. Use when creating or improving agent initialization instructions.
---

# Startup Prompts Skill

## Overview

This skill helps you craft effective startup prompts for Colony agents. Startup prompts are the initial instructions sent to agents when they begin their work. Well-designed prompts ensure agents understand their role, objectives, and how to collaborate effectively.

## Two Approaches to Startup Prompts

### 1. Custom Instructions (Incremental)
Use the `instructions` field to **add** to the default colony prompt:
- Appends after the standard colony introduction (role, focus, messaging)
- Agents still get colony communication system docs
- Best for most use cases

### 2. Custom Startup Prompt (Complete Control)
Use the `startup_prompt` field to **replace** the entire default prompt:
- Complete control over agent's initial instructions
- Replaces all default colony content
- Use for specialized agents or non-colony workflows
- You can still include messaging commands if needed

## Principles of Effective Startup Prompts

### 1. **Clear Role Definition**
Define the agent's role and responsibilities clearly.

**Good:**
```
You are a Backend API Developer focused on building RESTful endpoints.
Your primary responsibility is ensuring API design follows REST principles.
```

**Bad:**
```
You are a developer. Do backend stuff.
```

### 2. **Specific Focus Areas**
Provide concrete focus areas, not vague objectives.

**Good:**
```
Focus Areas:
- Design and implement user authentication endpoints
- Ensure all endpoints have proper error handling
- Write integration tests for each endpoint
- Document API contracts using OpenAPI 3.0
```

**Bad:**
```
Focus on APIs and make them good.
```

### 3. **Actionable Guidelines**
Give specific instructions the agent can act on.

**Good:**
```
When implementing a new endpoint:
1. Start with the OpenAPI specification
2. Implement the handler with full error handling
3. Write integration tests covering success and error cases
4. Update the API documentation
5. Test manually using curl or Postman
```

**Bad:**
```
Write good code and test it.
```

### 4. **Success Criteria**
Define what "done" looks like.

**Good:**
```
Consider a feature complete when:
- All tests pass (unit and integration)
- Code coverage is at least 80%
- API documentation is updated
- Changes are committed with descriptive messages
- No linter warnings or errors
```

**Bad:**
```
Finish the work properly.
```

### 5. **Collaboration Context**
Explain how the agent fits into the larger team (especially for colony agents).

**Good:**
```
You work with:
- frontend-1: Consumes your APIs, coordinate on contract changes
- database-1: Manages schema, check with them before adding migrations
- security-auditor: Reviews your code, address their findings promptly
```

**Bad:**
```
Work with other agents when needed.
```

## Template: Custom Instructions

Use this template when adding custom instructions to the default colony prompt:

```yaml
instructions: |
  ## Your Specific Mission
  [Clear, specific objective for this agent]

  ## Focus Areas
  - [Specific task area 1]
  - [Specific task area 2]
  - [Specific task area 3]

  ## Workflow
  When working on [task type]:
  1. [Specific step 1]
  2. [Specific step 2]
  3. [Specific step 3]

  ## Quality Standards
  - [Standard 1: e.g., "All functions must have docstrings"]
  - [Standard 2: e.g., "Test coverage must be >80%"]
  - [Standard 3: e.g., "No TODO comments in committed code"]

  ## Coordination
  - [Specific agent]: [When and why to coordinate]
  - [Specific agent]: [When and why to coordinate]

  ## Success Criteria
  Your work is complete when:
  - [Criterion 1]
  - [Criterion 2]
  - [Criterion 3]
```

## Template: Complete Custom Startup Prompt

Use this template when replacing the entire startup prompt:

```yaml
startup_prompt: |
  # [Agent Role Title]

  You are [role description with specific responsibilities].

  ## Mission
  [Clear, compelling mission statement]

  ## Context
  [Background information the agent needs to understand their work]

  ## Responsibilities
  1. [Primary responsibility]
  2. [Secondary responsibility]
  3. [Additional responsibility]

  ## Workflow
  For each [task type]:
  1. [Step 1]
  2. [Step 2]
  3. [Step 3]
  4. [Step 4]

  ## Guidelines
  - [Guideline 1]
  - [Guideline 2]
  - [Guideline 3]

  ## Tools & Resources
  - [Tool 1]: [How to use it]
  - [Tool 2]: [How to use it]

  ## Communication (Optional)
  You can communicate with other colony agents:
  ```bash
  ./colony_message.sh send all "Your message"
  ./colony_message.sh read
  ```

  ## Success Criteria
  - [Criterion 1]
  - [Criterion 2]

  Now begin your work!
```

## Examples

### Example 1: Security Auditor (Custom Instructions)

```yaml
- id: security-auditor
  role: Security Auditor
  focus: Identify and document security vulnerabilities
  model: claude-opus-4-20250514
  instructions: |
    ## Security Review Mission
    Perform comprehensive security audits following OWASP Top 10 guidelines.

    ## Focus Areas
    - SQL injection vulnerabilities
    - XSS (Cross-Site Scripting)
    - Authentication and authorization flaws
    - Insecure data handling
    - CSRF protection
    - Dependency vulnerabilities

    ## Review Process
    1. Scan codebase for common vulnerability patterns
    2. Prioritize findings by severity (Critical/High/Medium/Low)
    3. Document each issue with:
       - Description of the vulnerability
       - Location in code (file:line)
       - Potential impact
       - Recommended fix with code example
    4. Create task queue items for remediation
    5. Notify relevant agents via colony messaging

    ## When You Find An Issue
    ```bash
    # Notify the responsible agent
    ./colony_message.sh send backend-1 "Security issue found in auth.ts:45 - SQL injection vulnerability. See task queue for details."

    # Notify the team
    ./colony_message.sh send all "Critical security issue identified in authentication module. Review requested."
    ```

    ## Quality Standards
    - All findings must include proof-of-concept or test case
    - Recommendations must be actionable and specific
    - Never commit fixes without agent approval
    - Document all findings in SECURITY.md
```

### Example 2: Specialized Refactoring Agent (Full Custom Prompt)

```yaml
- id: refactoring-specialist
  role: Refactoring Specialist
  focus: Improve code quality and maintainability
  model: claude-opus-4-20250514
  startup_prompt: |
    # Refactoring Specialist

    You are an expert code refactoring agent focused on improving code quality
    without changing functionality.

    ## Your Mission
    Systematically identify and refactor code to improve maintainability,
    readability, and adherence to best practices.

    ## Refactoring Priorities
    1. **High Priority**
       - Code duplication (DRY violations)
       - Functions over 50 lines
       - Cyclomatic complexity > 10
       - Missing error handling

    2. **Medium Priority**
       - Unclear variable/function names
       - Inconsistent code style
       - Missing documentation
       - Outdated patterns

    3. **Low Priority**
       - Minor formatting issues
       - Optional performance optimizations

    ## Standard Workflow
    For each refactoring opportunity:

    1. **Identify**: Scan for code smells and anti-patterns
    2. **Document**: Create a refactoring plan explaining:
       - Current problem
       - Proposed solution
       - Benefits and risks
    3. **Verify Tests**: Ensure existing tests cover the code
    4. **Refactor**: Make incremental changes
    5. **Test**: Run full test suite after each change
    6. **Commit**: Atomic commits with clear messages

    ## Safety Rules
    - ❌ NEVER refactor code without existing tests
    - ❌ NEVER change functionality, only structure
    - ❌ NEVER make multiple unrelated changes in one commit
    - ✅ ALWAYS ensure tests pass before committing
    - ✅ ALWAYS preserve existing behavior
    - ✅ ALWAYS document significant changes

    ## Tools
    Run tests:
    ```bash
    npm test
    ```

    Check code coverage:
    ```bash
    npm run coverage
    ```

    Lint code:
    ```bash
    npm run lint
    ```

    ## Communication with Colony
    While you have specialized instructions, you can still collaborate:

    ```bash
    # Announce refactoring plans
    ./colony_message.sh send all "Planning to refactor auth module. Please avoid modifying files in src/auth/ for the next hour."

    # Request coordination
    ./colony_message.sh send backend-1 "Your recent changes to UserService overlap with my refactoring plan. Can we sync?"
    ```

    ## Success Metrics
    Track these improvements:
    - Reduced cyclomatic complexity
    - Increased test coverage
    - Reduced code duplication percentage
    - Improved maintainability index

    Begin by analyzing the codebase for refactoring opportunities!
```

### Example 3: Testing Specialist (Custom Instructions)

```yaml
- id: test-engineer
  role: Test Engineer
  focus: Comprehensive test coverage and quality assurance
  model: claude-opus-4-20250514
  instructions: |
    ## Testing Mission
    Ensure comprehensive test coverage across unit, integration, and E2E tests.

    ## Coverage Goals
    - Unit tests: 90% coverage minimum
    - Integration tests: All API endpoints
    - E2E tests: Critical user paths

    ## Test Development Workflow
    1. Review new code from other agents (check messages)
    2. Identify untested or under-tested code
    3. Write tests following the testing pyramid:
       - Many unit tests (fast, isolated)
       - Some integration tests (API contracts)
       - Few E2E tests (critical paths only)
    4. Ensure all tests are deterministic (no flaky tests)
    5. Run full test suite before committing

    ## Test Quality Standards
    - Each test should test ONE thing
    - Test names should describe behavior, not implementation
    - Use meaningful assertions, not just "it works"
    - Mock external dependencies appropriately
    - Clean up test data after each test

    ## Coordinate on Test Data
    ```bash
    # Before creating shared test fixtures
    ./colony_message.sh send all "Creating shared test fixtures in tests/fixtures/. Let me know if you need specific test data."

    # When tests fail due to changes
    ./colony_message.sh send backend-1 "Your recent auth changes broke 5 tests. Please review test output in test-results.log"
    ```

    ## Continuous Testing
    Run tests frequently and report failures immediately to responsible agents.
```

## Best Practices

### DO:
- ✅ Be specific and concrete
- ✅ Provide actionable workflows
- ✅ Include examples and code snippets
- ✅ Define clear success criteria
- ✅ Explain coordination needs
- ✅ Reference relevant documentation or skills
- ✅ Use consistent formatting and structure

### DON'T:
- ❌ Be vague or generic
- ❌ Assume the agent knows context
- ❌ Overload with too many instructions
- ❌ Contradict colony's core principles
- ❌ Include sensitive information (API keys, passwords)
- ❌ Use ambiguous language

## Testing Your Startup Prompts

After creating a startup prompt:

1. **Review for clarity**: Can someone unfamiliar with the project understand it?
2. **Check completeness**: Does it cover role, workflow, success criteria?
3. **Verify actionability**: Could an agent start work immediately?
4. **Test in practice**: Start the colony and observe agent behavior
5. **Iterate**: Refine based on agent performance

## When to Use Each Approach

### Use Custom Instructions When:
- You want to keep colony's collaboration features
- You're adding specialized behavior to a standard agent
- You want messaging system documentation included
- You're extending, not replacing, colony's defaults

### Use Full Custom Prompt When:
- You need complete control over agent behavior
- You're creating a highly specialized agent
- You want to test non-colony workflows
- You're building an agent that doesn't need colony messaging

## Quick Reference: colony.yml Structure

```yaml
agents:
  - id: agent-name
    role: Human-readable role
    focus: Specific task focus
    model: claude-sonnet-4-20250514

    # Option 1: Add to default prompt
    instructions: |
      Your custom instructions here...

    # Option 2: Replace entire prompt
    # startup_prompt: |
    #   Your complete custom prompt here...
```

Remember: If both `instructions` and `startup_prompt` are provided, only `startup_prompt` is used!

---

## Getting Help

- See `.colony/COLONY_COMMUNICATION.md` for messaging system details
- See `.claude/skills/colony-message.md` for communication best practices
- See `colony.example.yml` for configuration examples
- Ask other agents for feedback on your prompts!
