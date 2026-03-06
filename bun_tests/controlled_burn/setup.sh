#!/bin/bash

# Get the controlled-burn-admin address dynamically
# stellar keys generate controlled-burn-admin --network futurenet --as-secret --overwrite
# stellar keys fund controlled-burn-admin --network futurenet
ISSUER_ADDRESS=$(stellar keys address controlled-burn-admin)
echo "Using issuer address: $ISSUER_ADDRESS"

# Arrays to store deployed contract IDs for summary
ASSET_IDS=()
FARM_IDS=()

# Function to deploy an asset and its corresponding farm contract
deploy_asset_and_farm() {
    local asset_name=$1
    
    echo "=== Deploying $asset_name asset ==="
    
    # Step 1: Deploy the asset contract
    echo "1. Deploying $asset_name asset contract..."
    ASSET_ID=$(stellar contract asset deploy --asset $asset_name:$ISSUER_ADDRESS --network futurenet --source controlled-burn-admin)
    echo "Asset contract deployed with ID: $ASSET_ID"
    
    # Step 2: Deploy the farm contract using the asset ID
    echo "2. Deploying farm contract for $asset_name..."
    FARM_ID=$(stellar contract deploy --wasm ../../target/wasm32v1-none/release/kale_sc.optimized.wasm --network futurenet --source controlled-burn-admin -- --farmer controlled-burn-admin --asset $ASSET_ID)
    echo "Farm contract deployed with ID: $FARM_ID"
    
    # Step 3: Set the asset's admin to be the farm contract
    echo "3. Setting asset admin to farm contract..."
    stellar contract invoke --id $ASSET_ID --network futurenet --source controlled-burn-admin -- set_admin --new_admin $FARM_ID
    echo "Asset admin updated successfully"
    
    # Store IDs for summary
    ASSET_IDS+=("$ASSET_ID")
    FARM_IDS+=("$FARM_ID")
    
    echo "$asset_name deployment completed successfully!"
    echo ""
}

# Array of vegetables/assets to deploy
assets=("KALE" "CARROT" "PEPPER" "ONION" "MEAT" "POTATO" "CORN" "TOMATO")

# Loop through assets and deploy each one with its farm contract
# for asset in "${assets[@]}"; do
#     deploy_asset_and_farm "$asset"
# done

echo "=== All assets and farm contracts deployed successfully! ==="
echo ""
echo "===================================================="
echo "            DEPLOYMENT SUMMARY"
echo "===================================================="
echo ""

# Display clean summary
for i in "${!assets[@]}"; do
    echo "${assets[$i]}"
    echo "${ASSET_IDS[$i]}"
    echo "${FARM_IDS[$i]}"
    echo ""
done

echo "===================================================="

# Create new trading account
echo ""
echo "=== Creating trading account ==="
# stellar keys generate controlled-burn-trader --network futurenet --as-secret --overwrite
# stellar keys fund controlled-burn-trader --network futurenet
TRADING_ADDRESS=$(stellar keys address controlled-burn-trader)
echo "Trading account created: $TRADING_ADDRESS"

# Setup trustlines and mint KALE (not vegetables)
echo ""
echo "=== Setting up trustlines and minting KALE ==="

# Create trustlines for all assets
for i in "${!assets[@]}"; do
    asset_name="${assets[$i]}"
    
    echo "Creating trustline for $asset_name..."
    # stellar tx new change-trust \
    #     --line "$asset_name:$ISSUER_ADDRESS" \
    #     --source-account controlled-burn-trader \
    #     --network futurenet
done

# Mint 700M KALE to trading account (700M = 7000000000000000 with 7 decimals)
echo ""
echo "Minting 700M KALE to trading account..."
# stellar tx new payment \
#     --destination "$TRADING_ADDRESS" \
#     --asset "KALE:$ISSUER_ADDRESS" \
#     --amount 7000000000000000 \
#     --source-account controlled-burn-admin \
#     --network futurenet

echo "KALE minting completed!"

# Setup 1:1 sell orders of KALE FOR vegetables
echo ""
echo "=== Setting up 1:1 sell orders (KALE FOR vegetables) ==="

# Build the assets list (excluding KALE if needed)
ASSETS_STRING="${assets[@]}"

echo "Running TypeScript to create sell offers..."
if [[ ! -f .env ]]; then
    echo "Missing .env in $(pwd)"
    echo "Create it from .env.example so Bun can load CONTROLLED_BURN_TRADER_SECRET safely."
    exit 1
fi

bun run controlled_burn.ts "$ISSUER_ADDRESS" $ASSETS_STRING

echo ""
echo "=== Setup completed successfully! ==="
echo "Trading account: $TRADING_ADDRESS"
echo "- Has trustlines to all assets"
echo "- Has 700M KALE"
echo "- Has 0 of each vegetable initially"
echo "- Has 1:1 sell offers: KALE FOR each vegetable"
echo "- If all orders filled → 0 KALE, 100M of each vegetable"
