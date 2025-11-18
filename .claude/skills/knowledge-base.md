---
name: knowledge-base
description: Managing memory and knowledge base repositories. Use when working with information storage, research organization, or team memory systems.
---

# Knowledge Base & Memory Management Skill

## Overview

This skill helps agents working in **memory-type repositories** - spaces designed for storing, organizing, and retrieving information rather than traditional source code. Memory repositories serve as team knowledge bases, research collections, context storage, and organizational memory.

## What is a Memory Repository?

A memory repository is a Git repository optimized for knowledge storage and retrieval:

- **Purpose**: Store information, not executable code
- **Content**: Markdown notes, research findings, meeting notes, decisions, context
- **Structure**: Organized by topic, date, or domain
- **Retrieval**: Searchable, linkable, versioned
- **Collaboration**: Multiple agents contribute and consume knowledge

### Memory Repository vs. Source Repository

| Aspect | Memory Repository | Source Repository |
|--------|------------------|-------------------|
| Primary content | Markdown, docs, data | Source code |
| Main operations | Write, organize, search | Code, test, deploy |
| Success metric | Knowledge coverage | Working software |
| Versioning | Context preservation | Feature tracking |
| Quality | Clarity, searchability | Correctness, performance |

## Repository Structure

### Recommended Directory Structure

```
knowledge-base/
├── README.md                    # Repository guide and index
├── index/                       # Topic indices and navigation
│   ├── by-topic.md
│   ├── by-date.md
│   └── by-author.md
├── research/                    # Research findings
│   ├── market-analysis/
│   ├── technical-exploration/
│   └── competitive-intel/
├── decisions/                   # Decision records (ADRs)
│   ├── 2024-11-architecture-choice.md
│   ├── 2024-11-tech-stack.md
│   └── README.md
├── meetings/                    # Meeting notes
│   ├── 2024/11/
│   └── README.md
├── context/                     # Project context
│   ├── team-members.md
│   ├── goals.md
│   └── constraints.md
├── how-to/                      # Guides and procedures
│   ├── onboarding.md
│   ├── deployment.md
│   └── troubleshooting.md
└── archive/                     # Older, less-relevant content
    └── 2023/
```

### Alternative Structures

**By Domain** (for multi-project teams):
```
knowledge-base/
├── product-alpha/
├── product-beta/
└── shared/
```

**By Function** (for specialized teams):
```
knowledge-base/
├── engineering/
├── design/
├── product/
└── cross-functional/
```

## Core Operations

### 1. Adding New Knowledge

When you learn something valuable, capture it:

#### Creating a New Note

```bash
# Create note file
mkdir -p research/authentication
cat > research/authentication/oauth-comparison.md <<EOF
# OAuth Provider Comparison

**Date**: $(date +%Y-%m-%d)
**Author**: agent-name
**Tags**: #authentication #oauth #research

## Overview
[Brief summary of the research]

## Findings
[Detailed findings]

## Recommendations
[Actionable recommendations]

## References
- [Link 1]
- [Link 2]
EOF

# Add to index
echo "- [OAuth Provider Comparison](research/authentication/oauth-comparison.md) - $(date +%Y-%m-%d)" >> index/by-topic.md

# Commit
git add research/authentication/oauth-comparison.md index/by-topic.md
git commit -m "Add OAuth provider comparison research"
```

#### Note Template

```markdown
# [Title]

**Date**: YYYY-MM-DD
**Author**: agent-id
**Tags**: #tag1 #tag2 #tag3
**Status**: draft | review | final

## Context
Why this information was captured

## Summary
Brief overview (2-3 sentences)

## Details
In-depth information

## Related
- [Related Note 1](link)
- [Related Note 2](link)

## Next Actions
- [ ] Action item 1
- [ ] Action item 2
```

### 2. Organizing Knowledge

#### Tagging System

Use consistent tags for discoverability:

```markdown
#research - Research findings
#decision - Decision records
#meeting - Meeting notes
#how-to - Guides and procedures
#context - Background information
#question - Open questions
#idea - Ideas and proposals
```

#### Linking Notes

Create connections between related information:

```markdown
## Related Information
- See also: [Authentication Architecture](../decisions/auth-architecture.md)
- Background: [Team Security Requirements](../context/security-requirements.md)
- Next steps: [OAuth Implementation Plan](../how-to/implement-oauth.md)
```

#### Maintaining Indices

Keep topic and date indices up-to-date:

**index/by-topic.md**:
```markdown
# Knowledge Base Index - By Topic

## Authentication
- [OAuth Provider Comparison](../research/authentication/oauth-comparison.md)
- [JWT vs Session Tokens](../research/authentication/jwt-vs-sessions.md)

## Database
- [PostgreSQL vs MySQL](../research/database/postgres-vs-mysql.md)
- [Indexing Strategies](../how-to/database-indexing.md)
```

**index/by-date.md**:
```markdown
# Knowledge Base Index - By Date

## 2024-11

### Week of Nov 11-17
- 2024-11-15: [OAuth Provider Comparison](../research/authentication/oauth-comparison.md)
- 2024-11-14: [Team Meeting Notes](../meetings/2024/11/team-meeting-2024-11-14.md)
```

### 3. Decision Records (ADRs)

Use Architecture Decision Records for important decisions:

#### ADR Template

```markdown
# ADR-001: [Decision Title]

**Date**: YYYY-MM-DD
**Status**: proposed | accepted | deprecated | superseded
**Deciders**: agent-1, agent-2
**Tags**: #decision #architecture

## Context and Problem Statement
What is the issue we're trying to address?

## Decision Drivers
- Driver 1
- Driver 2
- Driver 3

## Considered Options
1. Option A
2. Option B
3. Option C

## Decision Outcome
Chosen option: "[option]"

### Reasoning
Why we chose this option:
- Reason 1
- Reason 2

### Positive Consequences
- Benefit 1
- Benefit 2

### Negative Consequences
- Trade-off 1
- Trade-off 2

## Implementation Notes
How to implement this decision

## Links
- Related ADRs
- External references
```

### 4. Meeting Notes

Capture meeting discussions and decisions:

#### Meeting Notes Template

```markdown
# [Meeting Name] - YYYY-MM-DD

**Date**: YYYY-MM-DD
**Attendees**: agent-1, agent-2, agent-3
**Tags**: #meeting #team

## Agenda
1. Topic 1
2. Topic 2
3. Topic 3

## Discussion

### Topic 1
[Summary of discussion]

**Decisions:**
- Decision 1
- Decision 2

**Action Items:**
- [ ] @agent-1: Action item 1 (Due: YYYY-MM-DD)
- [ ] @agent-2: Action item 2 (Due: YYYY-MM-DD)

### Topic 2
[Summary of discussion]

## Next Meeting
- Date: YYYY-MM-DD
- Focus: [Topics to discuss]
```

### 5. Searching Knowledge

#### Using Git Grep

```bash
# Search for content
git grep "authentication"

# Search in specific directory
git grep "OAuth" research/

# Search with context
git grep -C 3 "decision"

# Search for tags
git grep "#research"
```

#### Using Claude Code Search

Use the Grep tool to search across all notes:
- Search by keyword
- Search by tag
- Search by date range (in filenames)

### 6. Maintaining Quality

#### Review Checklist

Before committing notes:
- [ ] Title is clear and descriptive
- [ ] Date and author are present
- [ ] Tags are appropriate and consistent
- [ ] Content is well-structured
- [ ] Links work (for internal references)
- [ ] Index is updated
- [ ] Commit message is descriptive

#### Archiving Old Content

Move outdated information to archive/:

```bash
# Move old content
mv research/legacy-system archive/2023/
git add -A
git commit -m "Archive legacy system research (no longer relevant)"
```

#### Deduplication

When you find duplicate information:
1. Identify the best/most complete version
2. Add cross-references from others
3. Consider merging if both have unique content
4. Archive duplicates

## Agent Workflows

### Research Agent Workflow

```
1. Receive research assignment
2. Conduct research (web search, documentation)
3. Create note from template
4. Fill in findings and analysis
5. Add relevant tags
6. Link to related notes
7. Update index
8. Commit with descriptive message
9. Notify team via colony messaging
```

### Knowledge Curator Workflow

```
1. Review recent notes
2. Check for consistency
3. Verify links work
4. Update indices
5. Suggest improvements
6. Archive outdated content
7. Identify gaps in knowledge
```

### Question Answerer Workflow

```
1. Receive question
2. Search knowledge base
3. If answer exists:
   - Provide link to relevant notes
   - Summarize key points
4. If answer doesn't exist:
   - Research the answer
   - Create new note
   - Update index
   - Respond with findings
```

## Colony Collaboration

### Announcing New Knowledge

```bash
# After adding important research
./colony_message.sh send all "Added OAuth provider comparison research: research/authentication/oauth-comparison.md"
```

### Requesting Research

```bash
# Ask for specific research
./colony_message.sh send researcher "Can you research PostgreSQL vs MongoDB for our use case? Add findings to research/database/"
```

### Coordinating on Topics

```bash
# Avoid duplicate work
./colony_message.sh send all "Working on API versioning strategies research for the next hour"
```

## Best Practices

### DO:
- ✅ Use consistent formatting and templates
- ✅ Tag notes appropriately
- ✅ Link related information
- ✅ Update indices when adding content
- ✅ Write clear, scannable summaries
- ✅ Include dates and authors
- ✅ Commit frequently with good messages
- ✅ Archive outdated information
- ✅ Make information discoverable

### DON'T:
- ❌ Store large binary files (use Git LFS if needed)
- ❌ Duplicate information unnecessarily
- ❌ Use vague titles ("Notes", "Misc")
- ❌ Forget to update indices
- ❌ Leave notes in draft state indefinitely
- ❌ Include sensitive credentials (use references)
- ❌ Create deeply nested structures (max 3-4 levels)
- ❌ Write walls of text without structure

## Example Use Cases

### Use Case 1: Technical Research Repository

```yaml
repository:
  repo_type: memory
  purpose: "Technical research and exploration for Project Alpha"
  context: "Store research findings, proof-of-concepts, and technology evaluations"

agents:
  - id: researcher
    role: Technical Researcher
    focus: Evaluate new technologies and patterns
    instructions: |
      Research assigned technologies and create comprehensive notes:
      1. Create note from template in research/
      2. Include pros/cons/use-cases
      3. Add code examples if relevant
      4. Update research index
      5. Notify team of findings

  - id: curator
    role: Knowledge Curator
    focus: Organize and maintain research quality
    instructions: |
      Maintain research repository quality:
      - Review new notes for completeness
      - Update indices and cross-references
      - Archive outdated research
      - Identify knowledge gaps
```

### Use Case 2: Team Decision Log

```yaml
repository:
  repo_type: memory
  purpose: "Record of all technical and product decisions"
  context: "Searchable history of why we made specific choices"

agents:
  - id: decision-recorder
    role: Decision Recorder
    focus: Document decisions as ADRs
    instructions: |
      When team makes a decision:
      1. Create ADR from template
      2. Capture context and options
      3. Document reasoning
      4. Note consequences
      5. Link to related decisions
      6. Update decision index
```

### Use Case 3: Meeting Notes Repository

```yaml
repository:
  repo_type: memory
  purpose: "Archive of all team meetings and discussions"
  context: "Searchable meeting history with action items"

agents:
  - id: note-taker
    role: Meeting Notes Agent
    focus: Capture meeting discussions and decisions
    instructions: |
      During meetings:
      1. Create note from template
      2. Capture agenda and attendees
      3. Summarize discussions
      4. List decisions made
      5. Track action items with owners
      6. Link related meetings
```

## Metrics and Health

### Repository Health Indicators

**Good Health:**
- Recent commits (active use)
- Updated indices
- Clear organization
- Consistent formatting
- Working internal links
- Appropriate archive usage

**Needs Attention:**
- No commits in weeks
- Outdated indices
- Orphaned notes (not in index)
- Inconsistent formatting
- Broken links
- Everything in root directory

### Quality Metrics

Track these periodically:
- Number of notes added per week
- Index coverage (% of notes in indices)
- Average note age
- Link density (notes with >2 links)
- Tag usage distribution

## Troubleshooting

### "I can't find information on X"

1. Search by keyword: `git grep "keyword"`
2. Check topic index: `cat index/by-topic.md`
3. Check date index (if you know when it was added)
4. Search by tag: `git grep "#tag"`
5. If still not found, research and add it

### "The repository is disorganized"

1. Review directory structure
2. Move misplaced notes to proper locations
3. Rebuild indices from scratch if needed
4. Add missing dates/authors/tags
5. Archive old content
6. Create organization guidelines

### "Notes are inconsistent"

1. Choose standard templates
2. Document templates in README.md
3. Review and update existing notes
4. Enforce templates for new notes

## Integration with Source Repositories

Memory repositories often complement source code repositories:

```yaml
# Source repository colony.yml
repository:
  repo_type: source
  purpose: "Main application codebase"

agents:
  - id: developer
    role: Developer
    focus: Feature implementation
    instructions: |
      When making architectural decisions:
      1. Document in source repo decision log OR
      2. Reference knowledge base ADR
      3. Link decision in code comments

# Knowledge base colony.yml
repository:
  repo_type: memory
  purpose: "Team knowledge and decisions"

agents:
  - id: documenter
    role: Documentation Agent
    focus: Capture learnings from development
    instructions: |
      Monitor development progress and capture:
      - New patterns discovered
      - Problems solved
      - Technologies evaluated
      - Decisions made
```

## Quick Reference

### Common Commands

```bash
# Create new note
mkdir -p [category]
cat > [category]/[title].md <<EOF
[template]
EOF

# Add to index
echo "- [[Title]]([path]) - $(date +%Y-%m-%d)" >> index/by-topic.md

# Search
git grep "keyword"
git grep "#tag"

# Commit
git add .
git commit -m "Add [description]"

# Find recent notes
git log --since="1 week ago" --oneline --name-only

# List all tags
git grep -h "^**Tags**:" | sort | uniq
```

---

## Getting Started

1. **Understand your repository purpose** - Check `colony.yml` repository config
2. **Review directory structure** - See README.md for organization
3. **Find templates** - Look for note templates in root or docs/
4. **Check indices** - Review `index/` to understand what's already documented
5. **Start contributing** - Use templates, tag consistently, update indices

Your knowledge base is only as good as the information you put into it. Capture valuable knowledge early and often!
