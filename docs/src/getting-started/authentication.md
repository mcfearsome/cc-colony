# Authentication

Colony supports multiple authentication methods to work with Claude AI models, allowing you to choose the approach that best fits your workflow and subscription type.

## Supported Methods

| Method | Best For | Setup Time |
|--------|----------|------------|
| **OAuth** | Claude Pro/Max users | 1 minute |
| **API Key** | Developers with API access | 30 seconds |
| **Bedrock** | AWS/Enterprise users | 2 minutes |
| **Vertex AI** | Google Cloud users | Coming soon |

## Quick Start

### OAuth (Recommended for Pro/Max Users)

Perfect for individual developers with Claude Pro or Max subscriptions.

```bash
# One command - opens browser
colony auth login

# Follow browser prompts
# ✅ Done!
```

**What happens:**
1. Opens your browser to claude.ai
2. Login with your existing account
3. Approve access
4. Automatically saves credentials
5. Ready to use!

**Advantages:**
- ✅ No API key needed
- ✅ Uses your existing subscription
- ✅ Secure OAuth 2.0 flow
- ✅ Automatic token refresh

### API Key (For Direct API Access)

For developers with Anthropic API keys.

```bash
# Interactive setup
colony auth login --method api-key

# Or set environment variable
export ANTHROPIC_API_KEY=sk-ant-api03-xxxxx

# Verify
colony auth status
```

**Advantages:**
- ✅ Simple setup
- ✅ Programmatic access
- ✅ Works in CI/CD pipelines
- ✅ No browser required

### Bedrock (For AWS Users)

For organizations using AWS infrastructure.

```bash
# Configure AWS credentials first
aws configure --profile my-profile

# Then authenticate Colony
colony auth login --method bedrock \
  --region us-east-1 \
  --profile my-profile

# Verify
colony auth status
```

**Advantages:**
- ✅ Enterprise-grade
- ✅ AWS billing integration
- ✅ IAM role support
- ✅ Regional deployment

## Configuration

Add to your `colony.yml`:

### OAuth Configuration

```yaml
auth:
  provider: anthropic-oauth
  token_path: ~/.colony/auth/tokens.json
```

### API Key Configuration

```yaml
# Option 1: Environment variable (recommended)
auth:
  provider: api-key
  # Uses ANTHROPIC_API_KEY from environment

# Option 2: Explicit key (not recommended)
auth:
  provider: api-key
  api_key: sk-ant-api03-xxxxx  # Don't commit this!
```

### Bedrock Configuration

```yaml
auth:
  provider: bedrock
  region: us-east-1
  profile: default  # AWS profile name
```

## OAuth Deep Dive

### How It Works

1. **Authentication Request**
   - Colony starts local server on port 8888
   - Opens browser to claude.ai/oauth/authorize
   - Generates secure PKCE challenge

2. **User Authorization**
   - You log in to claude.ai
   - Review and approve access
   - Browser redirects to localhost:8888

3. **Token Exchange**
   - Colony receives authorization code
   - Exchanges code for access token
   - Stores token securely with 0600 permissions

4. **Automatic Refresh**
   - Tokens expire after 30 days
   - Colony auto-refreshes when needed
   - Or manually: `colony auth refresh`

### Security Features

**PKCE (Proof Key for Code Exchange)**
- Prevents authorization code interception
- No client secret needed
- Secure for public clients (CLIs)

**State Parameter**
- CSRF protection
- Validates redirect authenticity

**Secure Storage**
- Tokens stored in `~/.colony/auth/tokens.json`
- File permissions: 0600 (owner read/write only)
- Not in project directory (safe from git)

### Token Management

**Check status:**
```bash
colony auth status
```

Output:
```
🔐 Authentication Status

Provider: Claude.ai OAuth (Pro/Max)
Status: ✅ Authenticated
Expires: in 29 days, 14 hours
```

**Refresh token:**
```bash
colony auth refresh
```

**Logout:**
```bash
colony auth logout
```

## API Key Best Practices

### Environment Variables

**~/.bashrc or ~/.zshrc:**
```bash
export ANTHROPIC_API_KEY=sk-ant-api03-xxxxx
```

**Per-project (.env file):**
```bash
# .env
ANTHROPIC_API_KEY=sk-ant-api03-xxxxx
```

Then:
```bash
# Load environment
source .env

# Or use direnv
echo 'export ANTHROPIC_API_KEY=sk-ant-...' > .envrc
direnv allow
```

### Key Rotation

```bash
# 1. Generate new key in Anthropic Console
# 2. Update environment variable
export ANTHROPIC_API_KEY=sk-ant-new-key

# 3. Verify
colony auth status

# 4. Revoke old key in Anthropic Console
```

### Security Checklist

- [ ] Never commit keys to version control
- [ ] Add `.env` to `.gitignore`
- [ ] Use separate keys for dev/prod
- [ ] Rotate keys every 90 days
- [ ] Monitor usage in Anthropic Console
- [ ] Revoke compromised keys immediately

## Bedrock Setup

### Prerequisites

1. **AWS Account** with Bedrock access
2. **AWS CLI** installed and configured
3. **Bedrock models** enabled in AWS Console

### Step-by-Step

**1. Enable Bedrock Models**
```bash
# In AWS Console:
# - Go to Amazon Bedrock
# - Click "Model access"
# - Request access to Claude models
# - Wait for approval (usually instant)
```

**2. Configure AWS Credentials**
```bash
# Method 1: AWS CLI
aws configure --profile my-colony
# Enter: Access Key, Secret Key, Region

# Method 2: IAM Role (EC2/ECS)
# Automatically uses instance role
```

**3. Test Access**
```bash
# Verify credentials
aws sts get-caller-identity --profile my-colony

# Test Bedrock access
aws bedrock list-foundation-models \
  --region us-east-1 \
  --profile my-colony
```

**4. Authenticate Colony**
```bash
colony auth login --method bedrock \
  --region us-east-1 \
  --profile my-colony
```

### IAM Permissions

Required IAM policy:
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "bedrock:InvokeModel",
        "bedrock:InvokeModelWithResponseStream"
      ],
      "Resource": "arn:aws:bedrock:*::foundation-model/anthropic.claude-*"
    }
  ]
}
```

### Cost Management

**Monitor costs:**
```bash
# AWS Cost Explorer
aws ce get-cost-and-usage \
  --time-period Start=2024-01-01,End=2024-01-31 \
  --granularity MONTHLY \
  --metrics BlendedCost \
  --filter file://bedrock-filter.json
```

**Set budget alerts:**
- AWS Budgets
- Alert when costs exceed threshold
- Email/SNS notifications

## Troubleshooting

### OAuth Issues

**Problem: Browser doesn't open**
```bash
# Manually open the URL shown in terminal
# URL format: https://claude.ai/oauth/authorize?...
```

**Problem: "Port 8888 already in use"**
```bash
# Find what's using port 8888
lsof -i :8888

# Kill the process
kill -9 <PID>

# Or use different port (future feature)
```

**Problem: Token expired**
```bash
# Refresh token
colony auth refresh

# Or re-authenticate
colony auth logout
colony auth login
```

### API Key Issues

**Problem: "Invalid API key"**
```bash
# Check key format (must start with sk-ant-)
echo $ANTHROPIC_API_KEY

# Verify in Anthropic Console
# Keys > Your API keys

# Test key
colony auth login --method api-key --api-key sk-ant-...
```

**Problem: Key not found**
```bash
# Check environment variable is set
env | grep ANTHROPIC

# Add to shell config
echo 'export ANTHROPIC_API_KEY=sk-ant-...' >> ~/.bashrc
source ~/.bashrc
```

### Bedrock Issues

**Problem: "Access denied"**
```bash
# Check AWS credentials
aws sts get-caller-identity

# Verify Bedrock access enabled
aws bedrock list-foundation-models --region us-east-1

# Check IAM permissions
# Need: bedrock:InvokeModel
```

**Problem: "Model not available in region"**
```bash
# Claude models available in:
# - us-east-1 (N. Virginia)
# - us-west-2 (Oregon)
# - eu-west-1 (Ireland)
# - ap-southeast-1 (Singapore)

# Change region
colony auth login --method bedrock --region us-east-1
```

## Authentication Status

### Check Current Auth

```bash
colony auth status
```

Example output:

**OAuth:**
```
🔐 Authentication Status

Provider: Claude.ai OAuth (Pro/Max)
Status: ✅ Authenticated
Expires: in 29 days, 14 hours
```

**API Key:**
```
🔐 Authentication Status

Provider: Anthropic API Key
Status: ✅ Configured via environment variable
Validation: ✅ API key is valid
```

**Bedrock:**
```
🔐 Authentication Status

Provider: AWS Bedrock
Region: us-east-1
Profile: default
Status: ✅ Connected
```

## Switching Authentication Methods

### From OAuth to API Key

```bash
# 1. Logout from OAuth
colony auth logout

# 2. Update colony.yml
# Change: provider: anthropic-oauth
# To:     provider: api-key

# 3. Set API key
export ANTHROPIC_API_KEY=sk-ant-...

# 4. Verify
colony auth status
```

### From API Key to Bedrock

```bash
# 1. Configure AWS
aws configure --profile bedrock-profile

# 2. Update colony.yml
# Change: provider: api-key
# To:     provider: bedrock
#         region: us-east-1
#         profile: bedrock-profile

# 3. Authenticate
colony auth login --method bedrock

# 4. Verify
colony auth status
```

## Multiple Projects

### Different Auth Per Project

```bash
# Project 1: OAuth
cd ~/project1
cat colony.yml  # auth: provider: anthropic-oauth
colony start

# Project 2: API Key
cd ~/project2
cat colony.yml  # auth: provider: api-key
export ANTHROPIC_API_KEY=sk-ant-proj2-...
colony start

# Project 3: Bedrock
cd ~/project3
cat colony.yml  # auth: provider: bedrock
colony start
```

### Shared OAuth Token

OAuth tokens are stored globally:
```
~/.colony/auth/tokens.json
```

Multiple projects can use the same OAuth authentication:
```yaml
# All projects can use this
auth:
  provider: anthropic-oauth
  token_path: ~/.colony/auth/tokens.json
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Colony Tests

on: [push]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Colony
        run: |
          # Install colony
          curl -sSL https://install.colony.sh | sh

      - name: Authenticate
        env:
          ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
        run: |
          # colony.yml uses environment variable
          colony auth status

      - name: Run tests
        run: |
          colony start --no-attach
          # Run tests...
          colony stop
```

### GitLab CI

```yaml
colony-test:
  image: ubuntu:latest
  variables:
    ANTHROPIC_API_KEY: $ANTHROPIC_API_KEY
  script:
    - curl -sSL https://install.colony.sh | sh
    - colony auth status
    - colony start --no-attach
    - # Run tests
    - colony stop
```

## Security Considerations

### Credential Storage

| Method | Storage Location | Permissions | Encryption |
|--------|-----------------|-------------|------------|
| OAuth | `~/.colony/auth/tokens.json` | 0600 | At rest (future) |
| API Key | Environment variable | N/A | In memory only |
| Bedrock | `~/.aws/credentials` | 0600 | AWS managed |

### Best Practices

**For Individuals:**
- ✅ Use OAuth for convenience
- ✅ Enable 2FA on claude.ai account
- ✅ Logout from shared machines

**For Teams:**
- ✅ Use API keys with rotation
- ✅ Separate dev/staging/prod keys
- ✅ Monitor usage per key

**For Enterprise:**
- ✅ Use Bedrock with IAM roles
- ✅ Enable CloudTrail logging
- ✅ Set up cost alerts
- ✅ Use SCPs for guardrails

## Next Steps

- [Quick Start Guide](./quick-start.md)
- [Configuration](./configuration.md)
- [Best Practices](../advanced/best-practices.md)
- [CLI Reference](../cli/overview.md)
