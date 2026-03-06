import { Horizon, Asset, Networks } from '@stellar/stellar-sdk';
import { Server as SorobanServer } from '@stellar/stellar-sdk/rpc';
import { readFileSync } from 'fs';

// Test configuration
const TEST_CONFIG = {
  SAMPLE_SIZE: 10,            // Test with just 10 addresses
  BATCH_SIZES: [1, 5, 10, 20],    // Different batch sizes to test
  DELAYS: [100, 500, 1000, 2000], // Different delays to test (ms)
};

const horizonServer = new Horizon.Server('https://horizon-futurenet.stellar.org');
const sorobanServer = new SorobanServer('https://rpc-futurenet.stellar.org');

// Add delay
const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

async function testGAddressRate(batchSize: number, delayMs: number, addresses: string[]) {
  console.log(`\n📊 Testing G addresses - Batch: ${batchSize}, Delay: ${delayMs}ms`);
  
  let success = 0;
  let rateLimited = 0;
  let errors = 0;
  const startTime = Date.now();
  
  for (let i = 0; i < addresses.length; i += batchSize) {
    const batch = addresses.slice(i, Math.min(i + batchSize, addresses.length));
    
    const results = await Promise.all(
      batch.map(async (address) => {
        try {
          await horizonServer.loadAccount(address);
          success++;
          return 'success';
        } catch (error: any) {
          if (error?.response?.status === 429) {
            rateLimited++;
            return 'rate-limited';
          } else if (error?.response?.status === 404) {
            success++; // 404 is expected for non-existent accounts
            return 'not-found';
          } else {
            errors++;
            return 'error';
          }
        }
      })
    );
    
    if (i + batchSize < addresses.length) {
      await delay(delayMs);
    }
  }
  
  const elapsed = ((Date.now() - startTime) / 1000).toFixed(2);
  const rate = (addresses.length / parseFloat(elapsed)).toFixed(2);
  
  console.log(`  ✅ Success: ${success}`);
  console.log(`  ⚠️  Rate limited: ${rateLimited}`);
  console.log(`  ❌ Errors: ${errors}`);
  console.log(`  ⏱️  Time: ${elapsed}s (${rate} addresses/sec)`);
  
  return { batchSize, delayMs, rateLimited, elapsed, rate };
}

async function testCAddressRate(batchSize: number, delayMs: number, addresses: string[]) {
  console.log(`\n📊 Testing C addresses - Batch: ${batchSize}, Delay: ${delayMs}ms`);
  
  let success = 0;
  let rateLimited = 0;
  let errors = 0;
  const startTime = Date.now();
  
  const KALE_ASSET_ISSUER = process.env.KALE_ASSET_ISSUER || '';
  const KALE_ASSET_CODE = process.env.KALE_ASSET_CODE || 'KALE';
  
  if (!KALE_ASSET_ISSUER) {
    console.log('  ⚠️  No KALE_ASSET_ISSUER set, skipping SAC balance test');
    return null;
  }
  
  const asset = new Asset(KALE_ASSET_CODE, KALE_ASSET_ISSUER);
  
  for (let i = 0; i < addresses.length; i += batchSize) {
    const batch = addresses.slice(i, Math.min(i + batchSize, addresses.length));
    
    const results = await Promise.all(
      batch.map(async (address) => {
        try {
          await sorobanServer.getSACBalance(address, asset, Networks.FUTURENET);
          success++;
          return 'success';
        } catch (error: any) {
          if (error?.response?.status === 429 || error?.message?.includes('rate')) {
            rateLimited++;
            return 'rate-limited';
          } else {
            success++; // Other errors might be expected for contracts without balance
            return 'no-balance';
          }
        }
      })
    );
    
    if (i + batchSize < addresses.length) {
      await delay(delayMs);
    }
  }
  
  const elapsed = ((Date.now() - startTime) / 1000).toFixed(2);
  const rate = (addresses.length / parseFloat(elapsed)).toFixed(2);
  
  console.log(`  ✅ Success: ${success}`);
  console.log(`  ⚠️  Rate limited: ${rateLimited}`);
  console.log(`  ❌ Errors: ${errors}`);
  console.log(`  ⏱️  Time: ${elapsed}s (${rate} addresses/sec)`);
  
  return { batchSize, delayMs, rateLimited, elapsed, rate };
}

async function main() {
  console.log('🧪 Rate Limit Testing Tool\n');
  
  // Read sample addresses
  let gAddresses: string[] = [];
  let cAddresses: string[] = [];
  
  try {
    const gContent = readFileSync('g_addresses.csv', 'utf-8');
    gAddresses = gContent.trim().split('\n').slice(0, TEST_CONFIG.SAMPLE_SIZE);
    console.log(`✅ Loaded ${gAddresses.length} G addresses for testing`);
  } catch (error) {
    console.log('❌ Could not load g_addresses.csv');
  }
  
  try {
    const cContent = readFileSync('c_addresses.csv', 'utf-8');
    cAddresses = cContent.trim().split('\n').slice(0, TEST_CONFIG.SAMPLE_SIZE);
    console.log(`✅ Loaded ${cAddresses.length} C addresses for testing`);
  } catch (error) {
    console.log('❌ Could not load c_addresses.csv');
  }
  
  const results = {
    g: [] as any[],
    c: [] as any[]
  };
  
  // Test G addresses
  if (gAddresses.length > 0) {
    console.log('\n🔍 Testing G Address (Horizon) Rate Limits...');
    
    for (const batchSize of TEST_CONFIG.BATCH_SIZES) {
      for (const delayMs of TEST_CONFIG.DELAYS) {
        const result = await testGAddressRate(batchSize, delayMs, gAddresses);
        results.g.push(result);
        
        // If we got rate limited, skip higher batch sizes
        if (result.rateLimited > 0 && batchSize < Math.max(...TEST_CONFIG.BATCH_SIZES)) {
          console.log(`  ⚠️  Rate limited detected, skipping larger batch sizes for ${delayMs}ms delay`);
          break;
        }
        
        // Wait between tests
        await delay(5000);
      }
    }
  }
  
  // Test C addresses
  if (cAddresses.length > 0) {
    console.log('\n🔍 Testing C Address (Soroban RPC) Rate Limits...');
    
    for (const batchSize of TEST_CONFIG.BATCH_SIZES) {
      for (const delayMs of TEST_CONFIG.DELAYS) {
        const result = await testCAddressRate(batchSize, delayMs, cAddresses);
        if (result) {
          results.c.push(result);
          
          // If we got rate limited, skip higher batch sizes
          if (result.rateLimited > 0 && batchSize < Math.max(...TEST_CONFIG.BATCH_SIZES)) {
            console.log(`  ⚠️  Rate limited detected, skipping larger batch sizes for ${delayMs}ms delay`);
            break;
          }
        }
        
        // Wait between tests
        await delay(5000);
      }
    }
  }
  
  // Summary
  console.log('\n📈 Recommendations:\n');
  
  if (results.g.length > 0) {
    const bestG = results.g
      .filter(r => r.rateLimited === 0)
      .sort((a, b) => parseFloat(b.rate) - parseFloat(a.rate))[0];
    
    if (bestG) {
      console.log('For G addresses (Horizon):');
      console.log(`  Recommended batch size: ${bestG.batchSize}`);
      console.log(`  Recommended delay: ${bestG.delayMs}ms`);
      console.log(`  Expected rate: ${bestG.rate} addresses/sec`);
    } else {
      console.log('For G addresses: All configurations hit rate limits. Use smaller batches or longer delays.');
    }
  }
  
  if (results.c.length > 0) {
    const bestC = results.c
      .filter(r => r.rateLimited === 0)
      .sort((a, b) => parseFloat(b.rate) - parseFloat(a.rate))[0];
    
    if (bestC) {
      console.log('\nFor C addresses (Soroban RPC):');
      console.log(`  Recommended batch size: ${bestC.batchSize}`);
      console.log(`  Recommended delay: ${bestC.delayMs}ms`);
      console.log(`  Expected rate: ${bestC.rate} addresses/sec`);
    } else {
      console.log('\nFor C addresses: All configurations hit rate limits. Use smaller batches or longer delays.');
    }
  }
}

// Run the test
main().catch(console.error); 