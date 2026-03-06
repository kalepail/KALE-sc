import { Asset, Networks, Contract, Address } from '@stellar/stellar-sdk';
import { readFileSync, writeFileSync } from 'fs';
import { Server as SorobanServer, Durability } from '@stellar/stellar-sdk/rpc';
import { xdr } from '@stellar/stellar-sdk';

// Connect to Stellar Futurenet
const sorobanServer = new SorobanServer('https://rpc-futurenet.stellar.org');

// Configuration
const CONFIG = {
  BATCH_SIZE: 5,              // Reduced from 20 to avoid rate limits
  RATE_LIMIT_DELAY: 2000,     // Increased from 200ms to 2 seconds (more conservative for RPC)
  RETRY_DELAY: 10000,         // 10 seconds wait on rate limit (RPC needs longer)
  MAX_RETRIES: 3,             // Maximum retries per address
  
  // TODO: Update this with the actual KALE token contract address
  // This needs to be identified from the 8 KALE contracts
  KALE_TOKEN_CONTRACT: process.env.KALE_TOKEN_CONTRACT || '',
  
  // For traditional SAC assets
  KALE_ASSET_CODE: process.env.KALE_ASSET_CODE || 'KALE',
  KALE_ASSET_ISSUER: process.env.KALE_ASSET_ISSUER || '',
  
  // The 8 KALE contracts from leaderboard (farming/homestead contracts)
  KALE_FARMING_CONTRACTS: [
    'CDVAFNGN7SBQBWJJWHZ4FESLK6PDLRC6CBLDLDQGZ3KX6XCEYW3XHDJ7',
    'CBYTMSEIA2S7S32JS2GHJA6ALCCIYVA52S5VZIIGGUBQ2C2KRJMHVDCF',
    'CDAG7OZW66ODS2PZOVM54HWUCPSBAVK6VSTTURX2WWLWAVW6N3ISSYM3',
    'CDDK5NTU4ZXT44A4ER25HTWSPKYUYVAPIITFDO5LZB7KJXKNMIP3NUN2',
    'CBOY65CSBDJTXV4R4HXSDYUFD2AB4KPS2WLPLNB7BKLMD4SPYU5LT4ZQ',
    'CCMEQJHYZX2O7KBVRL7MZEVT56T4VDC3ZFIZ6JTEIDMGWMRNZJJZ3ZDA',
    'CCGCFEUX6E7CQJQR4Q74NRK2EUTE6KLJROEVFTLZSOEGOQG3B4PYROU6',
    'CBNCFROZILPWNVSF2OLBS6TI3DWQ3IM4C53V4GHCIO57MVWUQP6EXYPT'
  ]
};

interface BalanceRecord {
  address: string;
  balance: string;
  balance_numeric: number;
  is_kale_contract: boolean;
}

// Add delay to avoid rate limiting
const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

// Check if this is one of our known KALE farming contracts
function isKaleFarmingContract(address: string): boolean {
  return CONFIG.KALE_FARMING_CONTRACTS.includes(address);
}

// Method 1: Get balance using getSACBalance for SAC tokens
async function getSACBalanceForContract(contractAddress: string): Promise<string | null> {
  if (!CONFIG.KALE_ASSET_ISSUER) {
    return null;
  }
  
  try {
    const asset = new Asset(CONFIG.KALE_ASSET_CODE, CONFIG.KALE_ASSET_ISSUER);
    
    const result = await sorobanServer.getSACBalance(
      contractAddress,
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
    // If the contract has no balance entry, return 0
    return '0';
  }
}

// Method 2: Query contract data directly if KALE uses custom storage
async function getCustomContractBalance(contractAddress: string): Promise<string | null> {
  if (!CONFIG.KALE_TOKEN_CONTRACT) {
    return null;
  }
  
  try {
    // Build the key for balance storage
    // This assumes a standard token contract storage pattern
    // Key structure: Balance(Address)
    
    const addressScVal = Address.fromString(contractAddress).toScVal();
    const key = xdr.ScVal.scvVec([
      xdr.ScVal.scvSymbol('Balance'),
      addressScVal
    ]);
    
    const result = await sorobanServer.getContractData(
      CONFIG.KALE_TOKEN_CONTRACT,
      key,
      Durability.Persistent
    );
    
    if (result.liveUntilLedgerSeq) {
      // For contract data entries, we need to access the value properly
      // This is a placeholder - actual implementation depends on KALE token structure
      return '0';
    }
    
    return '0';
  } catch (error) {
    return '0';
  }
}

// Method 3: For KALE farming contracts, check internal balances
async function getFarmingContractBalance(contractAddress: string): Promise<string | null> {
  try {
    // For farming contracts, we might need to check their internal state
    // This could be stored in various ways depending on the contract design
    
    // Try to get the 'asset' storage key which might hold the token contract
    const assetKey = xdr.ScVal.scvSymbol('asset');
    
    const assetResult = await sorobanServer.getContractData(
      contractAddress,
      assetKey,
      Durability.Temporary
    );
    
    if (assetResult.liveUntilLedgerSeq) {
      // Now we know which asset this farming contract uses
      // We can check its balance of that asset
      // This is a simplified approach - actual implementation may vary
      return '0'; // Placeholder - would need actual balance check
    }
    
    return '0';
  } catch (error) {
    return '0';
  }
}

// Main function to get KALE balance for a contract with retry
async function getContractKaleBalance(contractAddress: string, retryCount: number = 0): Promise<string> {
  try {
    // Try SAC balance first (most common for token holdings)
    const sacBalance = await getSACBalanceForContract(contractAddress);
    if (sacBalance && sacBalance !== '0') {
      return sacBalance;
    }
    
    // Try custom contract storage
    const customBalance = await getCustomContractBalance(contractAddress);
    if (customBalance && customBalance !== '0') {
      return customBalance;
    }
    
    // For KALE farming contracts, check internal balances
    if (isKaleFarmingContract(contractAddress)) {
      const farmingBalance = await getFarmingContractBalance(contractAddress);
      if (farmingBalance) {
        return farmingBalance;
      }
    }
    
    return '0';
  } catch (error: any) {
    // Handle rate limiting
    if ((error?.response?.status === 429 || error?.message?.includes('rate')) && retryCount < CONFIG.MAX_RETRIES) {
      console.log(`⏳ Rate limited for ${contractAddress}, waiting ${CONFIG.RETRY_DELAY}ms before retry ${retryCount + 1}/${CONFIG.MAX_RETRIES}...`);
      await delay(CONFIG.RETRY_DELAY);
      return getContractKaleBalance(contractAddress, retryCount + 1);
    }
    throw error;
  }
}

async function main() {
  try {
    console.log('KALE C-Address (Contract) Balance Checker\n');
    console.log('Configuration:');
    console.log(`  KALE Asset Code: ${CONFIG.KALE_ASSET_CODE}`);
    console.log(`  KALE Asset Issuer: ${CONFIG.KALE_ASSET_ISSUER || 'Not specified'}`);
    console.log(`  KALE Token Contract: ${CONFIG.KALE_TOKEN_CONTRACT || 'Not specified'}`);
    console.log(`  Known KALE Farming Contracts: ${CONFIG.KALE_FARMING_CONTRACTS.length}`);
    console.log(`  Batch Size: ${CONFIG.BATCH_SIZE}`);
    console.log(`  Rate Limit Delay: ${CONFIG.RATE_LIMIT_DELAY}ms`);
    console.log();
    
    // Read C addresses from CSV
    const csvContent = readFileSync('c_addresses.csv', 'utf-8');
    const addresses = csvContent.trim().split('\n').filter(a => a);
    
    console.log(`Found ${addresses.length} unique C addresses to check\n`);
    
    const balances: BalanceRecord[] = [];
    let processed = 0;
    let totalWithBalance = 0;
    let kaleFarmingContracts = 0;
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
          const isKaleContract = isKaleFarmingContract(address);
          if (isKaleContract) {
            kaleFarmingContracts++;
          }
          
          const balance = await getContractKaleBalance(address);
          const numericBalance = parseFloat(balance);
          
          if (numericBalance > 0) {
            totalWithBalance++;
            console.log(`✓  ${address}: ${balance} KALE${isKaleContract ? ' (KALE Contract)' : ''}`);
          } else if (isKaleContract) {
            console.log(`○  ${address}: ${balance} KALE (KALE Contract)`);
          }
          
          return {
            address,
            balance,
            balance_numeric: numericBalance,
            is_kale_contract: isKaleContract
          };
        } catch (error: any) {
          errors++;
          console.error(`❌ Error checking ${address}:`, error.message || error);
          return {
            address,
            balance: '0',
            balance_numeric: 0,
            is_kale_contract: isKaleFarmingContract(address)
          };
        }
      });
      
      const batchResults = await Promise.all(batchPromises);
      balances.push(...batchResults);
      
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
      'address,balance,is_kale_contract',
      ...balances.map(b => `${b.address},${b.balance},${b.is_kale_contract}`)
    ].join('\n');
    
    writeFileSync('c_addresses_leaderboard.csv', leaderboardCSV);
    console.log(`\n✅ Saved leaderboard to: c_addresses_leaderboard.csv`);
    
    // Save summary
    const totalBalance = balances.reduce((sum, b) => sum + b.balance_numeric, 0);
    const kaleContractBalances = balances
      .filter(b => b.is_kale_contract)
      .reduce((sum, b) => sum + b.balance_numeric, 0);
    
    console.log('\nSummary:');
    console.log(`  Total contracts checked: ${addresses.length}`);
    console.log(`  Contracts with balance: ${totalWithBalance}`);
    console.log(`  KALE farming contracts found: ${kaleFarmingContracts}`);
    console.log(`  Errors encountered: ${errors}`);
    console.log(`  Total KALE balance: ${totalBalance.toFixed(7)}`);
    console.log(`  KALE in farming contracts: ${kaleContractBalances.toFixed(7)}`);
    
    if (balances.length > 0) {
      console.log('\nTop 10 KALE-holding contracts:');
      balances
        .filter(b => b.balance_numeric > 0)
        .slice(0, 10)
        .forEach((b, i) => {
          const label = b.is_kale_contract ? ' (KALE Contract)' : '';
          console.log(`  ${i + 1}. ${b.address}: ${b.balance} KALE${label}`);
        });
    }
    
    // Show KALE farming contracts separately
    const farmingContracts = balances.filter(b => b.is_kale_contract);
    if (farmingContracts.length > 0) {
      console.log('\nKALE Farming Contracts:');
      farmingContracts.forEach(b => {
        console.log(`  ${b.address}: ${b.balance} KALE`);
      });
    }
    
    // Warning if configuration might be incomplete
    if (!CONFIG.KALE_ASSET_ISSUER && !CONFIG.KALE_TOKEN_CONTRACT) {
      console.log('\n⚠️  WARNING: Neither KALE_ASSET_ISSUER nor KALE_TOKEN_CONTRACT is configured.');
      console.log('   Please set the appropriate environment variables:');
      console.log('   - For SAC token: KALE_ASSET_CODE and KALE_ASSET_ISSUER');
      console.log('   - For custom token: KALE_TOKEN_CONTRACT');
      console.log('\n   Example:');
      console.log('   KALE_ASSET_ISSUER=GXXXXX... bun run fetch_c_balances.ts');
    }
    
  } catch (error) {
    console.error('Fatal error:', error);
  }
}

// Run the script
main(); 