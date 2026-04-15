// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts-upgradeable/token/ERC721/ERC721Upgradeable.sol";
import "@openzeppelin/contracts-upgradeable/token/ERC721/extensions/ERC721EnumerableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/token/ERC721/extensions/ERC721URIStorageUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/token/common/ERC2981Upgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/security/PausableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/utils/Address.sol";
import "../interfaces/ISkillNFT.sol";

/**
 * @title SkillNFT
 * @notice ERC721 NFT for agent skills with ERC2981 royalty standard
 * 
 * Features:
 * - ERC2981 compliant royalty standard
 * - Upgradeable via UUPS
 * - Pausable for emergency
 * - Enumerable for easy querying
 * - URI storage for metadata
 */
contract SkillNFT is 
    ISkillNFT,
    ERC721Upgradeable,
    ERC721EnumerableUpgradeable,
    ERC721URIStorageUpgradeable,
    ERC2981Upgradeable,
    OwnableUpgradeable,
    PausableUpgradeable,
    UUPSUpgradeable 
{
    using Address for address;
    
    uint256 public tokenCounter;
    mapping(uint256 => Skill) private skills;
    
    // Royalty configuration: default 2.5% (250 basis points)
    uint96 public constant DEFAULT_ROYALTY_BPS = 250;
    uint96 public constant MAX_ROYALTY_BPS = 1000; // 10% max
    
    // Mapping for token-specific royalties (override default)
    mapping(uint256 => uint96) private _tokenRoyaltyBps;
    
    address private immutable __self;
    
    event SkillMinted(
        uint256 indexed tokenId, 
        address indexed creator, 
        string name, 
        string version,
        uint96 royaltyBps
    );
    event RoyaltyUpdated(uint256 indexed tokenId, uint96 newRoyaltyBps);
    event DefaultRoyaltySet(address indexed receiver, uint96 royaltyBps);
    event TokenRoyaltyReset(uint256 indexed tokenId);
    
    constructor() {
        __self = address(this);
    }
    
    function initialize() public initializer {
        __ERC721_init("BeeBotOS Skill", "SKILL");
        __ERC721Enumerable_init();
        __ERC721URIStorage_init();
        __ERC2981_init();
        __Ownable_init();
        __Pausable_init();
        __UUPSUpgradeable_init();
        
        tokenCounter = 0;
        
        // Set default royalty to contract owner (can be updated)
        _setDefaultRoyalty(msg.sender, DEFAULT_ROYALTY_BPS);
        emit DefaultRoyaltySet(msg.sender, DEFAULT_ROYALTY_BPS);
    }
    
    function mintSkill(
        string calldata name,
        string calldata version,
        string calldata metadataURI,
        bool transferable
    ) external override whenNotPaused returns (uint256 tokenId) {
        require(bytes(name).length > 0, "SkillNFT: name required");
        require(bytes(version).length > 0, "SkillNFT: version required");
        
        tokenId = tokenCounter++;
        
        Skill storage skill = skills[tokenId];
        skill.tokenId = tokenId;
        skill.creator = msg.sender;
        skill.name = name;
        skill.version = version;
        skill.metadataURI = metadataURI;
        skill.isTransferable = transferable;
        skill.createdAt = block.timestamp;
        
        _safeMint(msg.sender, tokenId);
        _setTokenURI(tokenId, metadataURI);
        
        // Set token-specific royalty to creator with default rate
        _tokenRoyaltyBps[tokenId] = DEFAULT_ROYALTY_BPS;
        _setTokenRoyalty(tokenId, msg.sender, DEFAULT_ROYALTY_BPS);
        
        emit SkillMinted(tokenId, msg.sender, name, version, DEFAULT_ROYALTY_BPS);
    }
    
    /**
     * @dev Update royalty for a specific token (only owner of token)
     * @param tokenId The token to update
     * @param royaltyBps New royalty in basis points (max 10%)
     */
    function setTokenRoyalty(uint256 tokenId, uint96 royaltyBps) external {
        require(_exists(tokenId), "SkillNFT: token does not exist");
        require(ownerOf(tokenId) == msg.sender, "SkillNFT: not token owner");
        require(royaltyBps <= MAX_ROYALTY_BPS, "SkillNFT: royalty too high");
        
        _tokenRoyaltyBps[tokenId] = royaltyBps;
        _setTokenRoyalty(tokenId, msg.sender, royaltyBps);
        
        emit RoyaltyUpdated(tokenId, royaltyBps);
    }
    
    /**
     * @dev Reset token royalty to default (only owner)
     * @param tokenId The token to reset
     */
    function resetTokenRoyalty(uint256 tokenId) external onlyOwner {
        require(_exists(tokenId), "SkillNFT: token does not exist");
        _resetTokenRoyalty(tokenId);
        delete _tokenRoyaltyBps[tokenId];
        emit TokenRoyaltyReset(tokenId);
    }
    
    /**
     * @dev Update default royalty for all new tokens
     * @param receiver Royalty receiver
     * @param feeNumerator Royalty in basis points
     */
    function setDefaultRoyalty(address receiver, uint96 feeNumerator) external onlyOwner {
        require(feeNumerator <= MAX_ROYALTY_BPS, "SkillNFT: royalty too high");
        _setDefaultRoyalty(receiver, feeNumerator);
        emit DefaultRoyaltySet(receiver, feeNumerator);
    }
    
    /**
     * @dev Get royalty info for a token sale (ERC2981)
     */
    function royaltyInfo(uint256 tokenId, uint256 salePrice) 
        public 
        view 
        override 
        returns (address receiver, uint256 royaltyAmount) 
    {
        return super.royaltyInfo(tokenId, salePrice);
    }
    
    /**
     * @dev Check if a token supports an interface (ERC165)
     */
    function supportsInterface(bytes4 interfaceId) 
        public 
        view 
        override(
            ERC721Upgradeable,
            ERC721EnumerableUpgradeable,
            ERC721URIStorageUpgradeable,
            ERC2981Upgradeable
        ) 
        returns (bool) 
    {
        return super.supportsInterface(interfaceId);
    }
    
    function getSkill(uint256 tokenId) external view override returns (Skill memory) {
        require(_exists(tokenId), "SkillNFT: token does not exist");
        return skills[tokenId];
    }
    
    /**
     * @dev Get token creator (royalty receiver)
     */
    function getCreator(uint256 tokenId) external view returns (address) {
        require(_exists(tokenId), "SkillNFT: token does not exist");
        return skills[tokenId].creator;
    }
    
    /**
     * @dev Check if token is transferable
     */
    function isTransferable(uint256 tokenId) external view returns (bool) {
        require(_exists(tokenId), "SkillNFT: token does not exist");
        return skills[tokenId].isTransferable;
    }
    
    /**
     * @dev Batch mint skills
     */
    function batchMintSkill(
        string[] calldata names,
        string[] calldata versions,
        string[] calldata metadataURIs,
        bool[] calldata transferableFlags
    ) external whenNotPaused returns (uint256[] memory tokenIds) {
        require(
            names.length == versions.length && 
            versions.length == metadataURIs.length &&
            metadataURIs.length == transferableFlags.length,
            "SkillNFT: array length mismatch"
        );
        require(names.length > 0, "SkillNFT: empty arrays");
        require(names.length <= 50, "SkillNFT: batch too large");
        
        tokenIds = new uint256[](names.length);
        
        for (uint i = 0; i < names.length; i++) {
            tokenIds[i] = this.mintSkill(names[i], versions[i], metadataURIs[i], transferableFlags[i]);
        }
        
        return tokenIds;
    }
    
    /**
     * @dev Get all tokens owned by an address
     */
    function getTokensByOwner(address owner) external view returns (uint256[] memory) {
        uint256 balance = balanceOf(owner);
        uint256[] memory tokens = new uint256[](balance);
        
        for (uint i = 0; i < balance; i++) {
            tokens[i] = tokenOfOwnerByIndex(owner, i);
        }
        
        return tokens;
    }
    
    /**
     * @dev Get total supply
     */
    function totalSupply() public view override(ERC721EnumerableUpgradeable) returns (uint256) {
        return super.totalSupply();
    }
    
    // Pause functions
    function pause() external onlyOwner {
        _pause();
    }
    
    function unpause() external onlyOwner {
        _unpause();
    }
    
    function _beforeTokenTransfer(
        address from,
        address to,
        uint256 tokenId,
        uint256 batchSize
    ) internal override(ERC721Upgradeable, ERC721EnumerableUpgradeable) whenNotPaused {
        super._beforeTokenTransfer(from, to, tokenId, batchSize);
        
        if (from != address(0) && to != address(0)) {
            require(skills[tokenId].isTransferable, "SkillNFT: skill not transferable");
        }
    }
    
    function _burn(uint256 tokenId) internal override(ERC721Upgradeable, ERC721URIStorageUpgradeable) {
        super._burn(tokenId);
    }
    
    function tokenURI(uint256 tokenId) 
        public 
        view 
        override(ERC721Upgradeable, ERC721URIStorageUpgradeable) 
        returns (string memory) 
    {
        return super.tokenURI(tokenId);
    }
    
    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}
    
    // Storage gap
    uint256[50] private __gap;
}
