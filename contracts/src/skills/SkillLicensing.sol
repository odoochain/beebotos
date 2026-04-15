// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

/**
 * @title SkillLicensing
 * @dev Manages skill licensing and royalties
 */
contract SkillLicensing is AccessControl, ReentrancyGuard {
    bytes32 public constant LICENSOR_ROLE = keccak256("LICENSOR_ROLE");

    enum LicenseType {
        Perpetual,
        Subscription,
        UsageBased
    }

    struct License {
        uint256 skillId;
        address licensee;
        LicenseType licenseType;
        uint256 startTime;
        uint256 expiration;
        uint256 usageLimit;
        uint256 usageCount;
        uint256 price;
        bool active;
    }

    struct Royalty {
        address recipient;
        uint256 percentage; // Basis points (100 = 1%)
    }

    // License ID => License
    mapping(uint256 => License) public licenses;
    
    // Skill ID => Royalties
    mapping(uint256 => Royalty[]) public royalties;
    
    // Skill ID => Total royalty percentage
    mapping(uint256 => uint256) public totalRoyaltyPercentage;
    
    uint256 public licenseCount;
    uint256 public constant MAX_ROYALTY = 3000; // 30%

    event LicenseGranted(
        uint256 indexed licenseId,
        uint256 indexed skillId,
        address indexed licensee,
        LicenseType licenseType
    );
    
    event LicenseRevoked(uint256 indexed licenseId);
    event RoyaltyPaid(uint256 indexed skillId, address indexed recipient, uint256 amount);
    event UsageRecorded(uint256 indexed licenseId, uint256 usageCount);

    constructor() {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(LICENSOR_ROLE, msg.sender);
    }

    /**
     * @dev Set royalties for a skill
     */
    function setRoyalties(
        uint256 skillId,
        address[] calldata recipients,
        uint256[] calldata percentages
    ) external onlyRole(LICENSOR_ROLE) {
        require(recipients.length == percentages.length, "Length mismatch");
        
        uint256 totalPercentage = 0;
        for (uint256 i = 0; i < percentages.length; i++) {
            totalPercentage += percentages[i];
        }
        require(totalPercentage <= MAX_ROYALTY, "Royalty too high");

        // Clear existing royalties
        delete royalties[skillId];

        // Set new royalties
        for (uint256 i = 0; i < recipients.length; i++) {
            royalties[skillId].push(Royalty({
                recipient: recipients[i],
                percentage: percentages[i]
            }));
        }

        totalRoyaltyPercentage[skillId] = totalPercentage;
    }

    /**
     * @dev Grant a license
     */
    function grantLicense(
        uint256 skillId,
        address licensee,
        LicenseType licenseType,
        uint256 duration,
        uint256 usageLimit,
        uint256 price
    ) external payable nonReentrant returns (uint256) {
        require(msg.value >= price, "Insufficient payment");

        licenseCount++;
        uint256 licenseId = licenseCount;

        licenses[licenseId] = License({
            skillId: skillId,
            licensee: licensee,
            licenseType: licenseType,
            startTime: block.timestamp,
            expiration: duration > 0 ? block.timestamp + duration : 0,
            usageLimit: usageLimit,
            usageCount: 0,
            price: price,
            active: true
        });

        // Distribute royalties
        _distributeRoyalties(skillId, price);

        emit LicenseGranted(licenseId, skillId, licensee, licenseType);

        return licenseId;
    }

    /**
     * @dev Record usage for a license
     */
    function recordUsage(uint256 licenseId) external {
        License storage license = licenses[licenseId];
        require(license.active, "License not active");
        require(
            license.licenseType == LicenseType.UsageBased,
            "Not usage-based license"
        );
        require(
            license.usageLimit == 0 || license.usageCount < license.usageLimit,
            "Usage limit reached"
        );
        require(
            license.expiration == 0 || block.timestamp < license.expiration,
            "License expired"
        );

        license.usageCount++;

        emit UsageRecorded(licenseId, license.usageCount);
    }

    /**
     * @dev Revoke a license
     */
    function revokeLicense(uint256 licenseId) external onlyRole(LICENSOR_ROLE) {
        licenses[licenseId].active = false;
        emit LicenseRevoked(licenseId);
    }

    /**
     * @dev Check if license is valid
     */
    function isLicenseValid(uint256 licenseId) external view returns (bool) {
        License storage license = licenses[licenseId];
        if (!license.active) return false;
        if (license.expiration > 0 && block.timestamp >= license.expiration) return false;
        if (license.licenseType == LicenseType.UsageBased && 
            license.usageLimit > 0 && 
            license.usageCount >= license.usageLimit) return false;
        return true;
    }

    /**
     * @dev Get royalties for a skill
     */
    function getRoyalties(uint256 skillId) external view returns (Royalty[] memory) {
        return royalties[skillId];
    }

    /**
     * @dev Distribute royalties
     */
    function _distributeRoyalties(uint256 skillId, uint256 amount) internal {
        Royalty[] storage skillRoyalties = royalties[skillId];
        
        for (uint256 i = 0; i < skillRoyalties.length; i++) {
            Royalty storage royalty = skillRoyalties[i];
            uint256 royaltyAmount = (amount * royalty.percentage) / 10000;
            
            (bool success, ) = royalty.recipient.call{value: royaltyAmount}("");
            if (success) {
                emit RoyaltyPaid(skillId, royalty.recipient, royaltyAmount);
            }
        }
    }

    receive() external payable {}
}
