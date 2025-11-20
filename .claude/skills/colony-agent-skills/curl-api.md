---
name: curl-api
description: Using curl for API interactions, HTTP requests, and data transfer
---

# Curl for Colony Agents

## Purpose

Master curl for API interactions, testing endpoints, downloading files, and integrating with web services.

## Basic Usage

### GET Requests

```bash
# Simple GET
curl https://api.example.com/users

# GET with headers
curl -H "Authorization: Bearer TOKEN" https://api.example.com/users

# GET with query parameters
curl "https://api.example.com/search?q=query&limit=10"

# Save response to file
curl -o output.json https://api.example.com/data
```

### POST Requests

```bash
# POST JSON data
curl -X POST https://api.example.com/users \
  -H "Content-Type: application/json" \
  -d '{"name":"John","email":"john@example.com"}'

# POST from file
curl -X POST https://api.example.com/users \
  -H "Content-Type: application/json" \
  -d @data.json

# POST form data
curl -X POST https://api.example.com/login \
  -d "username=admin&password=secret"

# POST multipart form (file upload)
curl -X POST https://api.example.com/upload \
  -F "file=@document.pdf" \
  -F "description=My file"
```

### Other HTTP Methods

```bash
# PUT (update)
curl -X PUT https://api.example.com/users/123 \
  -H "Content-Type: application/json" \
  -d '{"name":"Updated Name"}'

# PATCH (partial update)
curl -X PATCH https://api.example.com/users/123 \
  -d '{"email":"new@example.com"}'

# DELETE
curl -X DELETE https://api.example.com/users/123

# HEAD (get headers only)
curl -I https://api.example.com/status
```

## Authentication

### Bearer Token

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  https://api.example.com/protected
```

### Basic Auth

```bash
# With username:password
curl -u username:password https://api.example.com/auth

# Just username (prompts for password)
curl -u username https://api.example.com/auth
```

### API Key

```bash
# In header
curl -H "X-API-Key: YOUR_KEY" https://api.example.com/data

# In query parameter
curl "https://api.example.com/data?api_key=YOUR_KEY"
```

## Response Handling

### Show Response Headers

```bash
# Headers only
curl -I https://api.example.com/status

# Headers + body
curl -i https://api.example.com/users

# Verbose (shows request + response details)
curl -v https://api.example.com/users
```

### Status Codes

```bash
# Get HTTP status code
STATUS=$(curl -o /dev/null -s -w "%{http_code}" https://api.example.com/users)
echo "Status: $STATUS"

# Check if successful
if [[ $STATUS == "200" ]]; then
    echo "Success"
else
    echo "Failed with status: $STATUS"
fi
```

### Follow Redirects

```bash
# Follow redirects automatically
curl -L https://short.url/abc123

# Limit redirects
curl -L --max-redirs 3 https://example.com
```

## Error Handling

### Fail on HTTP Errors

```bash
# Exit with error if HTTP status >= 400
curl -f https://api.example.com/users || echo "Request failed"

# Silent fail (no error output, just exit code)
curl -sS -f https://api.example.com/users
```

### Retry on Failure

```bash
# Retry up to 3 times
curl --retry 3 https://api.example.com/users

# Retry with delay
curl --retry 3 --retry-delay 2 https://api.example.com/users

# Retry only on specific errors
curl --retry 3 --retry-connrefused https://api.example.com/users
```

### Timeouts

```bash
# Connection timeout (seconds)
curl --connect-timeout 10 https://api.example.com/users

# Max time for entire operation
curl --max-time 30 https://api.example.com/large-file

# Both
curl --connect-timeout 5 --max-time 30 https://api.example.com/data
```

## Advanced Features

### Custom Headers

```bash
# Multiple headers
curl -H "Accept: application/json" \
     -H "User-Agent: Colony-Agent/1.0" \
     -H "X-Custom-Header: value" \
     https://api.example.com/data
```

### Cookies

```bash
# Send cookie
curl -b "session=abc123" https://api.example.com/profile

# Save cookies to file
curl -c cookies.txt https://api.example.com/login \
  -d "username=admin&password=secret"

# Use saved cookies
curl -b cookies.txt https://api.example.com/profile
```

### Download Files

```bash
# Download with original filename
curl -O https://example.com/file.zip

# Download with custom name
curl -o myfile.zip https://example.com/download

# Resume interrupted download
curl -C - -O https://example.com/large-file.zip

# Progress bar
curl --progress-bar -O https://example.com/file.zip
```

### Upload Files

```bash
# PUT file
curl -T file.txt https://example.com/upload

# POST file as binary
curl -X POST https://api.example.com/upload \
  --data-binary @file.pdf \
  -H "Content-Type: application/pdf"
```

## Integration with jq

### Parse JSON Response

```bash
# Extract field from JSON
curl -s https://api.example.com/user/123 | jq -r '.name'

# Filter array
curl -s https://api.example.com/users | jq '.[] | select(.active == true)'

# Transform response
curl -s https://api.example.com/users | jq '[.[] | {id, name}]'
```

### Build JSON for POST

```bash
# Simple JSON
DATA=$(jq -n --arg name "John" --arg email "john@example.com" '{name:$name, email:$email}')
curl -X POST https://api.example.com/users \
  -H "Content-Type: application/json" \
  -d "$DATA"

# From file with jq transformation
jq '.items[] | select(.status=="pending")' input.json | \
while read -r item; do
    curl -X POST https://api.example.com/process \
      -H "Content-Type: application/json" \
      -d "$item"
done
```

## Colony-Specific Patterns

### API Testing and Reporting

```bash
#!/bin/bash
# Test API endpoints and report status

endpoints=(
    "https://api.example.com/health"
    "https://api.example.com/users"
    "https://api.example.com/products"
)

results=()
for endpoint in "${endpoints[@]}"; do
    status=$(curl -o /dev/null -s -w "%{http_code}" "$endpoint")
    if [[ $status == "200" ]]; then
        results+=("✓ $endpoint")
    else
        results+=("✗ $endpoint (HTTP $status)")
    fi
done

# Report to orchestrator
message=$(printf '%s\n' "${results[@]}")
./colony_message.sh send orchestrator "API Health Check Results:
$message"
```

### Data Pipeline

```bash
#!/bin/bash
# Fetch, transform, and save data

# 1. Fetch from API
curl -s https://api.example.com/data > /tmp/api-response.json

# 2. Transform with jq
jq '.items[] | {id, name, created: .created_at}' /tmp/api-response.json > /tmp/transformed.json

# 3. Report
COUNT=$(jq length /tmp/transformed.json)
./colony_message.sh send backend-1 "Processed $COUNT items from API. See /tmp/transformed.json"
```

### Webhook Integration

```bash
#!/bin/bash
# Send webhook notification

notify_webhook() {
    local message=$1
    local webhook_url="https://hooks.example.com/webhook"

    payload=$(jq -n \
        --arg agent "$(whoami)" \
        --arg message "$message" \
        --arg timestamp "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
        '{agent: $agent, message: $message, timestamp: $timestamp}')

    curl -X POST "$webhook_url" \
        -H "Content-Type: application/json" \
        -d "$payload" \
        -s -o /dev/null -w "%{http_code}"
}

# Usage
status=$(notify_webhook "Task completed successfully")
echo "Webhook returned: $status"
```

## Debugging

### Verbose Output

```bash
# Show request/response headers and timing
curl -v https://api.example.com/users

# Trace ASCII (shows everything)
curl --trace - https://api.example.com/users

# Trace to file
curl --trace trace.txt https://api.example.com/users
```

### Test Timing

```bash
# Show timing breakdown
curl -w "\n\nTime:\n  DNS: %{time_namelookup}s\n  Connect: %{time_connect}s\n  Transfer: %{time_starttransfer}s\n  Total: %{time_total}s\n" \
  -o /dev/null -s https://api.example.com/users
```

### Save Full Response

```bash
# Save headers and body separately
curl -D headers.txt -o body.json https://api.example.com/users

# Or both in one file with -i
curl -i https://api.example.com/users > response.txt
```

## Common Patterns

### Check if URL is Reachable

```bash
if curl -sSf https://api.example.com/health > /dev/null; then
    echo "API is up"
else
    echo "API is down"
fi
```

### Parallel Requests

```bash
# Launch multiple requests in background
urls=("https://api.example.com/endpoint1" "https://api.example.com/endpoint2")

for url in "${urls[@]}"; do
    curl -s "$url" > "result-$(basename $url).json" &
done

# Wait for all to complete
wait

echo "All requests complete"
```

### Rate Limiting

```bash
# Process URLs with delay
while read -r url; do
    curl -s "$url"
    sleep 1  # Rate limit: 1 request per second
done < urls.txt
```

### GraphQL Queries

```bash
# GraphQL POST request
query='{
  "query": "{ users { id name email } }"
}'

curl -X POST https://api.example.com/graphql \
  -H "Content-Type: application/json" \
  -d "$query" | jq '.data.users'
```

## Best Practices

**1. Always use `-s` for scripts**
```bash
# Suppress progress bar in scripts
curl -s https://api.example.com/data
```

**2. Check status codes**
```bash
# Use -f to fail on HTTP errors
curl -sf https://api.example.com/data || echo "Failed"
```

**3. Set timeouts**
```bash
# Prevent hanging
curl --max-time 30 https://api.example.com/data
```

**4. Use `-S` with `-s` for errors**
```bash
# Silent but show errors
curl -sS https://api.example.com/data
```

**5. Secure sensitive data**
```bash
# Never log tokens
TOKEN=$(cat ~/.api-token)
curl -H "Authorization: Bearer $TOKEN" https://api.example.com/data

# Not: curl -v (would log the token)
```

## Useful Flags Reference

```
-X METHOD      HTTP method (GET, POST, PUT, DELETE, etc.)
-H "header"    Add request header
-d "data"      Send data in request body
-d @file       Send file contents as data
-F "key=val"   Multipart form data
-o file        Save output to file
-O             Save with remote filename
-i             Include response headers in output
-I             Fetch headers only (HEAD request)
-L             Follow redirects
-u user:pass   Basic authentication
-s             Silent mode (no progress)
-S             Show errors even in silent mode
-f             Fail silently on HTTP errors (exit code)
-v             Verbose (show request/response details)
-w format      Custom output format
--retry N      Retry N times on failure
--max-time N   Maximum time in seconds
```

## curl + jq Power Combo

### Extract Nested Data

```bash
# Get specific user's email
curl -s https://api.example.com/users | \
  jq -r '.users[] | select(.id == 123) | .email'
```

### Transform and POST

```bash
# Fetch data, transform, POST to another API
curl -s https://api.source.com/data | \
  jq '{processed: .items | map({id, name})}' | \
  curl -X POST https://api.dest.com/import \
    -H "Content-Type: application/json" \
    -d @-
```

### Batch Processing

```bash
# Process each item from API
curl -s https://api.example.com/items | \
  jq -r '.items[].id' | \
while read -r id; do
    result=$(curl -s "https://api.example.com/items/$id/process")
    echo "Processed item $id: $result"
done
```

## Troubleshooting

### SSL Certificate Issues

```bash
# Ignore SSL verification (use cautiously!)
curl -k https://self-signed-cert.example.com

# Specify CA certificate
curl --cacert /path/to/ca.crt https://example.com
```

### Connection Problems

```bash
# Use IPv4 only
curl -4 https://api.example.com

# Use IPv6 only
curl -6 https://api.example.com

# Specify DNS server
curl --dns-servers 8.8.8.8 https://api.example.com
```

### Proxy

```bash
# Use proxy
curl -x http://proxy.example.com:8080 https://api.example.com

# With proxy auth
curl -x http://user:pass@proxy.example.com:8080 https://api.example.com
```

## Resources

- Manual: `man curl`
- Everything curl book: https://everything.curl.dev/
- Test APIs: https://httpbin.org/
