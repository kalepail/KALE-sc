#!/bin/bash

# KALE Contract Analysis - Full Workflow Script
# This script runs the complete analysis: fetching transactions, extracting addresses, and checking balances

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# Function to print colored output
print_header() {
    echo -e "\n${BLUE}===================================================${NC}"
    echo -e "${GREEN}$1${NC}"
    echo -e "${BLUE}===================================================${NC}\n"
}

print_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Function to check if required tools are installed
check_requirements() {
    print_header "Checking Requirements"
    
    if ! command -v bun &> /dev/null; then
        print_error "bun is not installed. Please install bun first."
        echo "Visit: https://bun.sh"
        exit 1
    fi
    
    print_success "All requirements met!"
}

# Function to display configuration
display_config() {
    print_header "Configuration"
    
    echo "KALE Asset Configuration:"
    echo "  KALE_ASSET_CODE: ${KALE_ASSET_CODE:-Not set}"
    echo "  KALE_ASSET_ISSUER: ${KALE_ASSET_ISSUER:-Not set}"
    echo "  KALE_TOKEN_CONTRACT: ${KALE_TOKEN_CONTRACT:-Not set}"
    echo ""
    
    if [[ -z "$KALE_ASSET_ISSUER" && -z "$KALE_TOKEN_CONTRACT" ]]; then
        print_info "No KALE asset configuration found."
        echo ""
        echo "To configure, set one of the following:"
        echo "  For traditional asset:"
        echo "    export KALE_ASSET_CODE=KALE"
        echo "    export KALE_ASSET_ISSUER=GXXXXX..."
        echo ""
        echo "  For SAC token:"
        echo "    export KALE_TOKEN_CONTRACT=CXXXXX..."
        echo ""
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
}

# Function to run transaction fetcher
fetch_transactions() {
    print_header "Step 1: Fetching Transactions and Extracting Addresses"
    
    # Check if we should skip this step
    if [[ -f "kale_transactions.csv" && -f "c_addresses.csv" && -f "g_addresses.csv" ]]; then
        print_info "Transaction files already exist."
        read -p "Re-fetch transactions? This may take a while. (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_info "Skipping transaction fetch..."
            return
        fi
    fi
    
    # Run the transaction fetcher
    print_info "Running transaction fetcher (this may take several minutes)..."
    
    # if [[ -n "$1" ]]; then
    #     # Resume from cursor if provided
    #     bun run fetch_transactions_deep.ts "$1"
    # else
    #     bun run fetch_transactions_deep.ts
    # fi
    
    print_success "Transaction fetch complete!"
    
    # Display results
    if [[ -f "kale_transactions.csv" ]]; then
        TX_COUNT=$(wc -l < kale_transactions.csv | tr -d ' ')
        print_info "Found $TX_COUNT unique transactions"
    fi
    
    if [[ -f "c_addresses.csv" ]]; then
        C_COUNT=$(wc -l < c_addresses.csv | tr -d ' ')
        print_info "Found $C_COUNT unique contract addresses"
    fi
    
    if [[ -f "g_addresses.csv" ]]; then
        G_COUNT=$(wc -l < g_addresses.csv | tr -d ' ')
        print_info "Found $G_COUNT unique account addresses"
    fi
}

# Function to fetch G address balances
fetch_g_balances() {
    print_header "Step 2: Fetching G Address (Account) Balances"
    
    if [[ ! -f "g_addresses.csv" ]]; then
        print_error "g_addresses.csv not found. Please run transaction fetch first."
        return 1
    fi
    
    print_info "Checking balances for G addresses..."
    bun run fetch_g_balances.ts
    
    if [[ -f "g_addresses_leaderboard.csv" ]]; then
        print_success "G address leaderboard created!"
        
        # Show top 5
        echo -e "\nTop 5 KALE holders (G addresses):"
        head -n 6 g_addresses_leaderboard.csv | tail -n 5 | while IFS=',' read -r address balance; do
            echo "  $address: $balance KALE"
        done
    fi
}

# Function to fetch C address balances
fetch_c_balances() {
    print_header "Step 3: Fetching C Address (Contract) Balances"
    
    if [[ ! -f "c_addresses.csv" ]]; then
        print_error "c_addresses.csv not found. Please run transaction fetch first."
        return 1
    fi
    
    print_info "Checking balances for C addresses..."
    bun run fetch_c_balances.ts
    
    if [[ -f "c_addresses_leaderboard.csv" ]]; then
        print_success "C address leaderboard created!"
        
        # Show top 5
        echo -e "\nTop 5 KALE-holding contracts:"
        head -n 6 c_addresses_leaderboard.csv | tail -n 5 | while IFS=',' read -r address balance is_kale; do
            if [[ "$is_kale" == "true" ]]; then
                echo "  $address: $balance KALE (KALE Contract)"
            else
                echo "  $address: $balance KALE"
            fi
        done
    fi
}

# Function to generate summary report
generate_summary() {
    print_header "Summary Report"
    
    REPORT_FILE="kale_analysis_summary_$(date +%Y%m%d_%H%M%S).txt"
    
    {
        echo "KALE Contract Analysis Summary"
        echo "Generated: $(date)"
        echo "=================================="
        echo ""
        
        if [[ -f "kale_transactions.csv" ]]; then
            echo "Transactions:"
            echo "  Total unique transactions: $(wc -l < kale_transactions.csv | tr -d ' ')"
            echo ""
        fi
        
        if [[ -f "g_addresses_leaderboard.csv" ]]; then
            echo "G Addresses (Accounts):"
            G_TOTAL=$(tail -n +2 g_addresses_leaderboard.csv | wc -l | tr -d ' ')
            G_WITH_BALANCE=$(tail -n +2 g_addresses_leaderboard.csv | awk -F',' '$2 != "0.0000000" { count++ } END { print count }')
            echo "  Total addresses: $G_TOTAL"
            echo "  Addresses with balance: ${G_WITH_BALANCE:-0}"
            echo ""
            echo "  Top 10 holders:"
            head -n 11 g_addresses_leaderboard.csv | tail -n 10 | while IFS=',' read -r address balance; do
                echo "    $address: $balance KALE"
            done
            echo ""
        fi
        
        if [[ -f "c_addresses_leaderboard.csv" ]]; then
            echo "C Addresses (Contracts):"
            C_TOTAL=$(tail -n +2 c_addresses_leaderboard.csv | wc -l | tr -d ' ')
            C_WITH_BALANCE=$(tail -n +2 c_addresses_leaderboard.csv | awk -F',' '$2 != "0.0000000" { count++ } END { print count }')
            C_KALE_CONTRACTS=$(tail -n +2 c_addresses_leaderboard.csv | awk -F',' '$3 == "true" { count++ } END { print count }')
            echo "  Total contracts: $C_TOTAL"
            echo "  Contracts with balance: ${C_WITH_BALANCE:-0}"
            echo "  KALE farming contracts: ${C_KALE_CONTRACTS:-0}"
            echo ""
            echo "  Top 10 contracts:"
            head -n 11 c_addresses_leaderboard.csv | tail -n 10 | while IFS=',' read -r address balance is_kale; do
                if [[ "$is_kale" == "true" ]]; then
                    echo "    $address: $balance KALE (KALE Contract)"
                else
                    echo "    $address: $balance KALE"
                fi
            done
        fi
        
        echo ""
        echo "Configuration used:"
        echo "  KALE_ASSET_CODE: ${KALE_ASSET_CODE:-Not set}"
        echo "  KALE_ASSET_ISSUER: ${KALE_ASSET_ISSUER:-Not set}"
        echo "  KALE_TOKEN_CONTRACT: ${KALE_TOKEN_CONTRACT:-Not set}"
        
    } > "$REPORT_FILE"
    
    print_success "Summary report saved to: $REPORT_FILE"
    
    # Also display the report
    echo ""
    cat "$REPORT_FILE"
}

# Function to clean up files
cleanup_files() {
    print_header "Cleanup Options"
    
    echo "Which files would you like to remove?"
    echo "1) Transaction files only (kale_transactions*.csv)"
    echo "2) Address files only (c_addresses*.csv, g_addresses*.csv)"
    echo "3) Leaderboard files only (*_leaderboard.csv)"
    echo "4) All generated files"
    echo "5) Cancel"
    
    read -p "Enter choice (1-5): " choice
    
    case $choice in
        1)
            rm -f kale_transactions.csv kale_transactions_detailed.csv
            print_success "Transaction files removed"
            ;;
        2)
            rm -f c_addresses.csv c_addresses_detailed.csv g_addresses.csv g_addresses_detailed.csv
            print_success "Address files removed"
            ;;
        3)
            rm -f g_addresses_leaderboard.csv c_addresses_leaderboard.csv
            print_success "Leaderboard files removed"
            ;;
        4)
            rm -f kale_transactions*.csv c_addresses*.csv g_addresses*.csv *_leaderboard.csv
            print_success "All generated files removed"
            ;;
        *)
            print_info "Cleanup cancelled"
            ;;
    esac
}

# Main menu
show_menu() {
    print_header "KALE Contract Analysis Tool"
    
    echo "Select an option:"
    echo "1) Run full analysis (all steps)"
    echo "2) Fetch transactions only"
    echo "3) Check G address balances only"
    echo "4) Check C address balances only"
    echo "5) Generate summary report"
    echo "6) Clean up files"
    echo "7) Exit"
    echo ""
    read -p "Enter choice (1-7): " choice
    
    case $choice in
        1)
            check_requirements
            display_config
            fetch_transactions
            fetch_g_balances
            fetch_c_balances
            generate_summary
            ;;
        2)
            check_requirements
            fetch_transactions
            ;;
        3)
            check_requirements
            display_config
            fetch_g_balances
            ;;
        4)
            check_requirements
            display_config
            fetch_c_balances
            ;;
        5)
            generate_summary
            ;;
        6)
            cleanup_files
            ;;
        7)
            print_info "Exiting..."
            exit 0
            ;;
        *)
            print_error "Invalid choice"
            ;;
    esac
}

# Parse command line arguments
if [[ "$1" == "--help" || "$1" == "-h" ]]; then
    echo "KALE Contract Analysis Tool"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --help, -h        Show this help message"
    echo "  --full            Run full analysis without prompts"
    echo "  --cursor CURSOR   Resume transaction fetch from cursor"
    echo ""
    echo "Environment variables:"
    echo "  KALE_ASSET_CODE       Asset code (default: KALE)"
    echo "  KALE_ASSET_ISSUER     Asset issuer for traditional assets"
    echo "  KALE_TOKEN_CONTRACT   Contract address for SAC tokens"
    echo ""
    echo "Examples:"
    echo "  # Run with traditional asset"
    echo "  KALE_ASSET_ISSUER=GXXXXX... $0 --full"
    echo ""
    echo "  # Run with SAC token"
    echo "  KALE_TOKEN_CONTRACT=CXXXXX... $0 --full"
    echo ""
    echo "  # Resume from cursor"
    echo "  $0 --cursor 123456789"
    exit 0
fi

if [[ "$1" == "--full" ]]; then
    # Run full analysis without menu
    check_requirements
    display_config
    fetch_transactions
    fetch_g_balances
    fetch_c_balances
    generate_summary
elif [[ "$1" == "--cursor" && -n "$2" ]]; then
    # Resume from cursor
    check_requirements
    fetch_transactions "$2"
else
    # Show interactive menu
    while true; do
        show_menu
        echo ""
        read -p "Press Enter to continue..." 
    done
fi 