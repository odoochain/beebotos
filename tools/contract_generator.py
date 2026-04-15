#!/usr/bin/env python3
"""
BeeBotOS Smart Contract Generator
Generates Solidity smart contracts from templates and specifications.
"""

import argparse
import json
import os
from datetime import datetime
from typing import Dict, List, Optional

# Contract templates
TEMPLATES = {
    "erc20": """// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title {name}
 * @dev {description}
 */
contract {name} is ERC20, Ownable {
    uint256 public constant MAX_SUPPLY = {max_supply} * 10**decimals();
    
    constructor() ERC20("{token_name}", "{symbol}") Ownable(msg.sender) {
        _mint(msg.sender, {initial_supply} * 10**decimals());
    }
    
    function mint(address to, uint256 amount) public onlyOwner {
        require(totalSupply() + amount <= MAX_SUPPLY, "Max supply exceeded");
        _mint(to, amount);
    }
    
    function burn(uint256 amount) public {
        _burn(msg.sender, amount);
    }
}
""",
    
    "erc721": """// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/Counters.sol";

/**
 * @title {name}
 * @dev {description}
 */
contract {name} is ERC721, Ownable {
    using Counters for Counters.Counter;
    
    Counters.Counter private _tokenIdCounter;
    
    uint256 public constant MAX_SUPPLY = {max_supply};
    uint256 public mintPrice = {mint_price} ether;
    string public baseTokenURI;
    
    constructor(string memory _baseURI) ERC721("{token_name}", "{symbol}") Ownable(msg.sender) {
        baseTokenURI = _baseURI;
    }
    
    function mint() public payable {
        require(msg.value >= mintPrice, "Insufficient payment");
        require(_tokenIdCounter.current() < MAX_SUPPLY, "Max supply reached");
        
        uint256 tokenId = _tokenIdCounter.current();
        _tokenIdCounter.increment();
        _safeMint(msg.sender, tokenId);
    }
    
    function _baseURI() internal view override returns (string memory) {
        return baseTokenURI;
    }
    
    function withdraw() public onlyOwner {
        payable(owner()).transfer(address(this).balance);
    }
}
""",
    
    "dao": """// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/governance/Governor.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorSettings.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorCountingSimple.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorVotes.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorVotesQuorumFraction.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorTimelockControl.sol";

/**
 * @title {name}
 * @dev {description}
 */
contract {name} is
    Governor,
    GovernorSettings,
    GovernorCountingSimple,
    GovernorVotes,
    GovernorVotesQuorumFraction,
    GovernorTimelockControl
{
    constructor(
        IVotes _token,
        TimelockController _timelock
    )
        Governor("{dao_name}")
        GovernorSettings(
            {voting_delay},
            {voting_period},
            {proposal_threshold}
        )
        GovernorVotes(_token)
        GovernorVotesQuorumFraction({quorum})
        GovernorTimelockControl(_timelock)
    {}

    function votingDelay() public view override(IGovernor, GovernorSettings) returns (uint256) {
        return super.votingDelay();
    }

    function votingPeriod() public view override(IGovernor, GovernorSettings) returns (uint256) {
        return super.votingPeriod();
    }

    function quorum(uint256 blockNumber) public view override(IGovernor, GovernorVotesQuorumFraction) returns (uint256) {
        return super.quorum(blockNumber);
    }

    function state(uint256 proposalId) public view override(Governor, GovernorTimelockControl) returns (ProposalState) {
        return super.state(proposalId);
    }

    function propose(
        address[] memory targets,
        uint256[] memory values,
        bytes[] memory calldatas,
        string memory description
    ) public override(Governor, IGovernor) returns (uint256) {
        return super.propose(targets, values, calldatas, description);
    }

    function _execute(
        uint256 proposalId,
        address[] memory targets,
        uint256[] memory values,
        bytes[] memory calldatas,
        bytes32 descriptionHash
    ) internal override(Governor, GovernorTimelockControl) {
        super._execute(proposalId, targets, values, calldatas, descriptionHash);
    }

    function _cancel(
        address[] memory targets,
        uint256[] memory values,
        bytes[] memory calldatas,
        bytes32 descriptionHash
    ) internal override(Governor, GovernorTimelockControl) returns (uint256) {
        return super._cancel(targets, values, calldatas, descriptionHash);
    }

    function _executor() internal view override(Governor, GovernorTimelockControl) returns (address) {
        return super._executor();
    }

    function supportsInterface(bytes4 interfaceId) public view override(Governor, GovernorTimelockControl) returns (bool) {
        return super.supportsInterface(interfaceId);
    }
}
""",
}


def generate_contract(
    template_type: str,
    name: str,
    params: Dict[str, str],
    output_dir: str = "generated"
) -> str:
    """Generate a contract from template."""
    
    if template_type not in TEMPLATES:
        raise ValueError(f"Unknown template: {template_type}")
    
    template = TEMPLATES[template_type]
    
    # Merge default params with provided params
    default_params = {
        "name": name,
        "token_name": name,
        "dao_name": name,
        "description": f"Auto-generated {template_type} contract",
        "symbol": name[:3].upper(),
        "max_supply": "1000000000",
        "initial_supply": "1000000",
        "mint_price": "0.01",
        "voting_delay": "1 days",
        "voting_period": "1 weeks",
        "proposal_threshold": "1000e18",
        "quorum": "4",
    }
    default_params.update(params)
    
    # Generate contract code
    contract_code = template.format(**default_params)
    
    # Create output directory
    os.makedirs(output_dir, exist_ok=True)
    
    # Save contract
    filename = f"{name}.sol"
    filepath = os.path.join(output_dir, filename)
    
    with open(filepath, "w") as f:
        f.write(contract_code)
    
    # Generate metadata
    metadata = {
        "name": name,
        "template": template_type,
        "generated_at": datetime.now().isoformat(),
        "parameters": default_params,
        "file": filepath,
    }
    
    metadata_path = os.path.join(output_dir, f"{name}.json")
    with open(metadata_path, "w") as f:
        json.dump(metadata, f, indent=2)
    
    return filepath


def main():
    parser = argparse.ArgumentParser(description="BeeBotOS Contract Generator")
    parser.add_argument("template", choices=TEMPLATES.keys(), help="Contract template type")
    parser.add_argument("name", help="Contract name")
    parser.add_argument("-o", "--output", default="generated", help="Output directory")
    parser.add_argument("-p", "--params", type=json.loads, default="{}", help="Template parameters as JSON")
    
    args = parser.parse_args()
    
    try:
        filepath = generate_contract(args.template, args.name, args.params, args.output)
        print(f"✅ Contract generated: {filepath}")
    except Exception as e:
        print(f"❌ Error: {e}")
        return 1
    
    return 0


if __name__ == "__main__":
    exit(main())
