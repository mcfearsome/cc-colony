/**
 * Script: Multi-Tool Workflow
 * Purpose: Demonstrates sequential operations with data flow between tools
 * Use Case: Building data processing pipelines where each step depends on the previous
 * MCP Tools Used: mcp__filesystem__*, mcp__database__*, example tools
 *
 * How to Adapt:
 * 1. Replace example tool names with your actual MCP tools
 * 2. Modify the data transformation logic for your use case
 * 3. Adjust error handling based on your requirements
 * 4. Add or remove pipeline steps as needed
 * 5. Customize the output format
 *
 * Example Usage:
 * Processing workflow: Fetch → Transform → Validate → Store → Report
 *
 * Execution:
 * deno run --allow-read --allow-run --allow-env multi-tool-workflow.ts
 */

import { callMCPTool } from "../../lib/mcp-client.ts";

interface PipelineResult {
  success: boolean;
  stepsCompleted: number;
  totalSteps: number;
  data?: unknown;
  error?: string;
  duration: number;
}

async function multiToolWorkflow(): Promise<PipelineResult> {
  const startTime = Date.now();
  const totalSteps = 5;
  let stepsCompleted = 0;

  try {
    console.log("=== Starting Multi-Tool Workflow ===");

    // Step 1: Fetch data from source
    console.log("[Step 1/5] Fetching data from source...");
    const rawData = (await callMCPTool("mcp__database__query", {
      table: "source_data",
      limit: 100,
    })) as Record<string, unknown>[];
    stepsCompleted++;
    console.log(`✓ Fetched ${rawData.length} records`);

    // Step 2: Transform data
    console.log("[Step 2/5] Transforming data...");
    const transformed = rawData.map((
      record,
    ) => ({
      id: record.id,
      name: record.full_name || record.name,
      email: (record.email as string).toLowerCase(),
      created_at: new Date(record.timestamp as string).toISOString(),
      status: record.is_active ? "active" : "inactive",
    }));
    stepsCompleted++;
    console.log(`✓ Transformed ${transformed.length} records`);

    // Step 3: Validate data
    console.log("[Step 3/5] Validating data...");
    const validated = transformed.filter((record) => {
      const hasRequiredFields = record.id && record.name && record.email;
      const validEmail = (record.email as string).includes("@");
      return hasRequiredFields && validEmail;
    });

    const invalidCount = transformed.length - validated.length;
    if (invalidCount > 0) {
      console.warn(`⚠ Filtered out ${invalidCount} invalid records`);
    }
    stepsCompleted++;
    console.log(`✓ Validated ${validated.length} records`);

    // Step 4: Store processed data
    console.log("[Step 4/5] Storing processed data...");
    const storeResult = (await callMCPTool("mcp__database__bulkInsert", {
      table: "processed_data",
      records: validated,
    })) as { inserted: number };
    stepsCompleted++;
    console.log(`✓ Stored ${storeResult.inserted} records`);

    // Step 5: Generate report
    console.log("[Step 5/5] Generating report...");
    const report = {
      timestamp: new Date().toISOString(),
      source_records: rawData.length,
      transformed: transformed.length,
      validated: validated.length,
      invalid: invalidCount,
      stored: storeResult.inserted,
      success_rate: ((validated.length / rawData.length) * 100).toFixed(2) +
        "%",
    };

    await callMCPTool("mcp__filesystem__writeFile", {
      path: "/tmp/workflow-report.json",
      content: JSON.stringify(report, null, 2),
    });
    stepsCompleted++;
    console.log("✓ Report saved to /tmp/workflow-report.json");

    const duration = Date.now() - startTime;
    console.log(`=== Workflow Complete (${duration}ms) ===`);

    return {
      success: true,
      stepsCompleted,
      totalSteps,
      data: report,
      duration,
    };
  } catch (error) {
    console.error(
      `✗ Workflow failed at step ${stepsCompleted + 1}/${totalSteps}`,
    );
    console.error("Error:", (error as Error).message);

    return {
      success: false,
      stepsCompleted,
      totalSteps,
      error: (error as Error).message,
      duration: Date.now() - startTime,
    };
  }
}

// Execute workflow
const result = await multiToolWorkflow();
console.log("Final result:", JSON.stringify(result, null, 2));
