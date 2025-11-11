# Colony Message Skill

You are working in a multi-agent colony. This skill helps you communicate effectively with other agents using the colony messaging system.

## Available Messaging Commands

You have access to a messaging helper script: `./colony_message.sh`

### Check Your Messages

Read messages sent to you by other agents:

```bash
./colony_message.sh read
```

This shows:
- Direct messages sent to you
- Broadcast messages sent to all agents
- Message timestamps and sender information

### Send a Message to a Specific Agent

```bash
./colony_message.sh send <agent-id> "Your message here"
```

Example:
```bash
./colony_message.sh send backend-1 "I've completed the frontend components. Ready for API integration."
```

### Broadcast to All Agents

```bash
./colony_message.sh send all "Your message here"
```

Example:
```bash
./colony_message.sh send all "Found a critical bug in the authentication module. All agents should review their auth-related code."
```

### List All Agents in the Colony

```bash
./colony_message.sh list-agents
```

This shows all agent IDs you can message.

## Best Practices

### 1. Check Messages Regularly

Periodically check for messages throughout your work:

```bash
./colony_message.sh read
```

**When to check:**
- Before starting a major task
- After completing significant work
- When blocked or needing help
- Every 10-15 minutes during active development

### 2. Announce Your Work

Let other agents know what you're working on to avoid duplicate effort:

```bash
./colony_message.sh send all "Starting work on user authentication endpoints"
```

### 3. Request Help When Blocked

Don't hesitate to ask for assistance:

```bash
./colony_message.sh send frontend-1 "Need help understanding the UI component structure. Can you provide guidance?"
```

### 4. Share Important Findings

Keep the team informed of discoveries:

```bash
./colony_message.sh send all "Discovered performance issue in database queries. Documented in PERFORMANCE.md"
```

### 5. Coordinate on Shared Resources

Communicate before modifying shared files or critical components:

```bash
./colony_message.sh send all "About to refactor the authentication module. Please don't modify auth code for the next 30 minutes."
```

### 6. Report Completion

Let others know when you finish tasks:

```bash
./colony_message.sh send all "Completed API endpoint implementation. Ready for integration testing."
```

## Message Examples by Situation

### Starting a Task

```bash
./colony_message.sh send all "Starting work on [specific task]. Will update when complete."
```

### Asking for Code Review

```bash
./colony_message.sh send review-agent "Pushed changes to feature/auth-improvements. Can you review?"
```

### Reporting a Bug

```bash
./colony_message.sh send all "Found bug in payment processing: [description]. Investigating now."
```

### Requesting Information

```bash
./colony_message.sh send backend-1 "What's the status of the database migration? Frontend needs it for testing."
```

### Sharing Documentation

```bash
./colony_message.sh send all "Created API documentation in docs/api.md. Please review your endpoints."
```

### Warning About Breaking Changes

```bash
./colony_message.sh send all "BREAKING CHANGE: Renamed User model to Account. Update your imports."
```

### Coordinating Deployments

```bash
./colony_message.sh send devops-1 "Backend ready for deployment. All tests passing."
```

## Message Format Tips

**Be Clear and Concise:**
- State the purpose immediately
- Include relevant context
- Specify what action (if any) is needed

**Good:** "Backend API endpoints complete. Ready for frontend integration. Check /api/v1/users"

**Bad:** "Done with some stuff. Let me know if you need anything."

**Include Locations:**
- File paths for code changes
- Issue/PR numbers
- Documentation references

**Example:** "Fixed authentication bug in src/auth/login.js:42. See commit abc123."

**Use Priority Indicators:**
- URGENT: for blocking issues
- FYI: for information only
- QUESTION: when you need a response
- COMPLETE: when finishing tasks

**Example:** "URGENT: Production database connection failing. Need immediate help."

## Common Workflows

### Collaborative Development

1. Announce what you're working on
2. Check messages for conflicts
3. Update others on progress
4. Request reviews before merging
5. Announce completion

### Getting Unblocked

1. Identify who can help
2. Send targeted message with context
3. Check for response regularly
4. Thank them and report resolution

### Coordinating Complex Changes

1. Broadcast intent to make changes
2. Wait for acknowledgment
3. Keep others updated on progress
4. Notify when safe to proceed
5. Share outcomes and documentation

## Troubleshooting

### No Messages Appearing?

Check if the messaging system is set up:
```bash
ls -la .colony/messages/
```

### Can't Send Messages?

Verify the script is executable:
```bash
chmod +x ./colony_message.sh
```

### Agent Not Listed?

Make sure you're in the correct working directory:
```bash
pwd
cat colony.yml
```

## Integration with Your Workflow

As you work, integrate messaging naturally:

1. **Before starting:** Check messages and announce your plan
2. **During work:** Share findings and request help as needed
3. **After completing:** Broadcast completion and next steps
4. **When blocked:** Immediately reach out for assistance

## Remember

- Messages are asynchronous - don't expect instant replies
- Other agents are working concurrently
- Clear communication prevents duplicate work and conflicts
- Use messages to maintain team awareness and coordination
- Regular communication builds effective multi-agent collaboration

## Quick Reference

| Action | Command |
|--------|---------|
| Read messages | `./colony_message.sh read` |
| Send to agent | `./colony_message.sh send <agent-id> "message"` |
| Broadcast | `./colony_message.sh send all "message"` |
| List agents | `./colony_message.sh list-agents` |

Now you're ready to communicate effectively within the colony!
