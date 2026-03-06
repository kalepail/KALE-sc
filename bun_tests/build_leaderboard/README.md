# KALE Contract Transaction Fetcher and Balance Checker

This directory contains scripts to fetch and analyze transactions for KALE smart contracts on Stellar Futurenet, as well as check KALE balances for addresses.

## Quick Start

Use the interactive bash script for the complete workflow:

```bash
# Make the script executable (first time only)
chmod +x run_full_analysis.sh

# Run with interactive menu
./run_full_analysis.sh

# Run full analysis without prompts
KALE_ASSET_ISSUER=GXXXXX... ./run_full_analysis.sh --full

# Show help
./run_full_analysis.sh --help
```

The script provides:
- Interactive menu for step-by-step execution
- Automatic full workflow with `--full` flag
- Configuration validation
- Progress tracking with colored output
- Summary report generation
- File cleanup options
- Resume capability for interrupted fetches

## Overview

These scripts properly decode XDR transaction envelopes to find `invoke_host_function` operations that interact with Soroban smart contracts. The contract addresses are encoded in the XDR data, so simple string searching won't work.

## Files

### Transaction and Address Extraction
- `fetch_transactions_deep.ts` - **Enhanced deep search script** that:
  - Searches from now back to a cutoff date (currently July 23rd, 2025)
  - Extracts ALL addresses from transactions:
    - **Transaction source** - The account that signs and pays for the transaction
    - **Operation sources** - Each operation can have its own source account
    - **Fee bump sources** - For fee bump transactions
    - **Soroban auth addresses** - Addresses in Soroban authorization entries
    - **Invoke arguments** - Addresses passed as arguments to smart contracts
  - Recursively searches through nested ScVal structures (vectors, maps, etc.)
  - Generates separate CSV files for contract and account addresses

### Balance Checking Scripts
- `fetch_g_balances.ts` - Fetches KALE balances for G addresses (Stellar accounts)
  - Supports both traditional Stellar assets and Stellar Asset Contracts (SAC)
  - Processes addresses in batches with rate limiting
  - Automatic retry on rate limit errors
  - Generates a leaderboard CSV ordered by balance
  
- `fetch_c_balances.ts` - Fetches KALE balances for C addresses (Soroban contracts)
  - Checks balances using getSACBalance for SAC tokens
  - Identifies known KALE farming contracts
  - Automatic retry on rate limit errors
  - Generates a leaderboard CSV with contract metadata

- `test_rate_limits.ts` - Tests optimal rate limit settings
  - Tests different batch sizes and delays
  - Provides recommendations for optimal settings
  - Helps avoid rate limit errors

## Usage

### Step 1: Fetch Transactions and Extract Addresses
```bash
cd bun_tests/build_leaderboard

# Run the enhanced deep search
bun run fetch_transactions_deep.ts
```

This generates:
- `kale_transactions.csv` - Simple list of unique transaction hashes (one per line)
- `kale_transactions_detailed.csv` - Detailed CSV with headers:
  - `transaction_hash` - The transaction hash
  - `contract_id` - The KALE contract that was invoked
  - `created_at` - Timestamp of the transaction
  - `ledger` - Ledger number
  - `operation_count` - Number of operations in the transaction
  - `successful` - Whether the transaction succeeded

- `c_addresses.csv` - Simple list of unique contract addresses found (one per line)
- `c_addresses_detailed.csv` - Detailed CSV with headers:
  - `address` - The contract address (C...)
  - `transaction_hash` - Transaction where this address was found
  - `invoked_contract` - Context where found:
    - KALE contract address if found in invoke arguments
    - `SOROBAN_AUTH` if found in auth entries
    - `TRANSACTION_SOURCE`, `OPERATION_SOURCE`, or `FEE_BUMP_SOURCE` for other contexts
  - `created_at` - Timestamp of the transaction

- `g_addresses.csv` - Simple list of unique account addresses found (one per line)
- `g_addresses_detailed.csv` - Detailed CSV with headers:
  - `address` - The account address (G...)
  - `transaction_hash` - Transaction where this address was found
  - `invoked_contract` - Context where found (same as C addresses)
  - `created_at` - Timestamp of the transaction

### Step 2: Fetch Balances

**For G addresses (accounts):**
```bash
# Configure the KALE asset (if known)
export KALE_ASSET_CODE=KALE
export KALE_ASSET_ISSUER=GXXXXX...  # The issuer account

# Or for SAC tokens
export KALE_TOKEN_CONTRACT=CXXXXX...  # The token contract

# Run the script
bun run fetch_g_balances.ts
```

**For C addresses (contracts):**
```bash
# Same configuration as above
bun run fetch_c_balances.ts
```

## Output Files

### Balance Leaderboards
- `g_addresses_leaderboard.csv` - Account balances ordered by amount
  - Columns: `address,balance`
  
- `c_addresses_leaderboard.csv` - Contract balances ordered by amount
  - Columns: `address,balance,is_kale_contract`
  - Includes a flag for known KALE farming contracts

## Configuration

The balance scripts support multiple methods to find KALE balances:

1. **Traditional Stellar Asset**:
   ```bash
   export KALE_ASSET_CODE=KALE
   export KALE_ASSET_ISSUER=GXXXXX...
   ```

2. **Stellar Asset Contract (SAC)**:
   ```bash
   export KALE_TOKEN_CONTRACT=CXXXXX...
   ```

The scripts will try multiple approaches to find balances and warn if configuration is missing.

## KALE Farming Contracts

The following contracts are identified as KALE farming/homestead contracts:
- CDVAFNGN7SBQBWJJWHZ4FESLK6PDLRC6CBLDLDQGZ3KX6XCEYW3XHDJ7
- CBYTMSEIA2S7S32JS2GHJA6ALCCIYVA52S5VZIIGGUBQ2C2KRJMHVDCF
- CDAG7OZW66ODS2PZOVM54HWUCPSBAVK6VSTTURX2WWLWAVW6N3ISSYM3
- CDDK5NTU4ZXT44A4ER25HTWSPKYUYVAPIITFDO5LZB7KJXKNMIP3NUN2
- CBOY65CSBDJTXV4R4HXSDYUFD2AB4KPS2WLPLNB7BKLMD4SPYU5LT4ZQ
- CCMEQJHYZX2O7KBVRL7MZEVT56T4VDC3ZFIZ6JTEIDMGWMRNZJJZ3ZDA
- CCGCFEUX6E7CQJQR4Q74NRK2EUTE6KLJROEVFTLZSOEGOQG3B4PYROU6
- CBNCFROZILPWNVSF2OLBS6TI3DWQ3IM4C53V4GHCIO57MVWUQP6EXYPT

## Technical Details

### Address Extraction
- Uses proper XDR decoding with `@stellar/stellar-sdk`
- Extracts addresses from multiple sources:
  - Transaction envelope source account (including V0, V1, and fee bump transactions)
  - Individual operation source accounts
  - Soroban authorization credentials
  - Smart contract invoke arguments (recursively through all data structures)
- Properly formats addresses using:
  - `StrKey.encodeContract()` for C addresses
  - `StrKey.encodeEd25519PublicKey()` for G addresses
- Handles muxed accounts by extracting the underlying Ed25519 key

### Balance Checking
- **G addresses**: Uses Horizon API for traditional assets or Soroban RPC for SAC tokens
- **C addresses**: Uses Soroban RPC `getSACBalance` method
- Handles rate limiting with configurable delays
- Processes in batches to optimize performance
- **Rate Limiting Configuration**:
  - G addresses: Batch size 10, 1 second delay between batches
  - C addresses: Batch size 5, 2 second delay between batches
  - Automatic retry with exponential backoff on rate limit errors
  - Up to 3 retries per address

## Resume from Cursor

The transaction fetch script can be resumed from where it left off:

```bash
bun run fetch_transactions_deep.ts [CURSOR_VALUE]
```

## Examples

### Full Workflow
```bash
# 1. Fetch all transactions and extract addresses
bun run fetch_transactions_deep.ts

# 2. Check balances for accounts (G addresses)
KALE_ASSET_ISSUER=GXXXXX... bun run fetch_g_balances.ts

# 3. Check balances for contracts (C addresses)
KALE_ASSET_ISSUER=GXXXXX... bun run fetch_c_balances.ts

# 4. View the leaderboards
cat g_addresses_leaderboard.csv | head -10
cat c_addresses_leaderboard.csv | head -10
``` 

## Troubleshooting

### Rate Limit Errors
If you encounter rate limit errors (429), try:

1. **Test optimal settings**:
   ```bash
   bun run test_rate_limits.ts
   ```

2. **Adjust batch sizes** in the scripts:
   - Reduce `BATCH_SIZE` in the configuration
   - Increase `RATE_LIMIT_DELAY` between batches

3. **Use environment variables** to override defaults:
   ```bash
   BATCH_SIZE=5 RATE_LIMIT_DELAY=2000 bun run fetch_g_balances.ts
   ``` 