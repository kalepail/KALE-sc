import { Horizon, Asset, Networks } from '@stellar/stellar-sdk';
import { readFileSync, writeFileSync } from 'fs';
import { Server as SorobanServer } from '@stellar/stellar-sdk/rpc';

// Connect to Stellar Futurenet
const horizonServer = new Horizon.Server('https://horizon-futurenet.stellar.org');
const sorobanServer = new SorobanServer('https://rpc-futurenet.stellar.org');

// Configuration
const CONFIG = {
  BATCH_SIZE: 10,             // Reduced from 50 to avoid rate limits
  RATE_LIMIT_DELAY: 1000,     // Increased from 100ms to 1 second
  RETRY_DELAY: 5000,          // 5 seconds wait on rate limit
  MAX_RETRIES: 3,             // Maximum retries per address
  
  // TODO: Update this with the actual KALE token contract address
  // This needs to be identified from the 8 KALE contracts
  KALE_TOKEN_CONTRACT: process.env.KALE_TOKEN_CONTRACT || '',
  
  // Alternative: Use traditional asset code and issuer format if KALE is a classic asset
  KALE_ASSET_CODE: process.env.KALE_ASSET_CODE || 'KALE',
  KALE_ASSET_ISSUER: process.env.KALE_ASSET_ISSUER || ''
};

interface BalanceRecord {
  address: string;
  balance: string;
  balance_numeric: number;
}

// Add delay to avoid rate limiting
const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

// Format balance for display (assuming 7 decimal places for Stellar assets)
function formatBalance(balance: string): number {
  return parseFloat(balance);
}

// Method 1: Check if KALE is a traditional Stellar asset
async function getTraditionalAssetBalance(address: string): Promise<string | null> {
  try {
    const account = await horizonServer.loadAccount(address);
    
    // Look for KALE balance in the account's balances
    for (const balance of account.balances) {
      if (balance.asset_type !== 'native' && 
          'asset_code' in balance &&
          balance.asset_code === CONFIG.KALE_ASSET_CODE && 
          balance.asset_issuer === CONFIG.KALE_ASSET_ISSUER) {
        return balance.balance;
      }
    }
    
    return '0';
  } catch (error: any) {
    if (error?.response?.status === 404) {
      // Account doesn't exist on-chain
      return null;
    }
    throw error;
  }
}

// Method 2: Check if KALE is a Stellar Asset Contract (SAC)
async function getSACBalance(address: string): Promise<string | null> {
  if (!CONFIG.KALE_TOKEN_CONTRACT) {
    return null;
  }
  
  try {
    // For G addresses, we need to convert to a contract address format
    // The SAC creates a virtual contract for each account
    const asset = new Asset(CONFIG.KALE_ASSET_CODE, CONFIG.KALE_ASSET_ISSUER);
    
    const result = await sorobanServer.getSACBalance(
      address,
      asset,
      Networks.FUTURENET
    );
    
    if (result.balanceEntry) {
      // Convert from stroops to decimal format
      const balance = BigInt(result.balanceEntry.amount);
      return (Number(balance) / 10000000).toFixed(7);
    }
    
    return '0';
  } catch (error) {
    // If the account has no balance entry, return 0
    return '0';
  }
}

// Flexible method that tries both approaches
async function getKaleBalance(address: string, retryCount: number = 0): Promise<string | null> {
  try {
    // First try traditional asset approach
    if (CONFIG.KALE_ASSET_ISSUER) {
      const traditionalBalance = await getTraditionalAssetBalance(address);
      if (traditionalBalance !== null && traditionalBalance !== '0') {
        return traditionalBalance;
      }
    }
    
    // Then try SAC approach
    if (CONFIG.KALE_TOKEN_CONTRACT) {
      const sacBalance = await getSACBalance(address);
      if (sacBalance !== null) {
        return sacBalance;
      }
    }
    
    // If both methods fail, check if it's a valid account
    try {
      await horizonServer.loadAccount(address);
      return '0'; // Account exists but has no KALE
    } catch (error: any) {
      if (error?.response?.status === 404) {
        return null; // Account doesn't exist
      }
      throw error;
    }
  } catch (error: any) {
    // Handle rate limiting
    if (error?.response?.status === 429 && retryCount < CONFIG.MAX_RETRIES) {
      console.log(`⏳ Rate limited for ${address}, waiting ${CONFIG.RETRY_DELAY}ms before retry ${retryCount + 1}/${CONFIG.MAX_RETRIES}...`);
      await delay(CONFIG.RETRY_DELAY);
      return getKaleBalance(address, retryCount + 1);
    }
    throw error;
  }
}

async function main() {
  try {
    console.log('KALE G-Address Balance Checker\n');
    console.log('Configuration:');
    console.log(`  KALE Asset Code: ${CONFIG.KALE_ASSET_CODE}`);
    console.log(`  KALE Asset Issuer: ${CONFIG.KALE_ASSET_ISSUER || 'Not specified'}`);
    console.log(`  KALE Token Contract: ${CONFIG.KALE_TOKEN_CONTRACT || 'Not specified'}`);
    console.log(`  Batch Size: ${CONFIG.BATCH_SIZE}`);
    console.log(`  Rate Limit Delay: ${CONFIG.RATE_LIMIT_DELAY}ms`);
    console.log();
    
    // Read G addresses from CSV
    const csvContent = readFileSync('g_addresses.csv', 'utf-8');
    const addresses = csvContent.trim().split('\n').filter(a => a);
    
    console.log(`Found ${addresses.length} unique G addresses to check\n`);
    
    const balances: BalanceRecord[] = [];
    let processed = 0;
    let skipped = 0;
    let totalWithBalance = 0;
    let errors = 0;
    
    // Estimate time
    const estimatedMinutes = Math.ceil((addresses.length / CONFIG.BATCH_SIZE) * (CONFIG.RATE_LIMIT_DELAY / 1000) / 60);
    console.log(`⏱️  Estimated time: ~${estimatedMinutes} minutes\n`);
    
    // Process addresses in batches
    for (let i = 0; i < addresses.length; i += CONFIG.BATCH_SIZE) {
      const batch = addresses.slice(i, Math.min(i + CONFIG.BATCH_SIZE, addresses.length));
      const batchNumber = Math.floor(i / CONFIG.BATCH_SIZE) + 1;
      const totalBatches = Math.ceil(addresses.length / CONFIG.BATCH_SIZE);
      
      console.log(`\n📦 Processing batch ${batchNumber}/${totalBatches}...`);
      
      const batchPromises = batch.map(async (address) => {
        try {
          const balance = await getKaleBalance(address);
          
          if (balance === null) {
            skipped++;
            console.log(`⚠️  Skipped ${address} (account not found)`);
            return null;
          }
          
          const numericBalance = formatBalance(balance);
          
          if (numericBalance > 0) {
            totalWithBalance++;
            console.log(`✓  ${address}: ${balance} KALE`);
          }
          
          return {
            address,
            balance,
            balance_numeric: numericBalance
          };
        } catch (error: any) {
          errors++;
          console.error(`❌ Error checking ${address}:`, error.message || error);
          return null;
        }
      });
      
      const batchResults = await Promise.all(batchPromises);
      
      // Add non-null results to balances
      batchResults.forEach(result => {
        if (result !== null) {
          balances.push(result);
        }
      });
      
      processed += batch.length;
      
      // Progress update
      const progress = ((processed / addresses.length) * 100).toFixed(1);
      console.log(`\nProgress: ${processed}/${addresses.length} (${progress}%) - ${totalWithBalance} with balance, ${errors} errors`);
      
      // Rate limiting between batches
      if (i + CONFIG.BATCH_SIZE < addresses.length) {
        console.log(`⏳ Rate limiting: waiting ${CONFIG.RATE_LIMIT_DELAY}ms before next batch...`);
        await delay(CONFIG.RATE_LIMIT_DELAY);
      }
    }
    
    // Sort by balance (descending)
    balances.sort((a, b) => b.balance_numeric - a.balance_numeric);
    
    // Save leaderboard CSV
    const leaderboardCSV = [
      'address,balance',
      ...balances.map(b => `${b.address},${b.balance}`)
    ].join('\n');
    
    writeFileSync('g_addresses_leaderboard.csv', leaderboardCSV);
    console.log(`\n✅ Saved leaderboard to: g_addresses_leaderboard.csv`);
    
    // Save summary
    const totalBalance = balances.reduce((sum, b) => sum + b.balance_numeric, 0);
    console.log('\nSummary:');
    console.log(`  Total addresses checked: ${addresses.length}`);
    console.log(`  Addresses with balance: ${totalWithBalance}`);
    console.log(`  Addresses skipped: ${skipped}`);
    console.log(`  Errors encountered: ${errors}`);
    console.log(`  Total KALE balance: ${totalBalance.toFixed(7)}`);
    
    if (balances.length > 0) {
      console.log('\nTop 10 KALE holders:');
      balances.slice(0, 10).forEach((b, i) => {
        console.log(`  ${i + 1}. ${b.address}: ${b.balance} KALE`);
      });
    }
    
    // Warning if configuration might be incomplete
    if (!CONFIG.KALE_ASSET_ISSUER && !CONFIG.KALE_TOKEN_CONTRACT) {
      console.log('\n⚠️  WARNING: Neither KALE_ASSET_ISSUER nor KALE_TOKEN_CONTRACT is configured.');
      console.log('   Please set the appropriate environment variables:');
      console.log('   - For traditional asset: KALE_ASSET_CODE and KALE_ASSET_ISSUER');
      console.log('   - For SAC token: KALE_TOKEN_CONTRACT');
    }
    
  } catch (error) {
    console.error('Fatal error:', error);
  }
}

// Run the script
main(); 