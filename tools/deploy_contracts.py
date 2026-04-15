#!/usr/bin/env python3
"""Contract deployment utility for BeeBotOS"""

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Dict, List, Optional

import requests
from web3 import Web3
from eth_account import Account


class ContractDeployer:
    """Deploys BeeBotOS smart contracts"""
    
    def __init__(self, rpc_url: str, private_key: str):
        self.w3 = Web3(Web3.HTTPProvider(rpc_url))
        self.account = Account.from_key(private_key)
        self.nonce = self.w3.eth.get_transaction_count(self.account.address)
        
    def deploy_contract(
        self,
        bytecode: str,
        abi: List[Dict],
        constructor_args: Optional[List] = None,
        gas_price: Optional[int] = None
    ) -> str:
        """Deploy a contract and return address"""
        
        if gas_price is None:
            gas_price = self.w3.eth.gas_price
            
        Contract = self.w3.eth.contract(abi=abi, bytecode=bytecode)
        
        if constructor_args:
            tx = Contract.constructor(*constructor_args).build_transaction({
                'from': self.account.address,
                'nonce': self.nonce,
                'gasPrice': gas_price,
            })
        else:
            tx = Contract.constructor().build_transaction({
                'from': self.account.address,
                'nonce': self.nonce,
                'gasPrice': gas_price,
            })
            
        signed_tx = self.account.sign_transaction(tx)
        tx_hash = self.w3.eth.send_raw_transaction(signed_tx.rawTransaction)
        tx_receipt = self.w3.eth.wait_for_transaction_receipt(tx_hash)
        
        self.nonce += 1
        
        return tx_receipt.contractAddress
    
    def deploy_dao(self, token_address: str) -> Dict[str, str]:
        """Deploy full DAO suite"""
        addresses = {}
        
        # Load contract artifacts
        contracts_dir = Path(__file__).parent.parent / "contracts" / "artifacts"
        
        # Deploy BeeToken (if not provided)
        if not token_address:
            with open(contracts_dir / "BeeToken.json") as f:
                artifact = json.load(f)
            token_address = self.deploy_contract(
                artifact["bytecode"],
                artifact["abi"]
            )
            addresses["BeeToken"] = token_address
            print(f"BeeToken deployed: {token_address}")
        else:
            addresses["BeeToken"] = token_address
        
        # Deploy AgentDAO
        with open(contracts_dir / "AgentDAO.json") as f:
            artifact = json.load(f)
        dao_address = self.deploy_contract(
            artifact["bytecode"],
            artifact["abi"],
            [token_address]
        )
        addresses["AgentDAO"] = dao_address
        print(f"AgentDAO deployed: {dao_address}")
        
        # Deploy TreasuryManager
        with open(contracts_dir / "TreasuryManager.json") as f:
            artifact = json.load(f)
        treasury_address = self.deploy_contract(
            artifact["bytecode"],
            artifact["abi"],
            [dao_address]
        )
        addresses["TreasuryManager"] = treasury_address
        print(f"TreasuryManager deployed: {treasury_address}")
        
        # Deploy ReputationManager
        with open(contracts_dir / "ReputationManager.json") as f:
            artifact = json.load(f)
        reputation_address = self.deploy_contract(
            artifact["bytecode"],
            artifact["abi"],
            [dao_address]
        )
        addresses["ReputationManager"] = reputation_address
        print(f"ReputationManager deployed: {reputation_address}")
        
        return addresses
    
    def verify_contract(
        self,
        address: str,
        contract_name: str,
        network: str = "monad"
    ) -> bool:
        """Verify contract on block explorer"""
        
        api_key = os.getenv("MONADSCAN_API_KEY")
        if not api_key:
            print("Warning: MONADSCAN_API_KEY not set, skipping verification")
            return False
        
        # Load source code
        contracts_dir = Path(__file__).parent.parent / "contracts"
        source_file = contracts_dir / "solidity" / f"{contract_name}.sol"
        
        if not source_file.exists():
            print(f"Source file not found: {source_file}")
            return False
        
        source_code = source_file.read_text()
        
        # API endpoint
        if network == "monad":
            api_url = "https://api.monadscan.io/api"
        else:
            raise ValueError(f"Unknown network: {network}")
        
        params = {
            "module": "contract",
            "action": "verifysourcecode",
            "apikey": api_key,
        }
        
        data = {
            "contractaddress": address,
            "sourceCode": source_code,
            "contractname": contract_name,
            "compilerversion": "v0.8.24+commit.80a8f1e3",
            "optimizationUsed": "1",
            "runs": "200",
            "evmversion": "paris",
            "licenseType": "3",  # MIT
        }
        
        response = requests.post(api_url, params=params, data=data)
        result = response.json()
        
        if result["status"] == "1":
            print(f"Verification submitted: {result['result']}")
            return True
        else:
            print(f"Verification failed: {result['result']}")
            return False


def main():
    parser = argparse.ArgumentParser(description="Deploy BeeBotOS contracts")
    parser.add_argument("--network", default="monad", choices=["monad", "monad-testnet", "local"])
    parser.add_argument("--rpc-url", help="RPC endpoint URL")
    parser.add_argument("--private-key", help="Deployer private key")
    parser.add_argument("--token-address", help="Existing token address (optional)")
    parser.add_argument("--verify", action="store_true", help="Verify on explorer")
    parser.add_argument("--output", default="deployments.json", help="Output file")
    
    args = parser.parse_args()
    
    # Get RPC URL
    rpc_url = args.rpc_url or os.getenv(f"{args.network.upper()}_RPC_URL")
    if not rpc_url:
        print(f"Error: --rpc-url or {args.network.upper()}_RPC_URL required")
        sys.exit(1)
    
    # Get private key
    private_key = args.private_key or os.getenv("DEPLOYER_PRIVATE_KEY")
    if not private_key:
        print("Error: --private-key or DEPLOYER_PRIVATE_KEY required")
        sys.exit(1)
    
    # Deploy
    deployer = ContractDeployer(rpc_url, private_key)
    
    print(f"Deploying to {args.network}...")
    print(f"Deployer: {deployer.account.address}")
    print()
    
    addresses = deployer.deploy_dao(args.token_address or "")
    
    # Save deployments
    deployment_info = {
        "network": args.network,
        "deployer": deployer.account.address,
        "contracts": addresses,
    }
    
    with open(args.output, "w") as f:
        json.dump(deployment_info, f, indent=2)
    
    print(f"\nDeployments saved to: {args.output}")
    
    # Verify
    if args.verify:
        print("\nVerifying contracts...")
        for name, address in addresses.items():
            if name != "BeeToken" or not args.token_address:
                deployer.verify_contract(address, name, args.network)
    
    print("\nDone!")


if __name__ == "__main__":
    main()
