// @ts-nocheck - Demo script with intentionally relaxed type checking
/**
 * Script: Error Recovery
 * Purpose: Demonstrate retry logic, exponential backoff, and fallback strategies
 * Use Case: Building resilient systems that handle failures gracefully
 * MCP Tools Used: Primary and fallback services, cache systems
 *
 * How to Adapt:
 * 1. Replace primary/secondary/cache tool names with your actual tools
 * 2. Adjust retry counts and backoff timing for your use case
 * 3. Customize fallback strategy (secondary service, cache, defaults)
 * 4. Add circuit breaker pattern if needed
 * 5. Implement custom error classification (retryable vs. permanent)
 *
 * Example Usage:
 * Fetching data from unreliable APIs, database queries with failover,
 * or any operation that might need multiple attempts
 *
 * Execution:
 * deno run --allow-read --allow-run --allow-env error-recovery.ts
 */

import { callMCPTool } from "../../lib/mcp-client.ts";

interface RetryConfig {
  maxRetries: number;
  initialDelay: number; // milliseconds
  maxDelay: number; // milliseconds
  backoffMultiplier: number;
}

interface RecoveryResult {
  success: boolean;
  data?: unknown;
  source: "primary" | "secondary" | "cache" | "default";
  attempts: number;
  duration: number;
  errors: string[];
}

/**
 * Execute a tool call with exponential backoff retry logic
 */
async function callWithRetry(
  toolName: string,
  params: Record<string, unknown>,
  config: RetryConfig,
): Promise<unknown> {
  let lastError: Error;

  for (let attempt = 0; attempt < config.maxRetries; attempt++) {
    try {
      console.log(`Attempt ${attempt + 1}/${config.maxRetries}: ${toolName}`);
      const result = await callMCPTool(toolName, params);
      console.log(`✓ Success on attempt ${attempt + 1}`);
      return result;
    } catch (error) {
      lastError = error;
      console.warn(
        `✗ Attempt ${attempt + 1} failed: ${(error as Error).message}`,
      );

      // Don't wait after the last attempt
      if (attempt < config.maxRetries - 1) {
        // Calculate delay with exponential backoff
        const delay = Math.min(
          config.initialDelay * Math.pow(config.backoffMultiplier, attempt),
          config.maxDelay,
        );

        console.log(`Waiting ${delay}ms before retry...`);
        await new Promise((resolve) => setTimeout(resolve, delay));
      }
    }
  }

  // All retries exhausted
  throw lastError;
}

/**
 * Fetch data with comprehensive error recovery strategy
 */
async function fetchDataWithRecovery(dataId: string): Promise<RecoveryResult> {
  const startTime = Date.now();
  const errors: string[] = [];
  let attempts = 0;

  // Configuration for retry logic
  const retryConfig: RetryConfig = {
    maxRetries: 3,
    initialDelay: 1000, // Start with 1 second
    maxDelay: 10000, // Cap at 10 seconds
    backoffMultiplier: 2, // Double delay each time (1s, 2s, 4s)
  };

  console.log("=== Starting Data Fetch with Error Recovery ===");
  console.log(`Target: ${dataId}`);

  // Strategy 1: Try primary service with retries
  console.log("\n[Strategy 1] Attempting primary service with retries...");
  try {
    const data = await callWithRetry(
      "mcp__primary__getData",
      { id: dataId, timeout: 5000 },
      retryConfig,
    );

    attempts += retryConfig.maxRetries; // Count actual attempts made

    return {
      success: true,
      data,
      source: "primary",
      attempts,
      duration: Date.now() - startTime,
      errors,
    };
  } catch (error) {
    errors.push(`Primary service: ${(error as Error).message}`);
    console.error(
      `✗ Primary service failed after ${retryConfig.maxRetries} attempts`,
    );
  }

  // Strategy 2: Try secondary/fallback service
  console.log("\n[Strategy 2] Attempting secondary service...");
  try {
    attempts++;
    const data = await callMCPTool("mcp__secondary__getData", {
      id: dataId,
    });

    console.log("✓ Secondary service succeeded");

    return {
      success: true,
      data,
      source: "secondary",
      attempts,
      duration: Date.now() - startTime,
      errors,
    };
  } catch (error) {
    errors.push(`Secondary service: ${(error as Error).message}`);
    console.error(`✗ Secondary service failed: ${(error as Error).message}`);
  }

  // Strategy 3: Try cache as last resort
  console.log("\n[Strategy 3] Attempting cached data...");
  try {
    attempts++;
    const data = await callMCPTool("mcp__cache__get", {
      key: `data:${dataId}`,
    });

    if (data && data.value) {
      console.log("✓ Found cached data");
      console.warn("⚠ Using potentially stale cached data");

      return {
        success: true,
        data: data.value,
        source: "cache",
        attempts,
        duration: Date.now() - startTime,
        errors,
      };
    } else {
      throw new Error("No cached data available");
    }
  } catch (error) {
    errors.push(`Cache: ${(error as Error).message}`);
    console.error(`✗ Cache lookup failed: ${(error as Error).message}`);
  }

  // Strategy 4: Return default/empty data
  console.log(
    "\n[Strategy 4] All strategies exhausted, returning default data",
  );
  console.warn("⚠ Returning default empty result");

  return {
    success: false,
    data: null,
    source: "default",
    attempts,
    duration: Date.now() - startTime,
    errors,
  };
}

// Example: Batch processing with error recovery
async function batchProcessWithRecovery(ids: string[]) {
  console.log(`\n=== Batch Processing ${ids.length} items ===\n`);

  const results = await Promise.all(
    ids.map((id) => fetchDataWithRecovery(id)),
  );

  const successful = results.filter((r) => r.success && r.source !== "default");
  const fromCache = results.filter((r) => r.success && r.source === "cache");
  const failed = results.filter((r) => !r.success);

  console.log("\n=== Batch Processing Summary ===");
  console.log(`Total items: ${ids.length}`);
  console.log(`Successful: ${successful.length}`);
  console.log(`From cache: ${fromCache.length}`);
  console.log(`Failed: ${failed.length}`);

  if (fromCache.length > 0) {
    console.warn(
      `⚠ ${fromCache.length} items retrieved from potentially stale cache`,
    );
  }

  return { successful, fromCache, failed };
}

// Execute single fetch
console.log("Example 1: Single Item Recovery");
const singleResult = await fetchDataWithRecovery("user-123");
console.log("\nResult:", {
  success: singleResult.success,
  source: singleResult.source,
  attempts: singleResult.attempts,
  duration: `${singleResult.duration}ms`,
  errorCount: singleResult.errors.length,
});

// Execute batch processing
console.log("\n\n" + "=".repeat(50));
console.log("Example 2: Batch Processing with Recovery");
const _batchResult = await batchProcessWithRecovery([
  "user-123",
  "user-456",
  "user-789",
]);
