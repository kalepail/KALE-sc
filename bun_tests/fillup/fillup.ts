import {
  Keypair,
  Networks,
  TransactionBuilder,
  Operation,
  Account,
  BASE_FEE,
  Horizon,
  Asset,
} from "@stellar/stellar-sdk";

// Configuration
const CENTRAL_ACCOUNT = "GAWV44MH2SGZ4W5DZ43QM7K3FCK4DUAJY3FKUWJB4AXWQENC2HHRMMJ3";
const TESTNET_HORIZON = "https://horizon-futurenet.stellar.org";
const FRIENDBOT_URL = "https://friendbot-futurenet.stellar.org";
const BATCH_SIZE = 20; // Number of accounts to create in parallel
const MERGE_BATCH_SIZE = 100; // Submit all transactions in parallel (unique sources)

// Initialize Horizon server
const server = new Horizon.Server(TESTNET_HORIZON);

// Helper function to fund an account using friendbot
async function fundWithFriendbot(publicKey: string): Promise<void> {
  const response = await fetch(`${FRIENDBOT_URL}?addr=${publicKey}`);
  if (!response.ok) {
    throw new Error(`Friendbot funding failed: ${response.statusText}`);
  }
  console.log(`Funded account: ${publicKey}`);
}

// Helper function to wait (for rate limiting)
const wait = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

// Main function to create and merge accounts
export async function fillupCentralAccount(accountsToCreate: number = 100) {
  console.log(`Starting fillup process for ${accountsToCreate} accounts...`);
  
  try {
    // Step 1: Generate keypairs for all accounts
    console.log("\n1. Generating keypairs...");
    const accounts: Keypair[] = [];
    for (let i = 0; i < accountsToCreate; i++) {
      accounts.push(Keypair.random());
    }
    console.log(`Generated ${accounts.length} keypairs`);

    // Step 2: Fund accounts with friendbot in batches
    console.log("\n2. Funding accounts with friendbot...");
    for (let i = 0; i < accounts.length; i += BATCH_SIZE) {
      const batch = accounts.slice(i, Math.min(i + BATCH_SIZE, accounts.length));
      
      await Promise.all(
        batch.map(async (keypair, index) => {
          try {
            // Add some jitter to avoid overwhelming friendbot
            await wait(index * 100);
            await fundWithFriendbot(keypair.publicKey());
          } catch (error) {
            console.error(`Failed to fund account ${keypair.publicKey()}: ${error}`);
            throw error;
          }
        })
      );
      
      console.log(`Funded ${Math.min(i + BATCH_SIZE, accounts.length)}/${accounts.length} accounts`);
      
      // Wait between batches to avoid rate limiting
      if (i + BATCH_SIZE < accounts.length) {
        await wait(2000);
      }
    }

    // Step 3: Load central account information
    console.log("\n3. Loading central account information...");
    const centralAccount = await server.loadAccount(CENTRAL_ACCOUNT);
    console.log(`Central account loaded. Current balance: ${centralAccount.balances.find(b => b.asset_type === 'native')?.balance} XLM`);

    // Step 4: Merge accounts into central account (ULTRA-FAST mode)
    console.log("\n4. 🚀 ULTRA-FAST: Submitting ALL merge transactions simultaneously...");
    console.log(`Leveraging Stellar's 1000-tx-per-ledger limit with ${accounts.length} unique sources`);
    
    let totalMerged = 0;
    
    // Submit ALL merge transactions in parallel - Stellar will batch across ledgers
    console.log(`Submitting ${accounts.length} merge transactions simultaneously...`);
    
    const mergePromises = accounts.map(async (sourceKeypair, index) => {
      try {
        // Minimal stagger (10ms) just to avoid overwhelming the HTTP client
        await wait(index * 10);
        
        // Load the source account
        const sourceAccount = await server.loadAccount(sourceKeypair.publicKey());
        
        // Build the merge transaction
        const transaction = new TransactionBuilder(sourceAccount, {
          fee: BASE_FEE,
          networkPassphrase: Networks.FUTURENET,
        })
          .addOperation(
            Operation.accountMerge({
              destination: CENTRAL_ACCOUNT,
            })
          )
          .setTimeout(60) // Longer timeout for high-throughput mode
          .build();
        
        // Sign with the source account (which will be merged)
        transaction.sign(sourceKeypair);
        
        // Submit the transaction (all in parallel!)
        await server.submitTransaction(transaction);
        
        // Log every 10th success to avoid spam
        if ((index + 1) % 10 === 0) {
          console.log(`✅ ${index + 1}/${accounts.length} transactions submitted...`);
        }
        
        return { success: true, publicKey: sourceKeypair.publicKey() };
      } catch (error) {
        console.error(`❌ Failed to merge account ${sourceKeypair.publicKey()}: ${error}`);
        return { success: false, publicKey: sourceKeypair.publicKey(), error };
      }
    });
    
    // Wait for ALL transactions to complete
    console.log(`⏳ Waiting for all ${accounts.length} transactions to complete...`);
    const allResults = await Promise.all(mergePromises);
    
    // Count successes and failures
    const successCount = allResults.filter(r => r.success).length;
    const failureCount = allResults.filter(r => !r.success).length;
    totalMerged = successCount;
    
    console.log(`\n🎯 PARALLEL SUBMISSION COMPLETE:`);
    console.log(`✅ Success: ${successCount}/${accounts.length} accounts merged`);
    console.log(`❌ Failed: ${failureCount}/${accounts.length} accounts`);
    
    if (failureCount > 0) {
      console.log(`\n🔄 Failed accounts (check logs above for details):`);
      allResults.filter(r => !r.success).forEach(r => 
        console.log(`  - ${r.publicKey}`)
      );
    }

    // Step 5: Check final balance
    console.log("\n5. Checking final balance...");
    const finalCentralAccount = await server.loadAccount(CENTRAL_ACCOUNT);
    const finalBalance = finalCentralAccount.balances.find(b => b.asset_type === 'native')?.balance;
    
    console.log(`\n✅ Fillup complete!`);
    console.log(`Total accounts merged: ${totalMerged}/${accountsToCreate}`);
    console.log(`Final central account balance: ${finalBalance} XLM`);
    console.log(`Approximate XLM added: ${totalMerged * 9999} XLM`); // ~9999 XLM per account after fees

    return {
      accountsMerged: totalMerged,
      finalBalance: finalBalance,
      centralAccount: CENTRAL_ACCOUNT,
    };
    
  } catch (error) {
    console.error("Error in fillup process:", error);
    throw error;
  }
}

// Function to run with custom number of accounts
export async function runFillup(numAccounts?: number) {
  const accounts = numAccounts || 100;
  console.log(`Creating and merging ${accounts} accounts...`);
  return await fillupCentralAccount(accounts);
}

// EXPERIMENTAL: Ultra-high throughput merge using payment operations
// Note: This is an alternative approach that uses payments instead of account merge
// to work around the limitation that account merge operations consume the source account
export async function experimentalFillupWithPayments(accountsToCreate: number = 100) {
  console.log(`🧪 EXPERIMENTAL: Starting payment-based fillup for ${accountsToCreate} accounts...`);
  
  try {
    // Step 1: Generate keypairs and fund with friendbot (same as before)
    console.log("\n1. Generating keypairs...");
    const accounts: Keypair[] = [];
    for (let i = 0; i < accountsToCreate; i++) {
      accounts.push(Keypair.random());
    }
    
    console.log("\n2. Funding accounts with friendbot...");
    for (let i = 0; i < accounts.length; i += BATCH_SIZE) {
      const batch = accounts.slice(i, Math.min(i + BATCH_SIZE, accounts.length));
      
      await Promise.all(
        batch.map(async (keypair, index) => {
          try {
            await wait(index * 100);
            await fundWithFriendbot(keypair.publicKey());
          } catch (error) {
            console.error(`Failed to fund account ${keypair.publicKey()}: ${error}`);
            throw error;
          }
        })
      );
      
      console.log(`Funded ${Math.min(i + BATCH_SIZE, accounts.length)}/${accounts.length} accounts`);
      if (i + BATCH_SIZE < accounts.length) {
        await wait(2000);
      }
    }

    // Step 3: ULTRA-FAST batched payments (all batches submitted simultaneously)
    console.log("\n3. 🚀 ULTRA-FAST: Submitting ALL batched transactions simultaneously...");
    let totalProcessed = 0;
    const maxAccountsPerTx = 20; // Limited by Stellar's 20 signature limit
    
    // Create all batches first
    const batches = [];
    for (let i = 0; i < accounts.length; i += maxAccountsPerTx) {
      batches.push(accounts.slice(i, Math.min(i + maxAccountsPerTx, accounts.length)));
    }
    
    console.log(`Creating ${batches.length} batched transactions (${maxAccountsPerTx} accounts each)...`);
    
    // Submit all batched transactions in parallel
    const batchPromises = batches.map(async (batch, batchIndex) => {
      try {
        // Minimal delay to stagger submissions
        await wait(batchIndex * 20);
        
        // Use the first account in the batch as the transaction source
        const sourceAccount = await server.loadAccount(batch[0].publicKey());
        
        // Build transaction with multiple payment operations
        let txBuilder = new TransactionBuilder(sourceAccount, {
          fee: (parseInt(BASE_FEE) * batch.length).toString(), // Increase fee for multi-op transaction
          networkPassphrase: Networks.FUTURENET,
        });
        
        // Add payment operations for each account in the batch
        for (const account of batch) {
          // Load account to get current balance
          const accountData = await server.loadAccount(account.publicKey());
          const balance = accountData.balances.find(b => b.asset_type === 'native')?.balance;
          
          if (balance && parseFloat(balance) > 2) {
            // Send most of the balance, keeping 2 XLM for fees and minimum balance
            const amountToSend = (parseFloat(balance) - 2).toFixed(7);
            
            txBuilder = txBuilder.addOperation(
              Operation.payment({
                destination: CENTRAL_ACCOUNT,
                asset: Asset.native(),
                amount: amountToSend,
                source: account.publicKey(),
              })
            );
          }
        }
        
        const transaction = txBuilder.setTimeout(60).build();
        
        // Sign with all accounts in the batch
        batch.forEach(keypair => transaction.sign(keypair));
        
        // Submit the batched transaction
        await server.submitTransaction(transaction);
        
        console.log(`✅ Batch ${batchIndex + 1}/${batches.length} complete: ${batch.length} accounts processed`);
        return { success: true, accountsProcessed: batch.length, batchIndex };
        
      } catch (error) {
        console.error(`❌ Batch ${batchIndex + 1} failed, falling back to individual merges:`, error);
        
        // Fallback to individual transactions for this batch
        let individualSuccess = 0;
        for (const keypair of batch) {
          try {
            const sourceAccount = await server.loadAccount(keypair.publicKey());
            const transaction = new TransactionBuilder(sourceAccount, {
              fee: BASE_FEE,
              networkPassphrase: Networks.FUTURENET,
            })
              .addOperation(Operation.accountMerge({ destination: CENTRAL_ACCOUNT }))
              .setTimeout(60)
              .build();
            
            transaction.sign(keypair);
            await server.submitTransaction(transaction);
            individualSuccess++;
          } catch (fallbackError) {
            console.error(`Failed individual fallback for ${keypair.publicKey()}:`, fallbackError);
          }
        }
        
        console.log(`🔄 Batch ${batchIndex + 1} fallback: ${individualSuccess}/${batch.length} accounts recovered`);
        return { success: false, accountsProcessed: individualSuccess, batchIndex };
      }
    });
    
    // Wait for all batched transactions to complete
    console.log(`⏳ Waiting for all ${batches.length} batched transactions to complete...`);
    const batchResults = await Promise.all(batchPromises);
    
    // Count total processed
    totalProcessed = batchResults.reduce((sum, result) => sum + result.accountsProcessed, 0);
    const successfulBatches = batchResults.filter(r => r.success).length;
    
    console.log(`\n🎯 BATCHED SUBMISSION COMPLETE:`);
    console.log(`✅ Successful batches: ${successfulBatches}/${batches.length}`);
    console.log(`📊 Total accounts processed: ${totalProcessed}/${accounts.length}`)

    // Final balance check
    const finalCentralAccount = await server.loadAccount(CENTRAL_ACCOUNT);
    const finalBalance = finalCentralAccount.balances.find(b => b.asset_type === 'native')?.balance;
    
    console.log(`\n🎉 Experimental fillup complete!`);
    console.log(`Total accounts processed: ${totalProcessed}/${accountsToCreate}`);
    console.log(`Final central account balance: ${finalBalance} XLM`);
    
    return {
      accountsProcessed: totalProcessed,
      finalBalance: finalBalance,
      centralAccount: CENTRAL_ACCOUNT,
    };
    
  } catch (error) {
    console.error("Error in experimental fillup:", error);
    throw error;
  }
}

// TURBO MODE: Maximum theoretical throughput using 1000-tx-per-ledger limit
export async function turboFillup(accountsToCreate: number = 1000) {
  console.log(`🚀 TURBO MODE: Creating ${accountsToCreate} accounts (max throughput)`);
  console.log(`⚡ Targeting Stellar's 1000-tx-per-ledger theoretical limit`);
  
  if (accountsToCreate > 1000) {
    console.log(`⚠️  Note: ${accountsToCreate} accounts will span multiple ledgers`);
  }
  
  // Use the ultra-fast standard mode since each account has unique source
  return await fillupCentralAccount(accountsToCreate);
}

// Run if called directly
if (import.meta.main) {
  const numAccounts = process.argv[2] ? parseInt(process.argv[2]) : 100;
  const mode = process.argv[3] || "standard";
  
  let fillupFunction;
  let modeDescription;
  
  switch(mode) {
    case "--experimental":
      fillupFunction = experimentalFillupWithPayments;
      modeDescription = "🧪 Experimental batched payment approach";
      break;
    case "--turbo":
      fillupFunction = turboFillup;
      modeDescription = "🚀 TURBO MODE: Maximum throughput (1000-tx capability)";
      break;
    default:
      fillupFunction = runFillup;
      modeDescription = "⚡ Ultra-fast parallel submission mode";
      break;
  }
  
  console.log(modeDescription);
  
  fillupFunction(numAccounts)
    .then(() => process.exit(0))
    .catch((error) => {
      console.error("Fatal error:", error);
      process.exit(1);
    });
}
