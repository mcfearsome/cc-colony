# Data Pipeline

Build an automated data processing pipeline with specialized agents for ETL workflows.

## Overview

This example demonstrates how to use Colony for data engineering tasks: extracting data from sources, transforming it, loading into destinations, and validating results.

## Use Case

**Scenario:** Daily data pipeline that:
1. Fetches data from multiple APIs
2. Cleans and transforms the data
3. Enriches with additional sources
4. Validates data quality
5. Loads into data warehouse
6. Generates reports

## Architecture

```
┌─────────────┐
│   Fetcher   │ ──┐
└─────────────┘   │
                  ├─→ ┌──────────────┐
┌─────────────┐   │   │              │      ┌──────────┐
│ Transformer │ ──┼─→ │ Orchestrator │ ───→ │  Loader  │
└─────────────┘   │   │              │      └──────────┘
                  ├─→ └──────────────┘           │
┌─────────────┐   │                              │
│  Validator  │ ──┘                              ▼
└─────────────┘                           ┌────────────┐
                                          │ Warehouse  │
                                          └────────────┘
```

## Configuration

### colony.yml

```yaml
name: data-pipeline-colony

# Enable state for pipeline coordination
shared_state:
  backend: git-backed
  location: in-repo

agents:
  - id: orchestrator
    role: Pipeline Orchestrator
    model: claude-sonnet-4-20250514
    worktree_branch: pipeline/orchestrator
    startup_prompt: |
      You are the pipeline orchestrator responsible for coordinating
      the entire ETL workflow.

      RESPONSIBILITIES:
      - Coordinate agent tasks and dependencies
      - Monitor pipeline health
      - Handle failures and retries
      - Generate pipeline reports

      WORKFLOW:
      1. Create tasks for data fetcher
      2. Wait for fetch completion
      3. Assign transform tasks
      4. Coordinate validation
      5. Trigger load when validated
      6. Generate completion report

  - id: data-fetcher
    role: Data Fetcher
    model: claude-sonnet-4-20250514
    worktree_branch: pipeline/fetcher
    env:
      API_KEY: $DATA_API_KEY
      RATE_LIMIT: "100"
    startup_prompt: |
      You are responsible for fetching data from external sources.

      DATA SOURCES:
      - REST APIs (with rate limiting)
      - CSV files from S3
      - Database exports

      REQUIREMENTS:
      - Handle API rate limits gracefully
      - Implement retry logic with exponential backoff
      - Validate data completeness before marking complete
      - Store raw data in staging/raw/
      - Log all fetch operations

      ERROR HANDLING:
      - API errors: Retry up to 3 times
      - Network timeouts: Implement circuit breaker
      - Data corruption: Flag for manual review

  - id: data-transformer
    role: Data Transformer
    model: claude-sonnet-4-20250514
    worktree_branch: pipeline/transformer
    startup_prompt: |
      Transform raw data into clean, structured format.

      TRANSFORMATIONS:
      - Data cleaning (nulls, duplicates, outliers)
      - Schema normalization
      - Type conversions
      - Data enrichment
      - Aggregations

      QUALITY STANDARDS:
      - All dates in ISO 8601 format
      - Standardized field names (snake_case)
      - Required fields must be non-null
      - Numeric fields validated for range
      - Document all transformations applied

      OUTPUT:
      - Store in staging/transformed/
      - Generate transformation report
      - Flag records that couldn't be transformed

  - id: data-validator
    role: Data Quality Validator
    model: claude-sonnet-4-20250514
    worktree_branch: pipeline/validator
    startup_prompt: |
      Validate data quality before loading to warehouse.

      VALIDATION CHECKS:
      - Schema compliance
      - Data completeness (% non-null)
      - Referential integrity
      - Business rule validation
      - Statistical anomaly detection

      THRESHOLDS:
      - Completeness: >= 95%
      - Duplicate rate: < 1%
      - Error rate: < 0.1%

      ACTIONS:
      - PASS: Mark for loading
      - WARN: Load but flag issues
      - FAIL: Block and alert orchestrator

  - id: data-loader
    role: Data Loader
    model: claude-sonnet-4-20250514
    worktree_branch: pipeline/loader
    env:
      WAREHOUSE_CONNECTION: $WAREHOUSE_URL
    startup_prompt: |
      Load validated data into the data warehouse.

      LOADING STRATEGY:
      - Use bulk insert for efficiency
      - Implement idempotent loads (upsert)
      - Handle conflicts (update vs append)
      - Maintain load history

      SAFETY:
      - Always use transactions
      - Verify row counts before/after
      - Create backup before major loads
      - Log all load operations

      POST-LOAD:
      - Update metadata tables
      - Refresh materialized views
      - Run post-load validations
      - Generate load report

  - id: reporter
    role: Report Generator
    model: claude-sonnet-4-20250514
    worktree_branch: pipeline/reporter
    startup_prompt: |
      Generate pipeline execution reports and metrics.

      REPORTS:
      - Daily pipeline summary
      - Data quality metrics
      - Performance statistics
      - Error analysis

      METRICS:
      - Records processed
      - Processing time
      - Error rates
      - Data quality scores

      OUTPUT:
      - Markdown reports in reports/
      - JSON metrics in metrics/
      - Send alerts for anomalies
```

## Workflow Setup

### 1. Initialize Pipeline

```bash
# Initialize colony
colony init

# Copy configuration
cat > colony.yml < [paste config above]

# Create directory structure
mkdir -p staging/{raw,transformed,validated}
mkdir -p reports metrics logs

# Start the colony
colony start
```

### 2. Define Workflow

Create `.colony/workflows/daily-etl.yaml`:

```yaml
workflow:
  name: daily-etl
  description: "Daily data pipeline execution"

  trigger:
    type: schedule
    cron: "0 2 * * *"  # Run at 2 AM daily

  steps:
    - name: fetch-data
      agent: data-fetcher
      timeout: 10m
      instructions: |
        Fetch data from all configured sources for yesterday's date.
        Store raw data in staging/raw/ with timestamp.
      output: raw_data_path

    - name: transform-data
      agent: data-transformer
      depends_on: [fetch-data]
      timeout: 15m
      instructions: |
        Transform raw data from {{steps.fetch-data.output}}.
        Apply all cleaning and normalization rules.
        Store in staging/transformed/
      output: transformed_data_path
      retry:
        max_attempts: 2
        backoff: exponential

    - name: validate-data
      agent: data-validator
      depends_on: [transform-data]
      timeout: 5m
      instructions: |
        Validate data at {{steps.transform-data.output}}.
        Run all quality checks and generate validation report.
      output: validation_status

    - name: load-data
      agent: data-loader
      depends_on: [validate-data]
      timeout: 20m
      instructions: |
        Load validated data to warehouse.
        Only proceed if validation_status is PASS or WARN.
      output: load_stats

    - name: generate-report
      agent: reporter
      depends_on: [load-data]
      timeout: 5m
      instructions: |
        Generate daily pipeline report with:
        - Records processed: {{steps.load-data.output.record_count}}
        - Quality score: {{steps.validate-data.output.quality_score}}
        - Processing time: total pipeline duration
        - Any warnings or errors

  error_handling:
    - step: notify-on-failure
      agent: orchestrator
      instructions: "Send alert about pipeline failure"
```

### 3. Create Task Dependencies

```bash
# Orchestrator creates the task chain
colony state task add "fetch-daily-data" \
  --description "Fetch data from all sources"

colony state task add "transform-daily-data" \
  --description "Clean and transform fetched data" \
  --blockers "fetch-daily-data"

colony state task add "validate-daily-data" \
  --description "Run quality validation" \
  --blockers "transform-daily-data"

colony state task add "load-daily-data" \
  --description "Load to warehouse" \
  --blockers "validate-daily-data"

colony state task add "generate-daily-report" \
  --description "Create pipeline report" \
  --blockers "load-daily-data"
```

## Execution

### Manual Run

```bash
# Start the pipeline manually
colony workflow run daily-etl

# Monitor in TUI
colony tui
# Tab 4: Watch workflow progress
# Tab 2: Monitor task completion

# Check specific agent logs
colony logs data-fetcher
colony logs data-validator

# View workflow status
colony workflow status <run-id>
```

### Monitoring

```bash
# Real-time monitoring in separate terminal
watch -n 5 'colony status && echo "" && colony state task list'

# Check for errors
colony logs --level error

# View metrics
colony metrics show pipeline_records_processed
colony metrics show pipeline_duration
```

## Agent Scripts

### Data Fetcher Implementation

Create `scripts/fetch_data.py`:

```python
#!/usr/bin/env python3
import os
import requests
import json
from datetime import datetime, timedelta
from pathlib import Path
import time

class DataFetcher:
    def __init__(self):
        self.api_key = os.environ.get('API_KEY')
        self.rate_limit = int(os.environ.get('RATE_LIMIT', 100))
        self.staging_dir = Path('staging/raw')
        self.staging_dir.mkdir(parents=True, exist_ok=True)

    def fetch_api_data(self, endpoint, date):
        """Fetch data from API with retry logic."""
        max_retries = 3
        for attempt in range(max_retries):
            try:
                response = requests.get(
                    endpoint,
                    params={'date': date, 'api_key': self.api_key},
                    timeout=30
                )
                response.raise_for_status()
                return response.json()
            except requests.exceptions.RequestException as e:
                if attempt == max_retries - 1:
                    raise
                wait_time = 2 ** attempt
                print(f"Retry {attempt + 1}/{max_retries} after {wait_time}s")
                time.sleep(wait_time)

    def save_raw_data(self, data, source_name, date):
        """Save raw data to staging."""
        filename = f"{source_name}_{date}.json"
        filepath = self.staging_dir / filename

        with open(filepath, 'w') as f:
            json.dump({
                'source': source_name,
                'fetch_time': datetime.now().isoformat(),
                'date': date,
                'record_count': len(data),
                'data': data
            }, f, indent=2)

        return str(filepath)

    def run(self):
        """Main fetch process."""
        yesterday = (datetime.now() - timedelta(days=1)).strftime('%Y-%m-%d')

        sources = [
            ('sales_api', 'https://api.example.com/sales'),
            ('inventory_api', 'https://api.example.com/inventory'),
        ]

        results = {}
        for source_name, endpoint in sources:
            print(f"Fetching from {source_name}...")
            data = self.fetch_api_data(endpoint, yesterday)
            filepath = self.save_raw_data(data, source_name, yesterday)
            results[source_name] = {
                'filepath': filepath,
                'records': len(data)
            }
            print(f"✓ Fetched {len(data)} records to {filepath}")

        # Update state
        with open('.colony/state/fetch_results.json', 'w') as f:
            json.dump(results, f, indent=2)

        return results

if __name__ == '__main__':
    fetcher = DataFetcher()
    results = fetcher.run()
    print(f"\nFetch complete: {sum(r['records'] for r in results.values())} total records")
```

### Validator Implementation

Create `scripts/validate_data.py`:

```python
#!/usr/bin/env python3
import json
import pandas as pd
from pathlib import Path
from datetime import datetime

class DataValidator:
    def __init__(self):
        self.thresholds = {
            'completeness': 0.95,
            'duplicate_rate': 0.01,
            'error_rate': 0.001
        }

    def validate_schema(self, df, required_columns):
        """Check schema compliance."""
        missing = set(required_columns) - set(df.columns)
        if missing:
            return False, f"Missing columns: {missing}"
        return True, "Schema valid"

    def check_completeness(self, df):
        """Check data completeness."""
        completeness = df.notna().sum() / len(df)
        avg_completeness = completeness.mean()

        return (
            avg_completeness >= self.thresholds['completeness'],
            f"Completeness: {avg_completeness:.2%}"
        )

    def check_duplicates(self, df, key_columns):
        """Check for duplicates."""
        duplicates = df.duplicated(subset=key_columns).sum()
        duplicate_rate = duplicates / len(df)

        return (
            duplicate_rate <= self.thresholds['duplicate_rate'],
            f"Duplicates: {duplicate_rate:.2%} ({duplicates} records)"
        )

    def validate(self, filepath):
        """Run all validation checks."""
        df = pd.read_json(filepath)

        results = {
            'file': filepath,
            'timestamp': datetime.now().isoformat(),
            'record_count': len(df),
            'checks': []
        }

        # Run checks
        checks = [
            ('schema', self.validate_schema(df, ['id', 'date', 'value'])),
            ('completeness', self.check_completeness(df)),
            ('duplicates', self.check_duplicates(df, ['id', 'date'])),
        ]

        all_passed = True
        for check_name, (passed, message) in checks:
            results['checks'].append({
                'name': check_name,
                'passed': passed,
                'message': message
            })
            all_passed = all_passed and passed

        results['status'] = 'PASS' if all_passed else 'FAIL'
        return results

if __name__ == '__main__':
    import sys
    validator = DataValidator()
    results = validator.validate(sys.argv[1])

    print(json.dumps(results, indent=2))

    # Save validation report
    report_path = Path('reports') / f"validation_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
    report_path.parent.mkdir(exist_ok=True)
    with open(report_path, 'w') as f:
        json.dump(results, f, indent=2)
```

## Best Practices

### Error Handling

**Implement Circuit Breakers:**
```python
class CircuitBreaker:
    def __init__(self, failure_threshold=5, timeout=60):
        self.failure_count = 0
        self.failure_threshold = failure_threshold
        self.timeout = timeout
        self.last_failure_time = None
        self.state = 'CLOSED'  # CLOSED, OPEN, HALF_OPEN

    def call(self, func, *args, **kwargs):
        if self.state == 'OPEN':
            if time.time() - self.last_failure_time > self.timeout:
                self.state = 'HALF_OPEN'
            else:
                raise Exception("Circuit breaker OPEN")

        try:
            result = func(*args, **kwargs)
            if self.state == 'HALF_OPEN':
                self.state = 'CLOSED'
                self.failure_count = 0
            return result
        except Exception as e:
            self.failure_count += 1
            self.last_failure_time = time.time()

            if self.failure_count >= self.failure_threshold:
                self.state = 'OPEN'
            raise
```

### Data Quality Metrics

Track key metrics:
```bash
# Record metrics after each pipeline run
colony metrics record pipeline_records_processed 10543
colony metrics record pipeline_duration_seconds 1834
colony metrics record pipeline_error_count 3
colony metrics record data_quality_score 0.98
```

### Alerting

```bash
# In reporter agent script
if [ $ERROR_COUNT -gt 10 ]; then
  colony broadcast "🚨 Pipeline errors exceeded threshold: $ERROR_COUNT errors"
fi

if [ $QUALITY_SCORE -lt 0.95 ]; then
  colony broadcast "⚠️ Data quality below threshold: $QUALITY_SCORE"
fi
```

## Monitoring Dashboard

View pipeline health in TUI:

```bash
colony tui

# Tab 1: Monitor agent status
# Tab 2: Track task progress
# Tab 4: View workflow state
# Tab 5: Check metrics

# Look for:
# - All agents running (no failures)
# - Tasks completing in sequence
# - No blocked tasks
# - Quality metrics within range
```

## Advanced Features

### Incremental Loading

```python
def get_last_load_timestamp():
    """Get timestamp of last successful load."""
    metadata_file = '.colony/state/load_metadata.json'
    if Path(metadata_file).exists():
        with open(metadata_file) as f:
            metadata = json.load(f)
            return metadata.get('last_load_time')
    return None

def fetch_incremental(since_timestamp):
    """Fetch only data since last load."""
    # Only fetch new/updated records
    pass
```

### Parallel Processing

```yaml
# In workflow definition
steps:
  - name: transform-data
    agent: data-transformer
    parallel: 4  # Spawn 4 parallel workers
    instructions: "Transform batch {{batch_id}} of data"
```

### Data Lineage

Track data provenance:
```python
lineage = {
    'source': 'sales_api',
    'fetch_time': '2024-01-15T02:00:00Z',
    'transformations': [
        {'type': 'clean_nulls', 'rows_affected': 123},
        {'type': 'normalize_dates', 'rows_affected': 10543},
    ],
    'validation_passed': True,
    'load_time': '2024-01-15T02:45:00Z',
    'warehouse_table': 'sales_fact'
}
```

## Troubleshooting

**Pipeline Stuck:**
```bash
# Check which step is blocking
colony workflow status <run-id>
colony state task list

# View agent logs
colony logs data-fetcher --level error
```

**Data Quality Issues:**
```bash
# Review validation reports
cat reports/validation_*.json | jq '.checks[] | select(.passed == false)'

# Check transformation logic
colony logs data-transformer --pattern "transform"
```

**Performance Issues:**
```bash
# Check metrics
colony metrics show pipeline_duration_seconds --hours 168  # Last week

# Identify slow steps
colony workflow history daily-etl --limit 10
```

## See Also

- [Workflows](../concepts/workflows.md) - Workflow orchestration
- [State Management](../concepts/state.md) - Shared state
- [Best Practices](../advanced/best-practices.md) - General guidelines
- [Testing Pipeline](./testing-pipeline.md) - Test automation
