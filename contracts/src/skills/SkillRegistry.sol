// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "./SkillNFT.sol";

/**
 * @title SkillRegistry
 * @dev Registry for managing agent skills
 */
contract SkillRegistry is AccessControl, Pausable, ReentrancyGuard {
    bytes32 public constant REGISTRAR_ROLE = keccak256("REGISTRAR_ROLE");
    bytes32 public constant VERIFIER_ROLE = keccak256("VERIFIER_ROLE");

    struct Skill {
        string name;
        string version;
        string description;
        string category;
        address author;
        bytes32 codeHash;
        bytes32 metadataHash;
        uint256 createdAt;
        bool verified;
        bool active;
        uint256 price;
    }

    struct SkillInstallation {
        address agent;
        uint256 skillId;
        uint256 installedAt;
        uint256 expiresAt;
        bool active;
    }

    // Skill ID => Skill info
    mapping(uint256 => Skill) public skills;
    
    // Agent => Installed skills
    mapping(address => mapping(uint256 => SkillInstallation)) public installations;
    mapping(address => uint256[]) public agentSkills;
    
    // Category => Skill IDs
    mapping(string => uint256[]) public skillsByCategory;
    
    // Code hash => Skill ID (prevent duplicates)
    mapping(bytes32 => uint256) public skillByCodeHash;
    
    uint256 public skillCount;
    SkillNFT public skillNFT;
    
    uint256 public constant VERIFICATION_FEE = 0.1 ether;
    uint256 public constant REGISTRATION_FEE = 0.01 ether;

    event SkillRegistered(
        uint256 indexed skillId,
        string name,
        address indexed author,
        bytes32 codeHash
    );
    
    event SkillVerified(uint256 indexed skillId, address indexed verifier);
    event SkillInstalled(uint256 indexed skillId, address indexed agent);
    event SkillUninstalled(uint256 indexed skillId, address indexed agent);

    constructor(address _skillNFT) {
        skillNFT = SkillNFT(_skillNFT);
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(REGISTRAR_ROLE, msg.sender);
        _grantRole(VERIFIER_ROLE, msg.sender);
    }

    /**
     * @dev Register a new skill
     */
    function registerSkill(
        string calldata name,
        string calldata version,
        string calldata description,
        string calldata category,
        bytes32 codeHash,
        bytes32 metadataHash,
        uint256 price
    ) external payable whenNotPaused returns (uint256) {
        require(msg.value >= REGISTRATION_FEE, "Insufficient fee");
        require(skillByCodeHash[codeHash] == 0, "Skill already registered");
        require(bytes(name).length > 0, "Name required");

        skillCount++;
        uint256 skillId = skillCount;

        skills[skillId] = Skill({
            name: name,
            version: version,
            description: description,
            category: category,
            author: msg.sender,
            codeHash: codeHash,
            metadataHash: metadataHash,
            createdAt: block.timestamp,
            verified: false,
            active: true,
            price: price
        });

        skillByCodeHash[codeHash] = skillId;
        skillsByCategory[category].push(skillId);

        // Mint NFT for the skill
        // Note: metadataHash is bytes32, converting to hex string for URI
        skillNFT.mintSkill(name, version, _toHexString(metadataHash), true);

        emit SkillRegistered(skillId, name, msg.sender, codeHash);

        return skillId;
    }

    /**
     * @dev Verify a skill
     */
    function verifySkill(uint256 skillId) external payable onlyRole(VERIFIER_ROLE) whenNotPaused {
        require(msg.value >= VERIFICATION_FEE, "Insufficient fee");
        require(skills[skillId].active, "Skill not active");
        require(!skills[skillId].verified, "Already verified");

        skills[skillId].verified = true;

        emit SkillVerified(skillId, msg.sender);
    }

    /**
     * @dev Install a skill for an agent
     */
    function installSkill(
        uint256 skillId,
        address agent,
        uint256 duration
    ) external payable nonReentrant whenNotPaused {
        Skill storage skill = skills[skillId];
        require(skill.active, "Skill not active");

        if (skill.price > 0) {
            require(msg.value >= skill.price, "Insufficient payment");
            // Transfer payment to author
            (bool success, ) = skill.author.call{value: skill.price}("");
            require(success, "Payment failed");
        }

        installations[agent][skillId] = SkillInstallation({
            agent: agent,
            skillId: skillId,
            installedAt: block.timestamp,
            expiresAt: duration > 0 ? block.timestamp + duration : 0,
            active: true
        });

        agentSkills[agent].push(skillId);

        emit SkillInstalled(skillId, agent);
    }

    /**
     * @dev Uninstall a skill
     */
    function uninstallSkill(uint256 skillId, address agent) external whenNotPaused {
        require(
            msg.sender == agent || msg.sender == skills[skillId].author,
            "Not authorized"
        );

        installations[agent][skillId].active = false;

        emit SkillUninstalled(skillId, agent);
    }

    /**
     * @dev Check if agent has skill installed and active
     */
    function hasSkill(address agent, uint256 skillId) external view returns (bool) {
        SkillInstallation storage inst = installations[agent][skillId];
        if (!inst.active) return false;
        if (inst.expiresAt > 0 && block.timestamp > inst.expiresAt) return false;
        return true;
    }

    /**
     * @dev Get skills by category
     */
    function getSkillsByCategory(string calldata category) external view returns (uint256[] memory) {
        return skillsByCategory[category];
    }

    /**
     * @dev Get all skills for an agent
     */
    function getAgentSkills(address agent) external view returns (uint256[] memory) {
        return agentSkills[agent];
    }

    /**
     * @dev Update skill price
     */
    function updateSkillPrice(uint256 skillId, uint256 newPrice) external whenNotPaused {
        require(msg.sender == skills[skillId].author, "Not author");
        skills[skillId].price = newPrice;
    }

    /**
     * @dev Deactivate a skill
     */
    function deactivateSkill(uint256 skillId) external whenNotPaused {
        require(
            msg.sender == skills[skillId].author || hasRole(DEFAULT_ADMIN_ROLE, msg.sender),
            "Not authorized"
        );
        skills[skillId].active = false;
    }

    /**
     * @dev Withdraw fees
     */
    function withdrawFees() external onlyRole(DEFAULT_ADMIN_ROLE) nonReentrant {
        uint256 balance = address(this).balance;
        require(balance > 0, "No fees to withdraw");
        (bool success, ) = msg.sender.call{value: balance}("");
        require(success, "Withdrawal failed");
    }

    receive() external payable {}
    
    /**
     * @dev Convert bytes32 to hex string
     */
    function _toHexString(bytes32 data) internal pure returns (string memory) {
        bytes memory alphabet = "0123456789abcdef";
        bytes memory str = new bytes(66);
        str[0] = "0";
        str[1] = "x";
        for (uint i = 0; i < 32; i++) {
            str[2 + i * 2] = alphabet[uint(uint8(data[i] >> 4))];
            str[3 + i * 2] = alphabet[uint(uint8(data[i] & 0x0f))];
        }
        return string(str);
    }
}
