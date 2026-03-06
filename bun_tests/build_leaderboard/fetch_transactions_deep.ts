import { Horizon, xdr, StrKey } from '@stellar/stellar-sdk';
import { writeFileSync } from 'fs';

const KALE_CONTRACTS = [
    "CDC4HALCSUNEB2BLCLDHMUDAKOIE2CT4HSPE2D42YIIOXKVE3DA2GMXZ",
    "CDVAFNGN7SBQBWJJWHZ4FESLK6PDLRC6CBLDLDQGZ3KX6XCEYW3XHDJ7",
    "CDYO2WBB4B3TYO3TW3AW4Z24NQIENHIPZ5U6ESMW3IXVP2W7VOAOEVMQ",

    "CBLFUOOOPPJICOIGNSOKXJUMTZ6IFQV7SLZXOSNBPAU7AE4CHGCKCXUA",
    "CBYTMSEIA2S7S32JS2GHJA6ALCCIYVA52S5VZIIGGUBQ2C2KRJMHVDCF",
    "CDF45P4VK2YYWS2NXZMIOFLVPKNZYLH5KWZXLUFNELWXXYBN2NHQPWJC",

    "CBTAD7IF6PQ5TMBW47VM5OXNEIHVRZKQJ7ROPGO4IAKFGUD255UCTEXB",
    "CDAG7OZW66ODS2PZOVM54HWUCPSBAVK6VSTTURX2WWLWAVW6N3ISSYM3",
    "CD4E6GPDUF4LSVQ3A6WFQWKENDJP26MGPPO26HYE65TCBI4K2SDZDAAU",

    "CBNRNN4ROJKCZNA43CO5IVNFQZEW3ZLLUXUADFJIM4LUQFWRRO36FCOW",
    "CDDK5NTU4ZXT44A4ER25HTWSPKYUYVAPIITFDO5LZB7KJXKNMIP3NUN2",
    "CBG7VQESI7L7GM7WAKDTUPAHOQW4I7TTW6XIEOM4NXLKKEW4NPJFPDNG",

    "CAEPTRELFI6A26IZNFXHEZGRGQBYO6XEIDJTMTYXOZBFAZUHLHI3JTMQ",
    "CBOY65CSBDJTXV4R4HXSDYUFD2AB4KPS2WLPLNB7BKLMD4SPYU5LT4ZQ",
    "CC63IHH5S3APBNMHCPLIP7OQJMGICT5BUPTG2JC7JPV4PKL7WITA3VJM",

    "CAWUA6LBZWK43R5WJEIMHHID5TNHKDC2Q62ETWMXHVITUDQP7IV7ZUXN",
    "CCMEQJHYZX2O7KBVRL7MZEVT56T4VDC3ZFIZ6JTEIDMGWMRNZJJZ3ZDA",
    "CBHXOBEJEQPIHFOXWIWAPFTJ32QEYUNTNWBISS3KYSMB5XMINPSCWCQN",

    "CBHPM2ZVZIXTZQQ3F2CPRECFOWMM5QQSQLVBDVFCOQ43XYWGUS2XOUVQ",
    "CCGCFEUX6E7CQJQR4Q74NRK2EUTE6KLJROEVFTLZSOEGOQG3B4PYROU6",
    "CAU4YQ3KB7JUOX4ASZARPAPWNZ3QERKJGONNGUIKU6U3HVZSAG4QOMZX",

    "CAO23UDENOORPLAVLEX7ME6YYM43EUUTRPWDBLBX267KY6AMYGK7AAPC",
    "CBNCFROZILPWNVSF2OLBS6TI3DWQ3IM4C53V4GHCIO57MVWUQP6EXYPT",
    "CAUJVN4CSQSGU5N2SPASUILUKRBRPQP34ZNBHCW6HAC4RFRTNDAZIVRO",
];

// Connect to Stellar Futurenet
const server = new Horizon.Server('https://horizon-futurenet.stellar.org');

interface TransactionRecord {
    hash: string;
    contract?: string;
    created_at: string;
    operation_count: number;
    successful: boolean;
    ledger: number;
}

interface AddressRecord {
    address: string;
    type: 'contract' | 'account';
    transaction_hash: string;
    invoked_contract: string;
    created_at: string;
}

// Configuration
const CONFIG = {
    CUTOFF_DATE: new Date('2025-07-23T00:00:00Z'), // July 23rd, 2025 00:00:00 UTC
    BATCH_SIZE: 200,          // Max allowed by Horizon
    RATE_LIMIT_DELAY: 500,    // ms between batches
    PROGRESS_INTERVAL: 20,    // Show progress every N batches
};

// Add delay to avoid rate limiting
const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

// stellar-base's XDR typings expose contract IDs as opaque arrays, even though
// the runtime value is the 32-byte contract hash expected by StrKey.
function contractIdToAddress(contractId: xdr.Hash): string {
    return StrKey.encodeContract(contractId as unknown as Buffer);
}

// Check if a transaction date is before our cutoff
function isBeforeCutoff(transactionDate: string): boolean {
    return new Date(transactionDate) < CONFIG.CUTOFF_DATE;
}

// Format date for display
function formatDate(date: Date): string {
    return date.toISOString().replace('T', ' ').replace('.000Z', ' UTC');
}

// Extract addresses from ScVal arguments recursively
function extractAddressesFromScVal(
    scVal: xdr.ScVal,
    addresses: AddressRecord[],
    txHash: string,
    invokedContract: string,
    createdAt: string
): void {
    try {
        switch (scVal.switch()) {
            case xdr.ScValType.scvAddress():
                const address = scVal.address();
                switch (address.switch()) {
                    case xdr.ScAddressType.scAddressTypeContract():
                        const contractId = address.contractId();
                        const contractAddress = contractIdToAddress(contractId);
                        addresses.push({
                            address: contractAddress,
                            type: 'contract',
                            transaction_hash: txHash,
                            invoked_contract: invokedContract,
                            created_at: createdAt
                        });
                        break;
                    case xdr.ScAddressType.scAddressTypeAccount():
                        const accountId = address.accountId().ed25519();
                        const accountAddress = StrKey.encodeEd25519PublicKey(accountId);
                        addresses.push({
                            address: accountAddress,
                            type: 'account',
                            transaction_hash: txHash,
                            invoked_contract: invokedContract,
                            created_at: createdAt
                        });
                        break;
                }
                break;
            case xdr.ScValType.scvVec():
                // Recursively check vector elements
                const vec = scVal.vec();
                if (vec) {
                    vec.forEach(elem => extractAddressesFromScVal(elem, addresses, txHash, invokedContract, createdAt));
                }
                break;
            case xdr.ScValType.scvMap():
                // Recursively check map entries
                const map = scVal.map();
                if (map) {
                    map.forEach(entry => {
                        extractAddressesFromScVal(entry.key(), addresses, txHash, invokedContract, createdAt);
                        extractAddressesFromScVal(entry.val(), addresses, txHash, invokedContract, createdAt);
                    });
                }
                break;
            // Add other container types if needed
        }
    } catch (error) {
        // Silently ignore extraction errors
    }
}

// Extract the transaction source account
function extractTransactionSource(envelope: xdr.TransactionEnvelope): string | null {
    try {
        let sourceAccount: xdr.MuxedAccount | null = null;
        
        switch (envelope.switch()) {
            case xdr.EnvelopeType.envelopeTypeTxV0():
                // V0 transactions use raw Ed25519 public key
                const v0Source = envelope.v0().tx().sourceAccountEd25519();
                if (v0Source) {
                    return StrKey.encodeEd25519PublicKey(v0Source);
                }
                break;
            case xdr.EnvelopeType.envelopeTypeTx():
                sourceAccount = envelope.v1().tx().sourceAccount();
                break;
            case xdr.EnvelopeType.envelopeTypeTxFeeBump():
                // For fee bump transactions, we want both the fee source and inner tx source
                const feeBumpTx = envelope.feeBump().tx();
                const feeSource = feeBumpTx.feeSource();
                
                // Get fee source account
                if (feeSource) {
                    const feeSourceStr = muxedAccountToAddress(feeSource);
                    // Also get the inner transaction source
                    const innerTx = feeBumpTx.innerTx();
                    if (innerTx.switch() === xdr.EnvelopeType.envelopeTypeTx()) {
                        sourceAccount = innerTx.v1().tx().sourceAccount();
                    }
                }
                break;
        }
        
        if (sourceAccount) {
            return muxedAccountToAddress(sourceAccount);
        }
    } catch (error) {
        // Silently ignore errors
    }
    
    return null;
}

// Convert MuxedAccount to string address
function muxedAccountToAddress(muxedAccount: xdr.MuxedAccount): string | null {
    try {
        switch (muxedAccount.switch()) {
            case xdr.CryptoKeyType.keyTypeEd25519():
                return StrKey.encodeEd25519PublicKey(muxedAccount.ed25519());
            case xdr.CryptoKeyType.keyTypeMuxedEd25519():
                // For muxed accounts, extract the underlying Ed25519 key
                return StrKey.encodeEd25519PublicKey(muxedAccount.med25519().ed25519());
            default:
                return null;
        }
    } catch (error) {
        return null;
    }
}

// Extract operation source accounts
function extractOperationSources(operations: xdr.Operation[]): string[] {
    const sources: string[] = [];
    
    for (const op of operations) {
        const opSource = op.sourceAccount();
        if (opSource) {
            const address = muxedAccountToAddress(opSource);
            if (address) {
                sources.push(address);
            }
        }
    }
    
    return sources;
}

// Extract addresses from Soroban authorization entries
function extractSorobanAuthAddresses(auth: xdr.SorobanAuthorizationEntry[]): string[] {
    const addresses: string[] = [];
    
    for (const authEntry of auth) {
        try {
            const credentials = authEntry.credentials();
            
            // Check if it's address-based credentials
            if (credentials.switch() === xdr.SorobanCredentialsType.sorobanCredentialsAddress()) {
                const addressCredentials = credentials.address();
                const address = addressCredentials.address();
                
                // Convert ScAddress to string
                switch (address.switch()) {
                    case xdr.ScAddressType.scAddressTypeAccount():
                        const accountId = address.accountId().ed25519();
                        addresses.push(StrKey.encodeEd25519PublicKey(accountId));
                        break;
                    case xdr.ScAddressType.scAddressTypeContract():
                        const contractId = address.contractId();
                        addresses.push(contractIdToAddress(contractId));
                        break;
                }
            }
        } catch (error) {
            // Silently ignore errors
        }
    }
    
    return addresses;
}

// Decode transaction to find invoke_host_function operations
function findContractInvocations(
    txEnvelopeXdr: string,
    txHash: string,
    createdAt: string,
    addressRecords: AddressRecord[]
): string[] {
    const contractsInvoked: string[] = [];

    try {
        // Decode the transaction envelope
        const envelope = xdr.TransactionEnvelope.fromXDR(txEnvelopeXdr, 'base64');
        
        // Extract transaction source account
        const txSource = extractTransactionSource(envelope);
        if (txSource) {
            addressRecords.push({
                address: txSource,
                type: 'account',
                transaction_hash: txHash,
                invoked_contract: 'TRANSACTION_SOURCE',
                created_at: createdAt
            });
        }

        // Get the operations from the transaction
        let operations: xdr.Operation[] = [];

        switch (envelope.switch()) {
            case xdr.EnvelopeType.envelopeTypeTxV0():
                operations = envelope.v0().tx().operations();
                break;
            case xdr.EnvelopeType.envelopeTypeTx():
                operations = envelope.v1().tx().operations();
                break;
            case xdr.EnvelopeType.envelopeTypeTxFeeBump():
                const innerTx = envelope.feeBump().tx().innerTx();
                if (innerTx.switch() === xdr.EnvelopeType.envelopeTypeTx()) {
                    operations = innerTx.v1().tx().operations();
                }
                // Also extract fee source for fee bump transactions
                const feeSource = envelope.feeBump().tx().feeSource();
                if (feeSource) {
                    const feeSourceAddr = muxedAccountToAddress(feeSource);
                    if (feeSourceAddr) {
                        addressRecords.push({
                            address: feeSourceAddr,
                            type: 'account',
                            transaction_hash: txHash,
                            invoked_contract: 'FEE_BUMP_SOURCE',
                            created_at: createdAt
                        });
                    }
                }
                break;
        }
        
        // Extract operation source accounts
        const opSources = extractOperationSources(operations);
        for (const opSource of opSources) {
            addressRecords.push({
                address: opSource,
                type: 'account',
                transaction_hash: txHash,
                invoked_contract: 'OPERATION_SOURCE',
                created_at: createdAt
            });
        }

        // Check each operation
        for (const op of operations) {
            if (op.body().switch() === xdr.OperationType.invokeHostFunction()) {
                const invokeOp = op.body().value() as xdr.InvokeHostFunctionOp;
                const hostFunction = invokeOp.hostFunction();
                
                // Extract Soroban auth addresses
                const authEntries = invokeOp.auth();
                if (authEntries && authEntries.length > 0) {
                    const authAddresses = extractSorobanAuthAddresses(authEntries);
                    for (const authAddr of authAddresses) {
                        const isContract = authAddr.startsWith('C');
                        addressRecords.push({
                            address: authAddr,
                            type: isContract ? 'contract' : 'account',
                            transaction_hash: txHash,
                            invoked_contract: 'SOROBAN_AUTH',
                            created_at: createdAt
                        });
                    }
                }

                // Check if it's invoking a contract function
                if (hostFunction.switch() === xdr.HostFunctionType.hostFunctionTypeInvokeContract()) {
                    const invokeArgs = hostFunction.invokeContract();

                    // InvokeContractArgs has contractAddress and functionName fields
                    const contractAddress = invokeArgs.contractAddress();

                    if (contractAddress.switch() === xdr.ScAddressType.scAddressTypeContract()) {
                        // Convert the contract ID to a C... address
                        const contractId = contractAddress.contractId();
                        const contractAddressStr = contractIdToAddress(contractId);
                        contractsInvoked.push(contractAddressStr);

                        // Only extract addresses if this is one of our KALE contracts
                        if (KALE_CONTRACTS.includes(contractAddressStr as any)) {
                            // Extract addresses from the invoke arguments
                            invokeArgs.args().forEach(arg => {
                                extractAddressesFromScVal(arg, addressRecords, txHash, contractAddressStr, createdAt);
                            });
                        }
                    }
                }
            }
        }
    } catch (error) {
        // Silently ignore decoding errors
    }

    return contractsInvoked;
}

async function searchTransactionsForContracts(startCursor?: string): Promise<{
    transactions: TransactionRecord[],
    addresses: AddressRecord[]
}> {
    const transactions: TransactionRecord[] = [];
    const addresses: AddressRecord[] = [];
    const uniqueHashes = new Set<string>();

    console.log('Date-based search for KALE contract invocations on Futurenet...\n');
    console.log('Target contracts:');
    KALE_CONTRACTS.forEach(contract => console.log(`  - ${contract}`));
    console.log(`\nSearching from now back to: ${formatDate(CONFIG.CUTOFF_DATE)}\n`);

    let hasMore = true;
    let cursor = startCursor;
    let batchCount = 0;
    let totalScanned = 0;
    let matchesFound = 0;
    let reachedCutoff = false;

    // Set up a target set for faster lookups
    const targetContracts = new Set(KALE_CONTRACTS);

    const startTime = Date.now();

    while (hasMore && !reachedCutoff) {
        try {
            batchCount++;

            // Query transactions
            let query = server.transactions()
                .order('desc')
                .limit(CONFIG.BATCH_SIZE);

            if (cursor) {
                query = query.cursor(cursor);
            }

            const response = await query.call();
            const records = response.records;

            if (records.length === 0) {
                hasMore = false;
                break;
            }

            // Process each transaction
            for (const tx of records) {
                totalScanned++;

                // Check if we've reached our cutoff date
                if (isBeforeCutoff(tx.created_at)) {
                    console.log(`\n📅 Reached cutoff date: ${tx.created_at}`);
                    reachedCutoff = true;
                    break;
                }

                // Skip if we've already processed this transaction
                if (uniqueHashes.has(tx.hash)) {
                    continue;
                }

                // Skip failed transactions
                if (!tx.successful) {
                    continue;
                }

                // Decode the transaction to find contract invocations and extract addresses
                const contractsInvoked = findContractInvocations(
                    tx.envelope_xdr,
                    tx.hash,
                    tx.created_at,
                    addresses
                );

                // Check if any of the invoked contracts match our targets
                const matchingContracts = contractsInvoked.filter(c => targetContracts.has(c as any));

                if (matchingContracts.length > 0) {
                    uniqueHashes.add(tx.hash);
                    matchesFound++;

                    // Add a record for each matching contract
                    for (const contract of matchingContracts) {
                        transactions.push({
                            hash: tx.hash,
                            contract: contract,
                            created_at: tx.created_at,
                            operation_count: tx.operation_count,
                            successful: tx.successful,
                            ledger: typeof tx.ledger_attr === 'string' ? parseInt(tx.ledger_attr) : tx.ledger_attr
                        });

                        console.log(`✓ Found match #${matchesFound}: ${tx.created_at.substring(0, 19)}Z`);
                        console.log(`  TX: ${tx.hash.substring(0, 12)}... (Ledger ${tx.ledger_attr})`);
                        console.log(`  Contract: ${contract}`);
                    }
                }
            }

            // Update cursor for next page if we haven't reached cutoff
            if (!reachedCutoff && response.records.length > 0) {
                cursor = response.records[response.records.length - 1].paging_token;
            }

            // Progress update
            if (batchCount % CONFIG.PROGRESS_INTERVAL === 0) {
                const elapsed = ((Date.now() - startTime) / 1000).toFixed(1);
                const rate = (totalScanned / parseFloat(elapsed)).toFixed(0);
                const lastDate = records.length > 0 ? records[records.length - 1].created_at.substring(0, 19) + 'Z' : 'N/A';
                console.log(`Progress: Scanned ${totalScanned.toLocaleString()} TXs, found ${matchesFound} matches (${rate} TXs/sec)`);
                console.log(`  Current date: ${lastDate}`);
                console.log(`  Addresses found: ${addresses.length} (C: ${addresses.filter(a => a.type === 'contract').length}, G: ${addresses.filter(a => a.type === 'account').length})`);
            }

            // Check if we have a next page
            hasMore = response.records.length === CONFIG.BATCH_SIZE;

            // Rate limiting
            await delay(CONFIG.RATE_LIMIT_DELAY);

        } catch (error: any) {
            if (error?.response?.status === 429) {
                console.log('Rate limited, waiting 10 seconds...');
                await delay(10000);
                continue;
            }
            console.error('Error fetching transactions:', error.message || error);
            console.log('Retrying in 5 seconds...');
            await delay(5000);
        }
    }

    const elapsed = ((Date.now() - startTime) / 1000).toFixed(1);
    const reason = reachedCutoff ? 'reached cutoff date' : 'no more transactions';
    console.log(`\nSearch complete (${reason}). Scanned ${totalScanned.toLocaleString()} transactions in ${elapsed} seconds.`);

    return { transactions, addresses };
}

async function main() {
    try {
        // Optional: Start from a specific cursor if you want to resume a search
        const startCursor = process.argv[2]; // Pass cursor as command line argument

        console.log(`Starting search from ${formatDate(new Date())} back to ${formatDate(CONFIG.CUTOFF_DATE)}\n`);

        // Search for transactions
        const { transactions, addresses } = await searchTransactionsForContracts(startCursor);

        if (transactions.length === 0) {
            console.log('\nNo transactions found that invoked the specified KALE contracts.');
            console.log('Suggestions:');
            console.log('1. The contracts may not have been invoked during this time period');
            console.log('2. Try a different date range or network');
            console.log('3. Verify the contract addresses are correct');
            return;
        }

        console.log(`\n✅ Found ${transactions.length} transaction(s) invoking KALE contracts!`);
        console.log(`✅ Found ${addresses.length} addresses in invoke arguments!`);

        // Sort by ledger (chronological order)
        transactions.sort((a, b) => a.ledger - b.ledger);

        // Save simple CSV (just unique hashes)
        const uniqueHashes = [...new Set(transactions.map(t => t.hash))];
        const simpleCSV = uniqueHashes.join('\n');
        writeFileSync('kale_transactions.csv', simpleCSV);
        console.log(`\nSaved ${uniqueHashes.length} unique transaction hashes to: kale_transactions.csv`);

        // Save detailed CSV
        const detailedCSV = [
            'transaction_hash,contract_id,created_at,ledger,operation_count,successful',
            ...transactions.map(t =>
                `${t.hash},${t.contract},${t.created_at},${t.ledger},${t.operation_count},${t.successful}`
            )
        ].join('\n');
        writeFileSync('kale_transactions_detailed.csv', detailedCSV);
        console.log('Saved detailed data to: kale_transactions_detailed.csv');

        // Save C addresses CSV
        const cAddresses = addresses.filter(a => a.type === 'contract');
        const uniqueCAddresses = [...new Set(cAddresses.map(a => a.address))];
        const cAddressesSimpleCSV = uniqueCAddresses.join('\n');
        writeFileSync('c_addresses.csv', cAddressesSimpleCSV);
        console.log(`Saved ${uniqueCAddresses.length} unique C addresses to: c_addresses.csv`);

        // Save detailed C addresses CSV
        const cAddressesDetailedCSV = [
            'address,transaction_hash,invoked_contract,created_at',
            ...cAddresses.map(a =>
                `${a.address},${a.transaction_hash},${a.invoked_contract},${a.created_at}`
            )
        ].join('\n');
        writeFileSync('c_addresses_detailed.csv', cAddressesDetailedCSV);
        console.log('Saved detailed C address data to: c_addresses_detailed.csv');

        // Save G addresses CSV
        const gAddresses = addresses.filter(a => a.type === 'account');
        const uniqueGAddresses = [...new Set(gAddresses.map(a => a.address))];
        const gAddressesSimpleCSV = uniqueGAddresses.join('\n');
        writeFileSync('g_addresses.csv', gAddressesSimpleCSV);
        console.log(`Saved ${uniqueGAddresses.length} unique G addresses to: g_addresses.csv`);

        // Save detailed G addresses CSV
        const gAddressesDetailedCSV = [
            'address,transaction_hash,invoked_contract,created_at',
            ...gAddresses.map(a =>
                `${a.address},${a.transaction_hash},${a.invoked_contract},${a.created_at}`
            )
        ].join('\n');
        writeFileSync('g_addresses_detailed.csv', gAddressesDetailedCSV);
        console.log('Saved detailed G address data to: g_addresses_detailed.csv');

        // Print summary
        const contractCounts = transactions.reduce((acc, t) => {
            acc[t.contract!] = (acc[t.contract!] || 0) + 1;
            return acc;
        }, {} as Record<string, number>);

        console.log('\nSummary by contract:');
        Object.entries(contractCounts)
            .sort(([, a], [, b]) => b - a)
            .forEach(([contract, count]) => {
                console.log(`  ${contract}: ${count} invocations`);
            });

        // Show time range
        if (transactions.length > 0) {
            const firstTx = transactions[0];
            const lastTx = transactions[transactions.length - 1];
            console.log('\nActual time range found:');
            console.log(`  First: ${firstTx.created_at} (Ledger ${firstTx.ledger})`);
            console.log(`  Last:  ${lastTx.created_at} (Ledger ${lastTx.ledger})`);
        }

        // Show search parameters
        console.log('\nSearch parameters:');
        console.log(`  Cutoff date: ${formatDate(CONFIG.CUTOFF_DATE)}`);
        console.log(`  Total unique transactions: ${uniqueHashes.length}`);
        console.log(`  Total contract invocations: ${transactions.length}`);
        console.log(`  Total addresses extracted: ${addresses.length}`);
        console.log(`    - Contract addresses (C): ${uniqueCAddresses.length} unique`);
        console.log(`    - Account addresses (G): ${uniqueGAddresses.length} unique`);

    } catch (error) {
        console.error('Fatal error:', error);
    }
}

// Run the script
main(); 
