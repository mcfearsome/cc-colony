---
name: jq-json
description: JSON processing with jq for parsing, filtering, and transforming data
---

# jq for Colony Agents

## Purpose

Master jq for JSON parsing, filtering, transforming, and building - essential for API integrations and data processing.

## Basic Usage

### Accessing Fields

```bash
# Get single field
echo '{"name":"John","age":30}' | jq '.name'
# Output: "John"

# Get nested field
echo '{"user":{"name":"John"}}' | jq '.user.name'

# Raw output (no quotes)
echo '{"name":"John"}' | jq -r '.name'
# Output: John
```

### Array Operations

```bash
# Get array length
echo '[1,2,3,4,5]' | jq 'length'
# Output: 5

# Get first element
echo '[1,2,3]' | jq '.[0]'

# Get last element
echo '[1,2,3]' | jq '.[-1]'

# Slice array
echo '[1,2,3,4,5]' | jq '.[1:3]'
# Output: [2,3]

# Iterate array
echo '[{"name":"John"},{"name":"Jane"}]' | jq '.[]'
# Outputs each object

# Get specific field from each
echo '[{"name":"John","age":30},{"name":"Jane","age":25}]' | jq '.[].name'
# Output: "John" "Jane"
```

## Filtering

### Select Objects

```bash
# Filter by condition
echo '[{"name":"John","active":true},{"name":"Jane","active":false}]' | \
  jq '.[] | select(.active == true)'

# Multiple conditions
jq '.[] | select(.age > 18 and .active == true)' users.json

# Check if field exists
jq '.[] | select(.email != null)' users.json

# String matching
jq '.[] | select(.name | contains("John"))' users.json
```

### Map and Transform

```bash
# Transform objects
jq '.[] | {id, fullName: .name}' users.json

# Add fields
jq '.[] | . + {processed: true}' items.json

# Remove fields
jq '.[] | del(.password)' users.json

# Map array
jq 'map({id, name})' users.json
```

## Building JSON

### Create Objects

```bash
# Simple object
jq -n '{name: "John", age: 30}'

# From variables
NAME="John"
AGE=30
jq -n --arg name "$NAME" --argjson age "$AGE" '{name: $name, age: $age}'

# Complex nested object
jq -n \
  --arg name "John" \
  --arg email "john@example.com" \
  '{
    user: {name: $name, email: $email},
    created: now | todate,
    active: true
  }'
```

### Create Arrays

```bash
# Array from multiple values
jq -n --arg a "one" --arg b "two" '[$a, $b]'

# Array from input
jq -s '.' file1.json file2.json file3.json
# Slurps all files into single array
```

## Advanced Filtering

### Complex Queries

```bash
# Multiple filters with pipe
jq '.users[] | select(.age > 18) | select(.active == true) | .name' data.json

# Group by field
jq 'group_by(.category)' items.json

# Sort
jq 'sort_by(.age)' users.json

# Reverse sort
jq 'sort_by(.age) | reverse' users.json

# Unique values
jq 'unique_by(.email)' users.json
```

### Aggregation

```bash
# Count
jq '[.[] | select(.active == true)] | length' users.json

# Sum
jq '[.[].price] | add' products.json

# Average
jq '[.[].age] | add / length' users.json

# Min/Max
jq '[.[].age] | min' users.json
jq '[.[].age] | max' users.json
```

## String Operations

### Manipulation

```bash
# Uppercase/lowercase
jq '.name | ascii_upcase' user.json
jq '.name | ascii_downcase' user.json

# Split string
echo '{"path":"/usr/local/bin"}' | jq '.path | split("/")'

# Join array
echo '{"parts":["a","b","c"]}' | jq '.parts | join("-")'

# Test regex
jq '.email | test("@gmail\\.com$")' user.json

# Extract with regex
jq '.text | match("([0-9]+)").captures[0].string' data.json
```

## Conditional Logic

### If-Then-Else

```bash
# Simple conditional
jq '.[] | if .age >= 18 then "adult" else "minor" end' users.json

# Multiple conditions
jq '.[] |
  if .score >= 90 then "A"
  elif .score >= 80 then "B"
  elif .score >= 70 then "C"
  else "F"
  end' scores.json
```

### Try-Catch

```bash
# Handle missing fields gracefully
jq '.[] | {id, email: (.email // "no-email")}' users.json

# Alternative operator
jq '.name // .username // "anonymous"' user.json
```

## Combining with Other Tools

### curl + jq

```bash
# Fetch and extract
curl -s https://api.github.com/users/octocat | jq -r '.name'

# POST with jq-built payload
payload=$(jq -n --arg title "Bug fix" '{title: $title, body: "Fixed the bug"}')
curl -X POST https://api.example.com/issues \
  -H "Content-Type: application/json" \
  -d "$payload"
```

### grep + jq

```bash
# Find files, process with jq
find . -name "package.json" -exec jq -r '.name' {} \;

# Search logs, parse JSON
grep "ERROR" app.log | jq -r '.message'
```

### Process Line by Line

```bash
# Stream processing (one JSON object per line)
cat data.jsonl | jq -c '.user.email'

# Build JSONL from array
jq -c '.[]' array.json > output.jsonl
```

## Colony Integration Patterns

### Message Queue Processing

```bash
#!/bin/bash
# Process messages as JSON

./colony_message.sh read | \
while read -r line; do
    # Extract message data (if messages were JSON)
    if echo "$line" | jq -e . >/dev/null 2>&1; then
        sender=$(echo "$line" | jq -r '.from')
        content=$(echo "$line" | jq -r '.content')
        echo "From $sender: $content"
    fi
done
```

### Config File Processing

```bash
# Read colony.yml as JSON (if converted)
# Extract agent IDs
yq -o=json eval colony.yml | jq -r '.agents[].id'

# Or process JSON configs
jq -r '.agents[] | "\(.id): \(.role)"' config.json
```

### API Response Analysis

```bash
#!/bin/bash
# Analyze API responses and report

response=$(curl -s https://api.example.com/analytics)

# Extract metrics
total=$(echo "$response" | jq '.total')
active=$(echo "$response" | jq '.active')
errors=$(echo "$response" | jq '.errors')

# Report
./colony_message.sh send orchestrator "API Analytics:
Total: $total
Active: $active
Errors: $errors"
```

## Advanced Techniques

### Recursive Descent

```bash
# Find all values for a key anywhere in JSON
jq '.. | .name? // empty' data.json

# Find all objects with specific field
jq '.. | objects | select(has("email"))' data.json
```

### Custom Functions

```bash
# Define reusable functions
jq 'def full_name: "\(.first) \(.last)";
    .users[] | {id, name: full_name}' data.json
```

### Reduce (Fold)

```bash
# Custom aggregation
jq 'reduce .[] as $item (0; . + $item.price)' products.json

# Build object from array
jq 'reduce .[] as $item ({}; . + {($item.key): $item.value})' data.json
```

## Output Formats

### Compact vs Pretty

```bash
# Compact (one line)
jq -c '.' data.json

# Pretty (default, indented)
jq '.' data.json

# Custom indentation
jq --indent 4 '.' data.json
```

### CSV Output

```bash
# Convert JSON to CSV
jq -r '.[] | [.id, .name, .email] | @csv' users.json

# With headers
echo "id,name,email"
jq -r '.[] | [.id, .name, .email] | @csv' users.json
```

### TSV Output

```bash
# Tab-separated values
jq -r '.[] | [.id, .name] | @tsv' users.json
```

## Error Handling

### Check if Valid JSON

```bash
if jq -e . input.json >/dev/null 2>&1; then
    echo "Valid JSON"
else
    echo "Invalid JSON"
fi
```

### Handle Missing Fields

```bash
# Use // for default values
jq '.email // "no-email"' user.json

# Check if field exists
jq 'has("email")' user.json

# Get field or null
jq '.email?' user.json
```

## Performance Tips

**1. Use `-c` for large outputs**
```bash
# Compact output is faster and smaller
jq -c '.[]' large-array.json
```

**2. Stream large files**
```bash
# Process without loading entire file into memory
jq -c '.[]' large-file.json | while read -r item; do
    process "$item"
done
```

**3. Limit output early**
```bash
# Get first 10 items only
jq '.items[:10]' data.json
```

## Common Recipes

### Merge JSON Files

```bash
# Merge multiple JSON objects
jq -s '.[0] * .[1]' file1.json file2.json

# Merge arrays
jq -s 'add' file1.json file2.json
```

### Flatten Nested Structure

```bash
# Flatten one level
jq '[.[] | .items[]]' nested.json

# Flatten completely
jq '.. | scalars' deeply-nested.json
```

### Find and Replace

```bash
# Update field values
jq '(.[] | select(.id == 123) | .status) = "active"' data.json

# Update all occurrences
jq '(.[] | .status) = "active"' data.json
```

### Validate Schema

```bash
# Check if all required fields exist
jq '.[] | select(has("id") and has("name") and has("email") | not) | .id' users.json
# Outputs IDs of invalid entries
```

## jq in Scripts

```bash
#!/bin/bash
# Example script using jq

INPUT_FILE="data.json"

# Validate input
if ! jq -e . "$INPUT_FILE" >/dev/null 2>&1; then
    echo "Error: Invalid JSON in $INPUT_FILE"
    exit 1
fi

# Process
active_count=$(jq '[.[] | select(.active == true)] | length' "$INPUT_FILE")
total_count=$(jq 'length' "$INPUT_FILE")

echo "Active: $active_count / $total_count"

# Transform and save
jq 'map(select(.active == true))' "$INPUT_FILE" > active-users.json

echo "Saved active users to active-users.json"
```

## Resources

- Manual: `man jq` or `jq --help`
- Online playground: https://jqplay.org/
- Tutorial: https://stedolan.github.io/jq/tutorial/
- Cookbook: https://github.com/stedolan/jq/wiki/Cookbook
