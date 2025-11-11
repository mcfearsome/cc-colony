/**
 * Script: Parallel Execution
 * Purpose: Execute multiple independent tool calls concurrently for better performance
 * Use Case: When you have multiple operations that don't depend on each other
 * MCP Tools Used: Multiple independent MCP tools (databases, APIs, services)
 *
 * How to Adapt:
 * 1. Replace example tool calls with your actual independent operations
 * 2. Adjust the number of parallel operations
 * 3. Decide on error handling strategy (fail-fast vs. partial success)
 * 4. Customize result aggregation logic
 * 5. Add timeout handling if needed
 *
 * Example Usage:
 * Fetch data from multiple sources simultaneously, aggregate analytics from
 * different services, or perform bulk operations in parallel
 *
 * Execution:
 * deno run --allow-read --allow-run --allow-env parallel-execution.ts
 */

import { callMCPTool } from "../../lib/mcp-client.ts";

interface ParallelResult {
  success: boolean;
  completed: number;
  failed: number;
  results: unknown[];
  errors: string[];
  duration: number;
  speedup: string;
}

async function parallelExecution(): Promise<ParallelResult> {
  console.log("=== Starting Parallel Execution ===");
  const startTime = Date.now();

  // Define independent operations to run in parallel
  const operations = [
    {
      name: "Fetch Users",
      call: () =>
        callMCPTool("mcp__database__query", {
          table: "users",
          limit: 1000,
        }),
    },
    {
      name: "Fetch Orders",
      call: () =>
        callMCPTool("mcp__database__query", {
          table: "orders",
          limit: 1000,
        }),
    },
    {
      name: "Fetch Products",
      call: () =>
        callMCPTool("mcp__database__query", {
          table: "products",
          limit: 500,
        }),
    },
    {
      name: "Get Analytics",
      call: () =>
        callMCPTool("mcp__analytics__getMetrics", {
          period: "last_7_days",
        }),
    },
    {
      name: "Load Config",
      call: () =>
        callMCPTool("mcp__config__get", {
          key: "app_settings",
        }),
    },
  ];

  console.log(`Executing ${operations.length} operations in parallel...`);

  // Execute all operations concurrently with Promise.allSettled
  // This allows us to handle partial failures gracefully
  const results = await Promise.allSettled(
    operations.map(async (op, index) => {
      const opStart = Date.now();
      console.log(`[${index + 1}/${operations.length}] Starting: ${op.name}`);

      try {
        const result = await op.call();
        const duration = Date.now() - opStart;
        console.log(
          `✓ [${
            index + 1
          }/${operations.length}] ${op.name} completed (${duration}ms)`,
        );

        return {
          name: op.name,
          success: true,
          data: result,
          duration,
        };
      } catch (error) {
        const duration = Date.now() - opStart;
        console.error(
          `✗ [${
            index + 1
          }/${operations.length}] ${op.name} failed (${duration}ms): ${
            (error as Error).message
          }`,
        );

        return {
          name: op.name,
          success: false,
          error: (error as Error).message,
          duration,
        };
      }
    }),
  );

  // Analyze results
  const completed = results.filter((r) =>
    r.status === "fulfilled" && (r.value as { success: boolean }).success
  );
  const failed = results.filter((r) =>
    r.status === "rejected" ||
    (r.status === "fulfilled" && !(r.value as { success: boolean }).success)
  );

  const successfulResults = completed.map((r) =>
    (r as PromiseFulfilledResult<unknown>).value
  );
  const errors = failed.map((r) => {
    if (r.status === "rejected") {
      return r.reason.message;
    } else {
      return (r as PromiseFulfilledResult<{ error: string }>).value.error;
    }
  });

  const totalDuration = Date.now() - startTime;

  // Calculate estimated sequential time (sum of individual durations)
  const estimatedSequentialTime: number = successfulResults.reduce(
    (sum: number, r) => sum + ((r as { duration?: number }).duration || 0),
    0,
  );
  const speedup = estimatedSequentialTime > 0
    ? `${(estimatedSequentialTime / totalDuration).toFixed(2)}x faster`
    : "N/A";

  console.log("\n=== Execution Summary ===");
  console.log(`Total time: ${totalDuration}ms`);
  console.log(`Estimated sequential time: ${estimatedSequentialTime}ms`);
  console.log(`Speedup: ${speedup}`);
  console.log(`Completed: ${completed.length}/${operations.length}`);
  console.log(`Failed: ${failed.length}/${operations.length}`);

  if (failed.length > 0) {
    console.warn("\nFailed operations:");
    errors.forEach((err, i) => console.warn(`  ${i + 1}. ${err}`));
  }

  return {
    success: failed.length === 0,
    completed: completed.length,
    failed: failed.length,
    results: successfulResults,
    errors,
    duration: totalDuration,
    speedup,
  };
}

// Alternative: Fail-fast approach using Promise.all
// Uncomment to use this pattern instead
/*
async function parallelExecutionFailFast() {
  console.log('=== Parallel Execution (Fail-Fast) ===');

  try {
    const [users, orders, products, analytics, config] = await Promise.all([
      callMCPTool('mcp__database__query', { table: 'users' }),
      callMCPTool('mcp__database__query', { table: 'orders' }),
      callMCPTool('mcp__database__query', { table: 'products' }),
      callMCPTool('mcp__analytics__getMetrics', { period: 'last_7_days' }),
      callMCPTool('mcp__config__get', { key: 'app_settings' })
    ]);

    console.log('All operations completed successfully');

    return {
      success: true,
      data: { users, orders, products, analytics, config }
    };
  } catch (error) {
    console.error('At least one operation failed:', (error as Error).message);
    // If any operation fails, all results are lost
    throw error;
  }
}
*/

// Execute
const result = await parallelExecution();

if (result.success) {
  console.log("\n✓ All operations completed successfully");
} else {
  console.log(`\n⚠ Completed with ${result.failed} failures`);
}
