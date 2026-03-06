import {
  Keypair,
  Horizon,
  TransactionBuilder,
  Operation,
  Asset,
  Networks,
} from "@stellar/stellar-sdk";

const args = process.argv.slice(2);
if (args.length < 2) {
  console.error("Usage: bun controlled_burn.ts <issuer_address> <assets...>");
  console.error("Example: bun controlled_burn.ts GBOI3... CARROT PEPPER");
  process.exit(1);
}

const ISSUER_ADDRESS = args[0];
const traderSecret = process.env.CONTROLLED_BURN_TRADER_SECRET?.trim();
const ASSET_NAMES = args.slice(1);

if (!traderSecret) {
  console.error("Set CONTROLLED_BURN_TRADER_SECRET in a gitignored .env file before running this script.");
  process.exit(1);
}

const TRADER_SECRET = traderSecret;
const server = new Horizon.Server("https://horizon-futurenet.stellar.org");
const networkPassphrase = Networks.FUTURENET;

async function createSellOffers() {
  try {
    // Load trader keypair
    const traderKeypair = Keypair.fromSecret(TRADER_SECRET);
    const traderAddress = traderKeypair.publicKey();
    
    console.log(`\n=== Creating sell offers for trading account ${traderAddress} ===`);
    
    // Load trader account
    const account = await server.loadAccount(traderAddress);
    
    // Create KALE asset
    const kaleAsset = new Asset("KALE", ISSUER_ADDRESS);
    
    // Build transaction with all sell offers
    let transactionBuilder = new TransactionBuilder(account, {
      fee: "100000", // 0.01 XLM per operation
      networkPassphrase,
    });
    
    // Add sell offer operations for each vegetable (selling KALE, buying vegetables)
    for (const assetName of ASSET_NAMES) {
      if (assetName === "KALE") continue;
      
      console.log(`\nAdding sell offer: KALE → ${assetName} (1:1)`);
      
      const buyingAsset = new Asset(assetName, ISSUER_ADDRESS);
      
      // Create sell offer operation
      // Amount: 100M KALE per vegetable (with 7 decimals = 1000000000000000 / 10^7 = 100000000)
      // Price: 1.0 (1:1 ratio)
      const operation = Operation.createPassiveSellOffer({
        selling: kaleAsset,
        buying: buyingAsset,
        amount: "100000000", // 100M units of KALE
        price: "1", // 1:1 ratio
        // offerId: "0", // 0 = create new offer
      });
      
      transactionBuilder = transactionBuilder.addOperation(operation);
    }
    
    // Set timeout and build transaction
    const transaction = transactionBuilder.setTimeout(300).build();
    
    // Sign transaction
    transaction.sign(traderKeypair);
    
    console.log("\n=== Submitting transaction ===");
    
    // Submit transaction
    const result = await server.submitTransaction(transaction);
    
    console.log("\n✅ Transaction successful!");
    console.log(`Transaction hash: ${result.hash}`);
    console.log(`\nView on explorer: https://futurenet.stellarchain.io/transactions/${result.hash}`);
    
    // Log which offers were created
    console.log("\n=== Created Sell Offers ===");
    for (const assetName of ASSET_NAMES) {
      if (assetName === "KALE") continue;
      console.log(`KALE → ${assetName}: 100M at 1:1 ratio`);
    }
    console.log("\nCheck the transaction on the explorer to see the offer IDs.");
    
  } catch (error: any) {
    console.error("\n❌ Error creating sell offers:", error);
    if (error.response && error.response.data) {
      console.error("Server response:", error.response.data);
    }
    process.exit(1);
  }
}

// Run the function
createSellOffers();
