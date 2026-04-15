// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "../../contracts/solidity/core/AgentRegistry.sol";

/**
 * @title AgentExample
 * @dev Example contract showing how to interact with BeeBotOS
 */
contract AgentExample {
    AgentRegistry public registry;
    
    event TaskCreated(bytes32 indexed taskId, address indexed agent);
    event TaskCompleted(bytes32 indexed taskId, string result);
    
    struct Task {
        bytes32 id;
        address agent;
        string description;
        bool completed;
        string result;
    }
    
    mapping(bytes32 => Task) public tasks;
    bytes32[] public taskList;
    
    constructor(address _registry) {
        registry = AgentRegistry(_registry);
    }
    
    function createTask(string memory _description) external returns (bytes32) {
        bytes32 taskId = keccak256(
            abi.encodePacked(
                msg.sender,
                block.timestamp,
                _description
            )
        );
        
        tasks[taskId] = Task({
            id: taskId,
            agent: msg.sender,
            description: _description,
            completed: false,
            result: ""
        });
        
        taskList.push(taskId);
        
        emit TaskCreated(taskId, msg.sender);
        return taskId;
    }
    
    function completeTask(bytes32 _taskId, string memory _result) external {
        Task storage task = tasks[_taskId];
        require(task.agent == msg.sender, "Not task owner");
        require(!task.completed, "Already completed");
        
        task.completed = true;
        task.result = _result;
        
        emit TaskCompleted(_taskId, _result);
    }
    
    function getTaskCount() external view returns (uint256) {
        return taskList.length;
    }
}
