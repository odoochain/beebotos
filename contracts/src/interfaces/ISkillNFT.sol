// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/**
 * @title ISkillNFT
 * @notice Interface for SkillNFT contract
 * @dev Note: Does not inherit IERC721 to avoid conflicts with upgradeable version
 */
interface ISkillNFT {
    struct Skill {
        uint256 tokenId;
        address creator;
        string name;
        string version;
        string metadataURI;
        bool isTransferable;
        uint256 createdAt;
    }
    
    event SkillMinted(uint256 indexed tokenId, address indexed creator, string name);
    
    function mintSkill(string calldata name, string calldata version, string calldata metadataURI, bool isTransferable) external returns (uint256);
    function getSkill(uint256 tokenId) external view returns (Skill memory);
}
