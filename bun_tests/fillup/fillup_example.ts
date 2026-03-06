import { fillupCentralAccount, runFillup, experimentalFillupWithPayments, turboFillup } from "./fillup";

// Example 1: Basic usage with default 100 accounts
async function example1() {
  console.log("=== Example 1: Basic fillup with 100 accounts ===");
  
  try {
    const result = await fillupCentralAccount();
    console.log("\n🎉 Success!");
    console.log(`✅ Merged ${result.accountsMerged} accounts`);
    console.log(`💰 Final balance: ${result.finalBalance} XLM`);
    console.log(`📈 Estimated XLM added: ~${result.accountsMerged * 9999} XLM`);
  } catch (error) {
    console.error("❌ Example 1 failed:", error);
  }
}

// Example 2: Custom number of accounts (smaller test run)
async function example2() {
  console.log("\n=== Example 2: Test run with 10 accounts ===");
  
  try {
    const result = await fillupCentralAccount(10);
    console.log("\n🎉 Test run complete!");
    console.log(`✅ Merged ${result.accountsMerged} accounts`);
    console.log(`💰 Final balance: ${result.finalBalance} XLM`);
  } catch (error) {
    console.error("❌ Example 2 failed:", error);
  }
}

// Example 3: Using the convenience function
async function example3() {
  console.log("\n=== Example 3: Using runFillup convenience function ===");
  
  try {
    const result = await runFillup(25);
    console.log("\n🎉 Fillup complete!");
    console.log(`Result:`, result);
  } catch (error) {
    console.error("❌ Example 3 failed:", error);
  }
}

// Example 4: Progressive fillup (multiple smaller batches)
async function example4() {
  console.log("\n=== Example 4: Progressive fillup (5 batches of 20 accounts) ===");
  
  let totalMerged = 0;
  const batchSize = 20;
  const numBatches = 5;
  
  for (let i = 1; i <= numBatches; i++) {
    console.log(`\n--- Starting batch ${i}/${numBatches} ---`);
    try {
      const result = await fillupCentralAccount(batchSize);
      totalMerged += result.accountsMerged;
      console.log(`Batch ${i} complete: +${result.accountsMerged} accounts`);
      console.log(`Running total: ${totalMerged} accounts merged`);
      
      // Wait between batches to be extra gentle on the network
      if (i < numBatches) {
        console.log("Waiting 30 seconds before next batch...");
        await new Promise(resolve => setTimeout(resolve, 30000));
      }
    } catch (error) {
      console.error(`❌ Batch ${i} failed:`, error);
    }
  }
  
  console.log(`\n🎉 Progressive fillup complete!`);
  console.log(`📊 Total accounts merged across all batches: ${totalMerged}`);
  console.log(`💰 Estimated total XLM added: ~${totalMerged * 9999} XLM`);
}

// Example 5: Experimental ultra-fast batching (20 signatures per transaction)
async function example5() {
  console.log("\n=== Example 5: 🚀 Experimental 20-signatures-per-transaction batching ===");
  
  try {
    const result = await experimentalFillupWithPayments(100);
    console.log("\n🎉 Experimental batching complete!");
    console.log(`✅ Processed ${result.accountsProcessed} accounts`);
    console.log(`💰 Final balance: ${result.finalBalance} XLM`);
    console.log(`⚡ Used advanced batching with up to 20 signatures per transaction!`);
  } catch (error) {
    console.error("❌ Experimental example failed:", error);
  }
}

// Example 6: TURBO mode - Maximum throughput test
async function example6() {
  console.log("\n=== Example 6: 🚀 TURBO MODE - Maximum Throughput Test ===");
  
  try {
    // Test with 200 accounts to demonstrate multi-ledger capability
    const result = await turboFillup(200);
    console.log("\n🎉 TURBO mode complete!");
    console.log(`✅ Processed ${result.accountsMerged} accounts`);
    console.log(`💰 Final balance: ${result.finalBalance} XLM`);
    console.log(`⚡ Demonstrated 1000-tx-per-ledger optimization!`);
  } catch (error) {
    console.error("❌ TURBO example failed:", error);
  }
}

// Main execution function
async function main() {
  const example = process.argv[2] || "1";
  
  switch (example) {
    case "1":
      await example1();
      break;
    case "2":
      await example2();
      break;
    case "3":
      await example3();
      break;
    case "4":
      await example4();
      break;
    case "5":
      await example5();
      break;
    case "6":
      await example6();
      break;
    case "all":
      console.log("Running all examples (this will take a while!)...\n");
      await example2(); // Start with small test
      await new Promise(resolve => setTimeout(resolve, 5000)); // 5 sec break
      await example3(); // Medium test
      await new Promise(resolve => setTimeout(resolve, 10000)); // 10 sec break
      await example5(); // Experimental test
      await new Promise(resolve => setTimeout(resolve, 15000)); // 15 sec break
      await example1(); // Full test
      await new Promise(resolve => setTimeout(resolve, 20000)); // 20 sec break
      await example6(); // TURBO test (200 accounts)
      break;
    default:
      console.log("Usage:");
      console.log("  bun run fillup_example.ts [1|2|3|4|5|6|all]");
      console.log("");
      console.log("Examples:");
      console.log("  1 - ⚡ Ultra-fast parallel (100 accounts) [DEFAULT]");
      console.log("  2 - Test run (10 accounts)");
      console.log("  3 - Using convenience function (25 accounts)");
      console.log("  4 - Progressive fillup (5 batches of 20)");
      console.log("  5 - 🧪 Experimental 20-signatures-per-transaction batching");
      console.log("  6 - 🚀 TURBO MODE: Maximum throughput (200 accounts)");
      console.log("  all - Run all examples in sequence");
      return;
  }
}

// Run if called directly
if (import.meta.main) {
  main()
    .then(() => {
      console.log("\n✨ All examples completed!");
      process.exit(0);
    })
    .catch((error) => {
      console.error("\n💥 Fatal error:", error);
      process.exit(1);
    });
}

export { example1, example2, example3, example4, example5, example6 }; 