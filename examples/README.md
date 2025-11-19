# Example Colony Configurations

This directory contains example `colony.yml` configurations for different authentication methods and use cases.

## Authentication Methods

### OAuth (Claude Pro/Max Users)
**File:** `colony-oauth.yml`

For developers with Claude Pro or Max subscriptions who want to use their existing account.

```bash
# First, authenticate via browser
colony auth login

# Then start your colony
colony start
```

### API Key (Direct)
**File:** `colony-api-key.yml`

For developers with Anthropic API keys.

```bash
# Set your API key in environment
export ANTHROPIC_API_KEY=sk-ant-api03-...

# Or authenticate interactively
colony auth login --method api-key

# Then start your colony
colony start
```

### AWS Bedrock
**File:** `colony-bedrock.yml`

For organizations using AWS Bedrock.

```bash
# Configure AWS credentials
aws configure --profile default

# Authenticate with Colony
colony auth login --method bedrock --region us-east-1 --profile default

# Then start your colony
colony start
```

### Environment Variables (Secure)
**File:** `colony-env-vars.yml`

Best practice configuration using environment variables for all secrets.

```bash
# Set environment variables
export ANTHROPIC_API_KEY=sk-ant-api03-...
export DATABASE_URL=postgresql://...
export REDIS_URL=redis://...

# Start colony
colony start
```

## Getting Started

1. **Choose a configuration** that matches your authentication method
2. **Copy to your project** as `colony.yml`:
   ```bash
   cp examples/colony-oauth.yml colony.yml
   ```
3. **Customize agents** for your specific needs
4. **Authenticate**:
   ```bash
   colony auth login
   ```
5. **Start your colony**:
   ```bash
   colony start
   ```

## Authentication Status

Check your current authentication status:

```bash
colony auth status
```

Output example:
```
🔐 Authentication Status

Provider: Claude.ai OAuth (Pro/Max)
Status: ✅ Authenticated
Expires: in 29 days, 14 hours
```

## Switching Authentication Methods

To switch between authentication methods:

1. **Logout** from current method:
   ```bash
   colony auth logout
   ```

2. **Update** `colony.yml` with new auth config

3. **Login** with new method:
   ```bash
   colony auth login --method <method>
   ```

## Security Best Practices

### ✅ DO:
- Use environment variables for API keys
- Keep `tokens.json` in your `.gitignore`
- Use OAuth for personal projects
- Use Bedrock for enterprise deployments
- Regularly rotate API keys

### ❌ DON'T:
- Commit API keys to version control
- Share OAuth tokens between machines
- Use production API keys in development
- Store credentials in plain text files

## Troubleshooting

### OAuth Issues
```bash
# Token expired
colony auth refresh

# Re-authenticate
colony auth logout
colony auth login
```

### API Key Issues
```bash
# Test API key
colony auth login --method api-key --api-key sk-ant-...

# Check environment variable
echo $ANTHROPIC_API_KEY
```

### Bedrock Issues
```bash
# Test AWS credentials
aws sts get-caller-identity --profile default

# Re-authenticate
colony auth login --method bedrock --region us-east-1
```

## More Examples

For more complex configurations, see:
- [Code Review Workflow](../docs/src/examples/code-review.md)
- [Data Pipeline](../docs/src/examples/data-pipeline.md)
- [Best Practices](../docs/src/advanced/best-practices.md)
