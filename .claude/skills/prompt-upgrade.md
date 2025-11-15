---
name: prompt-upgrade
description: Analyze and improve prompts for clarity, effectiveness, and actionability. Use when refining user prompts, agent instructions, or system prompts.
---

# Prompt Upgrade Skill

## Overview

This skill helps you analyze and improve prompts to make them more effective, clear, and actionable. Whether you're refining a user's request, improving agent startup instructions, or optimizing system prompts, this skill provides a systematic approach to prompt enhancement.

## The Prompt Quality Framework

Great prompts share these characteristics:

1. **Clarity** - Unambiguous, easy to understand
2. **Specificity** - Concrete details, not vague generalities
3. **Actionability** - Clear steps or outcomes
4. **Context** - Necessary background information
5. **Constraints** - Boundaries and requirements
6. **Success Criteria** - Definition of "done"

## Prompt Analysis Process

When you receive a prompt to upgrade, follow these steps:

### Step 1: Identify Prompt Type

**Question Prompts**
- Asking for information or explanation
- Example: "How does authentication work?"

**Task Prompts**
- Requesting action or implementation
- Example: "Add user authentication"

**Instruction Prompts**
- Providing ongoing guidance (like agent startup prompts)
- Example: "You are a backend developer..."

**Creative Prompts**
- Generating content or ideas
- Example: "Write a blog post about..."

### Step 2: Analyze Current State

Review the prompt against the quality framework:

```
Original Prompt Analysis:
- Clarity: [High/Medium/Low] - [Why?]
- Specificity: [High/Medium/Low] - [Why?]
- Actionability: [High/Medium/Low] - [Why?]
- Context: [Sufficient/Partial/Missing] - [Why?]
- Constraints: [Present/Absent] - [List them]
- Success Criteria: [Clear/Vague/Missing] - [Why?]
```

### Step 3: Identify Improvements

List specific improvements needed:
- Missing information
- Vague language to clarify
- Ambiguous requirements to specify
- Assumed context to make explicit
- Success criteria to define

### Step 4: Generate Upgraded Prompt

Create an improved version following best practices for the prompt type.

### Step 5: Explain Improvements

Document what changed and why it's better.

## Upgrade Patterns

### Pattern 1: Vague → Specific

**Before:**
```
Make the app faster
```

**After:**
```
Optimize the application's performance:

Goals:
- Reduce initial page load time from 3s to <1s
- Decrease API response time from 500ms to <200ms
- Improve Time to Interactive (TTI) score to >90

Focus Areas:
1. Enable code splitting for routes
2. Implement lazy loading for images
3. Add Redis caching for frequently-accessed data
4. Optimize database queries (add indexes, reduce N+1)

Success Criteria:
- Lighthouse performance score >90
- All Core Web Vitals in "Good" range
- Verified with real-world testing on 4G connection
```

**Improvements:**
- ✅ Quantified performance goals
- ✅ Specific optimization techniques
- ✅ Measurable success criteria
- ✅ Clear scope

### Pattern 2: Missing Context → Context-Rich

**Before:**
```
Fix the bug in the login function
```

**After:**
```
Fix authentication bug in user login flow

Context:
- Bug location: src/auth/login.ts, handleLogin() function
- Symptom: Users can't log in with valid credentials
- Error message: "Invalid session token" (appears in browser console)
- Affects: All users, started after deployment on 2024-01-15
- Stack trace: [paste relevant stack trace]

Expected Behavior:
- User enters valid email/password
- System validates credentials
- User receives JWT token
- User is redirected to dashboard

Current Behavior:
- User enters valid email/password
- System validates credentials
- Session token generation fails
- User sees "Invalid session token" error

Investigation Done:
- Verified credentials are valid
- Confirmed database connection works
- Checked JWT_SECRET environment variable is set

Please investigate and fix the token generation issue.
```

**Improvements:**
- ✅ Specific location provided
- ✅ Clear symptoms described
- ✅ Expected vs actual behavior
- ✅ Reproduction context
- ✅ Investigation already completed

### Pattern 3: No Success Criteria → Clear Success

**Before:**
```
Write tests for the user module
```

**After:**
```
Write comprehensive tests for the user module

Scope:
- File: src/modules/user.ts
- Functions to test: createUser, updateUser, deleteUser, getUserById

Test Requirements:
1. Unit tests for each function
2. Test happy path (success cases)
3. Test error cases (validation, not found, duplicates)
4. Test edge cases (empty strings, null values, SQL injection attempts)
5. Mock database calls (use jest.mock)
6. Ensure no external dependencies in unit tests

Coverage Goals:
- Line coverage: >90%
- Branch coverage: >85%
- All error paths covered

Test File Location:
- src/modules/user.test.ts

Success Criteria:
- All tests pass
- Coverage goals met
- Tests are deterministic (no random failures)
- Tests run in <2 seconds
- No console warnings or errors
```

**Improvements:**
- ✅ Specific scope defined
- ✅ Clear test requirements
- ✅ Coverage goals quantified
- ✅ Success criteria measurable

### Pattern 4: Ambiguous → Unambiguous

**Before:**
```
Improve the code quality
```

**After:**
```
Refactor authentication module to improve code quality

Specific Issues to Address:
1. Function length: auth.ts has 3 functions >100 lines (target: <50 lines)
2. Cyclomatic complexity: handleLogin has complexity of 15 (target: <10)
3. Code duplication: Token generation logic duplicated in 3 places
4. Missing error handling: 7 functions don't handle promise rejections
5. Inconsistent naming: Mix of camelCase and snake_case

Refactoring Plan:
1. Extract large functions into smaller, focused functions
2. Reduce complexity by extracting conditional logic
3. Create shared utility function for token generation
4. Add try-catch blocks and proper error handling
5. Standardize on camelCase naming convention

Constraints:
- Don't change public API (maintain backward compatibility)
- Ensure all existing tests still pass
- Add tests for any new functions created

Success Criteria:
- All functions <50 lines
- Max cyclomatic complexity <10
- No code duplication (DRY principle)
- All promise rejections handled
- Consistent naming convention
- All tests pass
- Code coverage maintained or improved
```

**Improvements:**
- ✅ Specific quality issues identified
- ✅ Measurable targets provided
- ✅ Concrete refactoring steps
- ✅ Constraints explicitly stated
- ✅ Clear success criteria

### Pattern 5: Single-Step → Workflow

**Before:**
```
Create a REST API for users
```

**After:**
```
Design and implement a RESTful API for user management

Workflow:
1. API Design
   - Define endpoints (GET/POST/PUT/DELETE /api/users)
   - Design request/response schemas
   - Document with OpenAPI 3.0 specification
   - Review design with team

2. Database Schema
   - Create users table migration
   - Add necessary indexes
   - Consider foreign key constraints
   - Run migration in dev environment

3. Implementation
   - Implement route handlers
   - Add input validation (use Joi or Zod)
   - Implement business logic
   - Add proper error handling
   - Log important operations

4. Testing
   - Write integration tests for each endpoint
   - Test success cases
   - Test error cases (400, 401, 404, 500)
   - Test edge cases
   - Verify >80% coverage

5. Documentation
   - Complete OpenAPI spec
   - Add code comments
   - Write API usage guide
   - Include curl examples

6. Security
   - Add authentication middleware
   - Implement authorization (RBAC)
   - Validate and sanitize inputs
   - Rate limiting
   - CORS configuration

Deliverables:
- OpenAPI spec: docs/api/users.yaml
- Implementation: src/routes/users.ts
- Tests: src/routes/users.test.ts
- Documentation: docs/api/users.md

Success Criteria:
- All CRUD operations working
- All tests passing (>80% coverage)
- API documented with examples
- Security measures implemented
- Code reviewed and approved
```

**Improvements:**
- ✅ Multi-step workflow defined
- ✅ Each step has specific tasks
- ✅ Deliverables clearly listed
- ✅ Security considerations included
- ✅ Comprehensive success criteria

## Prompt Upgrade Templates

### Template: Question Prompt

**Before:**
```
How does [thing] work?
```

**After:**
```
Explain how [thing] works in [specific context]

Please include:
1. High-level overview
2. Key components and their roles
3. Step-by-step process flow
4. Common use cases
5. Important considerations or gotchas

Context:
- [Relevant background information]
- [Current understanding level]
- [Why this information is needed]

Format:
- Start with a brief summary (2-3 sentences)
- Use examples to illustrate concepts
- Include diagrams if helpful
- Provide code examples where relevant
```

### Template: Task Prompt

**Before:**
```
[Do something]
```

**After:**
```
[Do something specific]

Context:
- [Why this is needed]
- [Current state]
- [Relevant constraints]

Requirements:
1. [Specific requirement 1]
2. [Specific requirement 2]
3. [Specific requirement 3]

Approach:
1. [Step 1]
2. [Step 2]
3. [Step 3]

Success Criteria:
- [Measurable outcome 1]
- [Measurable outcome 2]
- [Measurable outcome 3]

Deliverables:
- [Specific output 1]
- [Specific output 2]
```

### Template: Instruction Prompt (Agent/System)

**Before:**
```
You are a [role]. [Do task].
```

**After:**
```
# Role: [Specific Role Title]

You are [detailed role description with specific responsibilities].

## Mission
[Clear, compelling mission statement]

## Responsibilities
1. [Primary responsibility with specifics]
2. [Secondary responsibility with specifics]
3. [Additional responsibility with specifics]

## Workflow
For each [task type]:
1. [Specific step 1]
2. [Specific step 2]
3. [Specific step 3]

## Guidelines
- [Guideline 1 with rationale]
- [Guideline 2 with rationale]
- [Guideline 3 with rationale]

## Success Criteria
- [Measurable criterion 1]
- [Measurable criterion 2]
- [Measurable criterion 3]

## Resources
- [Tool/resource 1]: [How to use]
- [Tool/resource 2]: [How to use]

[Call to action to begin work]
```

## Common Prompt Problems and Fixes

### Problem: Assumed Knowledge

**Symptom:** Prompt assumes context not explicitly stated

**Fix:** Make all assumptions explicit

**Example:**
```
Before: "Update the config"
After: "Update the database connection config in config/database.ts to use PostgreSQL instead of MySQL. Keep existing connection pool settings."
```

### Problem: Multiple Interpretations

**Symptom:** Prompt can be understood in different ways

**Fix:** Add specificity to eliminate ambiguity

**Example:**
```
Before: "Make the button bigger"
After: "Increase the primary CTA button size from 40px to 48px height, maintaining the 16px horizontal padding. Apply to all primary buttons using the .btn-primary class."
```

### Problem: No Measurable Outcome

**Symptom:** Can't determine if task is complete

**Fix:** Add quantifiable success criteria

**Example:**
```
Before: "Improve performance"
After: "Reduce page load time to under 2 seconds (measured by Lighthouse on desktop, 4G throttling)"
```

### Problem: Too Abstract

**Symptom:** Prompt uses vague, high-level language

**Fix:** Provide concrete examples and specifics

**Example:**
```
Before: "Follow best practices"
After: "Follow these best practices:
- Use async/await instead of callbacks
- Handle all promise rejections with try-catch
- Validate inputs with Zod schemas
- Log errors with Winston logger
- Write JSDoc comments for public functions"
```

### Problem: Missing Constraints

**Symptom:** No boundaries or limitations specified

**Fix:** Explicitly state what should NOT be done

**Example:**
```
Before: "Refactor the auth module"
After: "Refactor the auth module

Constraints:
- Don't change the public API (breaking changes not allowed)
- Maintain backward compatibility with v1 tokens
- Don't introduce new dependencies
- Keep the same error response format
- All existing tests must pass unchanged"
```

### Problem: No Priority

**Symptom:** Everything seems equally important

**Fix:** Specify priorities or ordering

**Example:**
```
Before: "Fix these issues: bug A, bug B, feature C, refactor D"
After: "Address these items in priority order:

1. CRITICAL: Fix bug A - authentication failing (blocks all users)
2. HIGH: Fix bug B - data corruption in edge case (affects 5% users)
3. MEDIUM: Feature C - add export functionality (requested feature)
4. LOW: Refactor D - improve code quality (nice to have)

Start with #1, then proceed in order if time permits."
```

## Upgrade Checklist

When upgrading a prompt, verify:

- [ ] **Role/Audience**: Clear who should respond or act
- [ ] **Context**: Sufficient background provided
- [ ] **Goal**: Explicit objective stated
- [ ] **Scope**: Boundaries clearly defined
- [ ] **Specificity**: Concrete details, not vague terms
- [ ] **Actionability**: Clear next steps or actions
- [ ] **Constraints**: Limitations and requirements stated
- [ ] **Success Criteria**: Measurable outcomes defined
- [ ] **Examples**: Concrete examples provided where helpful
- [ ] **Format**: Structured and easy to parse

## Prompt Upgrade Examples

### Example 1: User Request

**Original:**
```
I need help with the database
```

**Analysis:**
- Clarity: LOW - "help" is vague
- Specificity: LOW - which database? what issue?
- Context: MISSING - no background
- Actionability: LOW - unclear what to do

**Upgraded:**
```
I need help debugging a PostgreSQL connection issue

Problem:
- Application can't connect to PostgreSQL database
- Error message: "ECONNREFUSED 127.0.0.1:5432"
- Started happening after upgrading from PostgreSQL 13 to 14

Environment:
- PostgreSQL 14.2
- Node.js application using pg library v8.7.3
- Running on Ubuntu 22.04
- Connection string: postgresql://localhost:5432/mydb

What I've Tried:
- Verified PostgreSQL is running (systemctl status postgresql shows active)
- Checked port 5432 is listening (netstat confirms)
- Verified credentials are correct
- Can connect using psql command-line tool

Expected Outcome:
- Node.js application successfully connects to PostgreSQL
- Understand root cause of connection failure
- Prevent similar issues in future

Relevant Files:
- Database config: src/config/database.ts
- Connection code: src/lib/db.ts
```

**Improvements:**
- ✅ Specific problem identified
- ✅ Error message included
- ✅ Environment details provided
- ✅ Troubleshooting already done
- ✅ Clear expected outcome

### Example 2: Code Review Request

**Original:**
```
Review my code
```

**Analysis:**
- Clarity: LOW - which code? what to review for?
- Specificity: LOW - no scope
- Context: MISSING
- Success Criteria: MISSING

**Upgraded:**
```
Review user authentication implementation for security and best practices

Code Location:
- PR #123: https://github.com/org/repo/pull/123
- Main files: src/auth/login.ts, src/auth/register.ts, src/middleware/auth.ts

What Changed:
- Implemented JWT-based authentication
- Added password hashing with bcrypt
- Created authentication middleware
- Added rate limiting on login endpoint

Review Focus Areas:
1. Security: Are there any vulnerabilities?
   - Password storage security
   - JWT token handling
   - Session management
   - Input validation

2. Best Practices:
   - Error handling completeness
   - Code organization
   - Type safety
   - Test coverage

3. Performance:
   - Bcrypt rounds appropriate?
   - Any unnecessary database queries?
   - Caching opportunities?

4. Documentation:
   - Are functions documented?
   - Is the auth flow clear?

Specific Questions:
- Is bcrypt work factor of 10 appropriate for our use case?
- Should we implement refresh tokens?
- Is the rate limiting configuration (5 attempts/15min) reasonable?

Success Criteria:
- Security review passed
- Best practices followed
- All questions answered
- Actionable feedback provided
```

**Improvements:**
- ✅ Specific code location provided
- ✅ Clear review focus areas
- ✅ Specific questions asked
- ✅ Success criteria defined

### Example 3: Agent Startup Instructions

**Original:**
```
You are a developer. Write good code.
```

**Analysis:**
- Clarity: MEDIUM - basic but vague
- Specificity: LOW - "good code" is subjective
- Actionability: LOW - no concrete guidance
- Success Criteria: MISSING

**Upgraded:**
```
# Backend API Developer

You are a backend API developer specializing in Node.js and Express.js.
Your role is to design, implement, and maintain RESTful APIs.

## Primary Responsibilities

1. **API Development**
   - Design RESTful endpoints following REST principles
   - Implement request handlers with proper error handling
   - Validate inputs using Zod schemas
   - Structure responses consistently

2. **Database Operations**
   - Write efficient SQL queries
   - Create and manage migrations
   - Ensure proper indexing
   - Optimize query performance

3. **Testing**
   - Write integration tests for all endpoints
   - Maintain >80% code coverage
   - Test happy paths and error cases
   - Ensure tests are deterministic

## Development Workflow

For each new endpoint:
1. Design the API contract (request/response schema)
2. Document in OpenAPI spec (docs/api/openapi.yaml)
3. Implement the route handler
4. Add input validation
5. Implement business logic
6. Add error handling
7. Write integration tests
8. Test manually with curl/Postman
9. Update documentation
10. Submit for code review

## Code Quality Standards

- **Functions**: Max 50 lines, single responsibility
- **Error Handling**: All async functions use try-catch
- **Validation**: All inputs validated with Zod
- **Logging**: Important operations logged with Winston
- **Documentation**: Public functions have JSDoc comments
- **Types**: Full TypeScript coverage, no 'any' types
- **Security**: Inputs sanitized, SQL injection prevented

## Before Committing

Run this checklist:
```bash
npm run lint        # No errors
npm test           # All tests pass
npm run coverage   # Coverage >80%
npm run build      # Builds successfully
```

## Coordination

You work with:
- **frontend-dev**: Coordinate on API contracts before implementation
- **database-admin**: Consult on schema changes and complex queries
- **security-auditor**: Address security findings promptly
- **test-engineer**: Collaborate on integration test strategy

## Communication

Announce major changes:
```bash
./colony_message.sh send all "Adding new /api/users endpoint. API contract in docs/api/openapi.yaml"
```

Request review:
```bash
./colony_message.sh send security-auditor "New auth endpoints ready for security review"
```

## Success Metrics

Your work is successful when:
- All endpoints have >80% test coverage
- API response time <200ms (95th percentile)
- No security vulnerabilities in code
- API documentation is up-to-date
- Code passes all quality checks
- Zero breaking changes without migration guide

Begin by checking for messages from teammates, then start on your assigned work!
```

**Improvements:**
- ✅ Specific role and responsibilities
- ✅ Concrete workflow steps
- ✅ Measurable quality standards
- ✅ Collaboration guidance
- ✅ Clear success metrics

## Self-Improvement Loop

After upgrading prompts, evaluate results:

1. **Test the upgraded prompt**
   - Use it in actual work
   - Observe the results
   - Note any confusion or ambiguity

2. **Collect feedback**
   - Did it produce expected results?
   - Was anything unclear?
   - What could be better?

3. **Refine further**
   - Address identified issues
   - Add clarifications
   - Improve examples

4. **Document patterns**
   - Note what worked well
   - Save successful templates
   - Build a pattern library

## Quick Reference: Before and After

| Weak Prompt | Strong Prompt |
|-------------|---------------|
| "Fix the bug" | "Fix authentication bug in login.ts:45 causing 'Invalid token' error for valid users" |
| "Add tests" | "Add unit tests for user.ts with >90% coverage, testing happy paths and error cases" |
| "Improve performance" | "Reduce API response time from 500ms to <200ms by adding Redis caching and query optimization" |
| "Write documentation" | "Document the authentication flow in docs/auth.md with sequence diagrams and code examples" |
| "Make it secure" | "Add input validation (Zod), SQL injection prevention (parameterized queries), and rate limiting (5 req/min)" |
| "Clean up code" | "Refactor auth.ts: split 150-line function into focused functions <50 lines each, maintaining test coverage" |

## Key Takeaways

1. **Specificity beats generality** - Concrete details enable action
2. **Context is crucial** - Background information prevents assumptions
3. **Measurable outcomes** - Define success criteria explicitly
4. **Show, don't tell** - Examples clarify expectations
5. **Structure helps** - Organized prompts are easier to process
6. **Iterate** - First draft is rarely perfect, refine based on results

---

## Using This Skill

When someone asks you to improve a prompt:

1. Analyze the original against the quality framework
2. Identify specific weaknesses
3. Apply relevant upgrade patterns
4. Generate improved version
5. Explain what changed and why

Remember: The best prompt is the one that gets the job done effectively. Sometimes simple is better than complex. Always optimize for clarity and actionability.
