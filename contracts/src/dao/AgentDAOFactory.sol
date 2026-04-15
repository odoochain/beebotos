// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "./AgentDAO.sol";
import "@openzeppelin/contracts/governance/TimelockController.sol";

/**
 * @title AgentDAOFactory
 * @dev Factory for creating AgentDAO instances
 */
contract AgentDAOFactory {
    
    struct DAOMetadata {
        address daoAddress;
        address creator;
        string name;
        string description;
        uint256 createdAt;
    }
    
    mapping(address => DAOMetadata) public daos;
    address[] public allDAOs;
    
    event DAOCreated(address indexed daoAddress, address indexed creator, string name);
    
    function createDAO(
        string calldata name,
        string calldata description,
        address token,
        address reputationSystem,
        address agentRegistry,
        address agentIdentity,
        uint256 votingDelay,
        uint256 votingPeriod,
        uint256 quorumNumerator
    ) external returns (address) {
        // Create timelock for the DAO
        address[] memory proposers = new address[](1);
        address[] memory executors = new address[](1);
        proposers[0] = msg.sender;
        executors[0] = msg.sender;
        TimelockController timelock = new TimelockController(
            2 days,
            proposers,
            executors,
            msg.sender
        );
        
        // Deploy new DAO instance
        AgentDAO dao = new AgentDAO(
            name,
            IVotes(token),
            timelock,
            reputationSystem,
            agentRegistry,
            agentIdentity,
            votingDelay,
            votingPeriod,
            quorumNumerator
        );
        
        DAOMetadata memory metadata = DAOMetadata({
            daoAddress: address(dao),
            creator: msg.sender,
            name: name,
            description: description,
            createdAt: block.timestamp
        });
        
        daos[address(dao)] = metadata;
        allDAOs.push(address(dao));
        
        emit DAOCreated(address(dao), msg.sender, name);
        
        return address(dao);
    }
    
    function getDAOCount() external view returns (uint256) {
        return allDAOs.length;
    }
    
    function getDAOs(uint256 offset, uint256 limit) external view returns (DAOMetadata[] memory) {
        uint256 end = offset + limit;
        if (end > allDAOs.length) {
            end = allDAOs.length;
        }
        
        DAOMetadata[] memory result = new DAOMetadata[](end - offset);
        for (uint256 i = offset; i < end; i++) {
            result[i - offset] = daos[allDAOs[i]];
        }
        
        return result;
    }
}
