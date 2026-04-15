// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Script.sol";
import "forge-std/console.sol";
import "@openzeppelin/contracts/governance/TimelockController.sol";
import "@contracts/dao/AgentDAO.sol";
import "@contracts/dao/BeeToken.sol";
import "@contracts/dao/TreasuryManager.sol";

contract DeployDAO is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);
        
        console.log("Deploying from:", deployer);
        
        vm.startBroadcast(deployerPrivateKey);
        
        // Deploy BeeToken with proper allocation
        address treasuryAddr = address(0x1); // placeholder, will be replaced
        address team = address(0x2);
        address investors = address(0x3);
        address ecosystem = address(0x4);
        address liquidity = address(0x5);
        
        BeeToken beeToken = new BeeToken(treasuryAddr, team, investors, ecosystem, liquidity);
        console.log("BeeToken deployed at:", address(beeToken));
        
        // Deploy TreasuryManager with DAO as owner (deployer temporarily)
        TreasuryManager treasury = new TreasuryManager(deployer);
        console.log("TreasuryManager deployed at:", address(treasury));
        
        // Deploy Mock contracts for dependencies (in production, these should be real contracts)
        address reputationSystem = address(0x6); // placeholder
        address agentRegistry = address(0x7);    // placeholder
        address agentIdentity = address(0x8);    // placeholder
        
        // Deploy Timelock (simplified - in production use proper TimelockController)
        TimelockController timelock = new TimelockController(
            1 days, // min delay
            new address[](0), // proposers (empty initially)
            new address[](0), // executors (empty initially)
            deployer
        );
        
        // Deploy AgentDAO
        AgentDAO dao = new AgentDAO(
            "BeeBotOS DAO",
            IVotes(address(beeToken)),
            timelock,
            reputationSystem,
            agentRegistry,
            agentIdentity,
            1,      // voting delay (blocks)
            40320,  // voting period (blocks) ~ 7 days
            400     // quorum numerator (4%)
        );
        console.log("AgentDAO deployed at:", address(dao));
        
        // Grant DAO roles for BeeToken
        bytes32 DEFAULT_ADMIN_ROLE = 0x00;
        beeToken.grantRole(DEFAULT_ADMIN_ROLE, address(dao));
        console.log("BeeToken admin role granted to DAO");
        
        // Grant DAO roles for TreasuryManager
        treasury.grantRole(DEFAULT_ADMIN_ROLE, address(dao));
        treasury.grantRole(treasury.TREASURY_ADMIN(), address(dao));
        console.log("Treasury admin roles granted to DAO");
        
        // Mint initial supply to deployer
        beeToken.mint(deployer, 1000000 * 10**18, "initial supply");
        console.log("Minted 1,000,000 BEE to deployer");
        
        vm.stopBroadcast();
        
        // Write deployment info
        string memory deployment = string.concat(
            "{\n",
            "  \"beeToken\": \"", vm.toString(address(beeToken)), "\",\n",
            "  \"treasury\": \"", vm.toString(address(treasury)), "\",\n",
            "  \"dao\": \"", vm.toString(address(dao)), "\"\n",
            "}\n"
        );
        
        vm.writeFile("deployment.json", deployment);
        console.log("Deployment info saved to deployment.json");
    }
}
