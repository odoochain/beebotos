// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../../src/skills/SkillNFT.sol";

/**
 * @title SkillNFTTest
 * @dev Comprehensive tests for SkillNFT (Target: 90%+ coverage)
 */
contract SkillNFTTest is Test {
    SkillNFT public skillNFT;
    
    address public owner = address(1);
    address public creator1 = address(2);
    address public creator2 = address(3);
    address public buyer = address(4);
    
    uint256 public tokenId1;
    uint256 public tokenId2;
    
    function setUp() public {
        vm.prank(owner);
        skillNFT = new SkillNFT();
        skillNFT.initialize();
    }
    
    // ============ Initialization Tests ============
    
    function testInitialization() public view {
        assertEq(skillNFT.name(), "BeeBotOS Skill");
        assertEq(skillNFT.symbol(), "SKILL");
        assertEq(skillNFT.owner(), owner);
    }
    
    function testCannotInitializeTwice() public {
        vm.prank(owner);
        vm.expectRevert("SkillNFT: already initialized");
        skillNFT.initialize();
    }
    
    function testDefaultRoyalty() public {
        (address receiver, uint256 royaltyAmount) = skillNFT.royaltyInfo(0, 10000);
        assertEq(receiver, owner);
        assertEq(royaltyAmount, 250); // 2.5%
    }
    
    // ============ Mint Tests ============
    
    function testMintSkill() public {
        vm.prank(creator1);
        tokenId1 = skillNFT.mintSkill(
            "Trading Bot",
            "v1.0",
            "ipfs://metadata1",
            true
        );
        
        assertEq(tokenId1, 0);
        assertEq(skillNFT.ownerOf(tokenId1), creator1);
        assertEq(skillNFT.tokenURI(tokenId1), "ipfs://metadata1");
        
        ISkillNFT.Skill memory skill = skillNFT.getSkill(tokenId1);
        assertEq(skill.tokenId, tokenId1);
        assertEq(skill.creator, creator1);
        assertEq(skill.name, "Trading Bot");
        assertEq(skill.version, "v1.0");
        assertEq(skill.metadataURI, "ipfs://metadata1");
        assertTrue(skill.isTransferable);
        assertGt(skill.createdAt, 0);
    }
    
    function testMintSkillEmitsEvent() public {
        vm.prank(creator1);
        vm.expectEmit(true, true, false, true);
        emit SkillNFT.SkillMinted(0, creator1, "Trading Bot", "v1.0", 250);
        skillNFT.mintSkill("Trading Bot", "v1.0", "ipfs://metadata1", true);
    }
    
    function testMintSkillEmptyName() public {
        vm.prank(creator1);
        vm.expectRevert("SkillNFT: name required");
        skillNFT.mintSkill("", "v1.0", "ipfs://metadata1", true);
    }
    
    function testMintSkillEmptyVersion() public {
        vm.prank(creator1);
        vm.expectRevert("SkillNFT: version required");
        skillNFT.mintSkill("Trading Bot", "", "ipfs://metadata1", true);
    }
    
    function testMintSkillWhenPaused() public {
        vm.prank(owner);
        skillNFT.pause();
        
        vm.prank(creator1);
        vm.expectRevert();
        skillNFT.mintSkill("Trading Bot", "v1.0", "ipfs://metadata1", true);
    }
    
    function testMintSkillIncrementsCounter() public {
        vm.prank(creator1);
        tokenId1 = skillNFT.mintSkill("Skill 1", "v1.0", "ipfs://1", true);
        
        vm.prank(creator2);
        tokenId2 = skillNFT.mintSkill("Skill 2", "v1.0", "ipfs://2", true);
        
        assertEq(tokenId1, 0);
        assertEq(tokenId2, 1);
    }
    
    // ============ Batch Mint Tests ============
    
    function testBatchMintSkill() public {
        string[] memory names = new string[](3);
        names[0] = "Skill 1";
        names[1] = "Skill 2";
        names[2] = "Skill 3";
        
        string[] memory versions = new string[](3);
        versions[0] = "v1.0";
        versions[1] = "v1.0";
        versions[2] = "v1.0";
        
        string[] memory uris = new string[](3);
        uris[0] = "ipfs://1";
        uris[1] = "ipfs://2";
        uris[2] = "ipfs://3";
        
        bool[] memory transferable = new bool[](3);
        transferable[0] = true;
        transferable[1] = false;
        transferable[2] = true;
        
        vm.prank(creator1);
        uint256[] memory tokenIds = skillNFT.batchMintSkill(names, versions, uris, transferable);
        
        assertEq(tokenIds.length, 3);
        assertEq(tokenIds[0], 0);
        assertEq(tokenIds[1], 1);
        assertEq(tokenIds[2], 2);
        
        assertEq(skillNFT.ownerOf(0), creator1);
        assertEq(skillNFT.ownerOf(1), creator1);
        assertEq(skillNFT.ownerOf(2), creator1);
    }
    
    function testBatchMintSkillArrayLengthMismatch() public {
        string[] memory names = new string[](2);
        names[0] = "Skill 1";
        names[1] = "Skill 2";
        
        string[] memory versions = new string[](3);
        versions[0] = "v1.0";
        versions[1] = "v1.0";
        versions[2] = "v1.0";
        
        string[] memory uris = new string[](3);
        bool[] memory transferable = new bool[](3);
        
        vm.prank(creator1);
        vm.expectRevert("SkillNFT: array length mismatch");
        skillNFT.batchMintSkill(names, versions, uris, transferable);
    }
    
    function testBatchMintSkillEmptyArrays() public {
        string[] memory names = new string[](0);
        string[] memory versions = new string[](0);
        string[] memory uris = new string[](0);
        bool[] memory transferable = new bool[](0);
        
        vm.prank(creator1);
        vm.expectRevert("SkillNFT: empty arrays");
        skillNFT.batchMintSkill(names, versions, uris, transferable);
    }
    
    function testBatchMintSkillTooLarge() public {
        string[] memory names = new string[](51);
        string[] memory versions = new string[](51);
        string[] memory uris = new string[](51);
        bool[] memory transferable = new bool[](51);
        
        for (uint i = 0; i < 51; i++) {
            names[i] = "Skill";
            versions[i] = "v1.0";
            uris[i] = "ipfs://test";
            transferable[i] = true;
        }
        
        vm.prank(creator1);
        vm.expectRevert("SkillNFT: batch too large");
        skillNFT.batchMintSkill(names, versions, uris, transferable);
    }
    
    // ============ Transfer Tests ============
    
    function testTransferTransferable() public {
        vm.prank(creator1);
        tokenId1 = skillNFT.mintSkill("Trading Bot", "v1.0", "ipfs://metadata1", true);
        
        vm.prank(creator1);
        skillNFT.transferFrom(creator1, buyer, tokenId1);
        
        assertEq(skillNFT.ownerOf(tokenId1), buyer);
    }
    
    function testTransferNonTransferable() public {
        vm.prank(creator1);
        tokenId1 = skillNFT.mintSkill("Trading Bot", "v1.0", "ipfs://metadata1", false);
        
        vm.prank(creator1);
        vm.expectRevert("SkillNFT: skill not transferable");
        skillNFT.transferFrom(creator1, buyer, tokenId1);
    }
    
    function testSafeTransferTransferable() public {
        vm.prank(creator1);
        tokenId1 = skillNFT.mintSkill("Trading Bot", "v1.0", "ipfs://metadata1", true);
        
        vm.prank(creator1);
        skillNFT.safeTransferFrom(creator1, buyer, tokenId1);
        
        assertEq(skillNFT.ownerOf(tokenId1), buyer);
    }
    
    function testSafeTransferNonTransferable() public {
        vm.prank(creator1);
        tokenId1 = skillNFT.mintSkill("Trading Bot", "v1.0", "ipfs://metadata1", false);
        
        vm.prank(creator1);
        vm.expectRevert("SkillNFT: skill not transferable");
        skillNFT.safeTransferFrom(creator1, buyer, tokenId1);
    }
    
    function testTransferWhenPaused() public {
        vm.prank(creator1);
        tokenId1 = skillNFT.mintSkill("Trading Bot", "v1.0", "ipfs://metadata1", true);
        
        vm.prank(owner);
        skillNFT.pause();
        
        vm.prank(creator1);
        vm.expectRevert();
        skillNFT.transferFrom(creator1, buyer, tokenId1);
    }
    
    // ============ Royalty Tests ============
    
    function testSetTokenRoyalty() public {
        vm.prank(creator1);
        tokenId1 = skillNFT.mintSkill("Trading Bot", "v1.0", "ipfs://metadata1", true);
        
        vm.prank(creator1);
        skillNFT.setTokenRoyalty(tokenId1, 500); // 5%
        
        (address receiver, uint256 royaltyAmount) = skillNFT.royaltyInfo(tokenId1, 10000);
        assertEq(receiver, creator1);
        assertEq(royaltyAmount, 500);
    }
    
    function testSetTokenRoyaltyEmitsEvent() public {
        vm.prank(creator1);
        tokenId1 = skillNFT.mintSkill("Trading Bot", "v1.0", "ipfs://metadata1", true);
        
        vm.prank(creator1);
        vm.expectEmit(true, false, false, true);
        emit SkillNFT.RoyaltyUpdated(tokenId1, 500);
        skillNFT.setTokenRoyalty(tokenId1, 500);
    }
    
    function testSetTokenRoyaltyNotOwner() public {
        vm.prank(creator1);
        tokenId1 = skillNFT.mintSkill("Trading Bot", "v1.0", "ipfs://metadata1", true);
        
        vm.prank(creator2);
        vm.expectRevert("SkillNFT: not token owner");
        skillNFT.setTokenRoyalty(tokenId1, 500);
    }
    
    function testSetTokenRoyaltyTooHigh() public {
        vm.prank(creator1);
        tokenId1 = skillNFT.mintSkill("Trading Bot", "v1.0", "ipfs://metadata1", true);
        
        vm.prank(creator1);
        vm.expectRevert("SkillNFT: royalty too high");
        skillNFT.setTokenRoyalty(tokenId1, 1001); // > 10%
    }
    
    function testSetTokenRoyaltyNonExistent() public {
        vm.prank(creator1);
        vm.expectRevert("SkillNFT: token does not exist");
        skillNFT.setTokenRoyalty(999, 500);
    }
    
    function testResetTokenRoyalty() public {
        vm.prank(creator1);
        tokenId1 = skillNFT.mintSkill("Trading Bot", "v1.0", "ipfs://metadata1", true);
        
        vm.prank(creator1);
        skillNFT.setTokenRoyalty(tokenId1, 500);
        
        vm.prank(owner);
        skillNFT.resetTokenRoyalty(tokenId1);
        
        (address receiver, uint256 royaltyAmount) = skillNFT.royaltyInfo(tokenId1, 10000);
        assertEq(royaltyAmount, 250); // Back to default
    }
    
    function testResetTokenRoyaltyEmitsEvent() public {
        vm.prank(creator1);
        tokenId1 = skillNFT.mintSkill("Trading Bot", "v1.0", "ipfs://metadata1", true);
        
        vm.prank(owner);
        vm.expectEmit(true, false, false, false);
        emit SkillNFT.TokenRoyaltyReset(tokenId1);
        skillNFT.resetTokenRoyalty(tokenId1);
    }
    
    function testResetTokenRoyaltyNotOwner() public {
        vm.prank(creator1);
        tokenId1 = skillNFT.mintSkill("Trading Bot", "v1.0", "ipfs://metadata1", true);
        
        vm.prank(creator2);
        vm.expectRevert();
        skillNFT.resetTokenRoyalty(tokenId1);
    }
    
    function testSetDefaultRoyalty() public {
        address newReceiver = address(999);
        
        vm.prank(owner);
        skillNFT.setDefaultRoyalty(newReceiver, 300); // 3%
        
        (address receiver, uint256 royaltyAmount) = skillNFT.royaltyInfo(999, 10000);
        assertEq(receiver, newReceiver);
        assertEq(royaltyAmount, 300);
    }
    
    function testSetDefaultRoyaltyEmitsEvent() public {
        address newReceiver = address(999);
        
        vm.prank(owner);
        vm.expectEmit(true, false, false, true);
        emit SkillNFT.DefaultRoyaltySet(newReceiver, 300);
        skillNFT.setDefaultRoyalty(newReceiver, 300);
    }
    
    function testSetDefaultRoyaltyTooHigh() public {
        vm.prank(owner);
        vm.expectRevert("SkillNFT: royalty too high");
        skillNFT.setDefaultRoyalty(address(999), 1001);
    }
    
    function testOnlyOwnerCanSetDefaultRoyalty() public {
        vm.prank(creator1);
        vm.expectRevert();
        skillNFT.setDefaultRoyalty(address(999), 300);
    }
    
    // ============ Query Tests ============
    
    function testGetSkill() public {
        vm.prank(creator1);
        tokenId1 = skillNFT.mintSkill("Trading Bot", "v1.0", "ipfs://metadata1", true);
        
        ISkillNFT.Skill memory skill = skillNFT.getSkill(tokenId1);
        assertEq(skill.name, "Trading Bot");
        assertEq(skill.version, "v1.0");
        assertTrue(skill.isTransferable);
    }
    
    function testGetSkillNonExistent() public {
        vm.expectRevert("SkillNFT: token does not exist");
        skillNFT.getSkill(999);
    }
    
    function testGetCreator() public {
        vm.prank(creator1);
        tokenId1 = skillNFT.mintSkill("Trading Bot", "v1.0", "ipfs://metadata1", true);
        
        assertEq(skillNFT.getCreator(tokenId1), creator1);
    }
    
    function testGetCreatorNonExistent() public {
        vm.expectRevert("SkillNFT: token does not exist");
        skillNFT.getCreator(999);
    }
    
    function testIsTransferable() public {
        vm.prank(creator1);
        tokenId1 = skillNFT.mintSkill("Trading Bot", "v1.0", "ipfs://metadata1", true);
        
        assertTrue(skillNFT.isTransferable(tokenId1));
        
        vm.prank(creator2);
        tokenId2 = skillNFT.mintSkill("Another", "v1.0", "ipfs://metadata2", false);
        
        assertFalse(skillNFT.isTransferable(tokenId2));
    }
    
    function testIsTransferableNonExistent() public {
        vm.expectRevert("SkillNFT: token does not exist");
        skillNFT.isTransferable(999);
    }
    
    function testGetTokensByOwner() public {
        vm.prank(creator1);
        tokenId1 = skillNFT.mintSkill("Skill 1", "v1.0", "ipfs://1", true);
        
        vm.prank(creator1);
        tokenId2 = skillNFT.mintSkill("Skill 2", "v1.0", "ipfs://2", true);
        
        uint256[] memory tokens = skillNFT.getTokensByOwner(creator1);
        
        assertEq(tokens.length, 2);
        assertTrue(tokens[0] == tokenId1 || tokens[1] == tokenId1);
        assertTrue(tokens[0] == tokenId2 || tokens[1] == tokenId2);
    }
    
    function testTotalSupply() public {
        assertEq(skillNFT.totalSupply(), 0);
        
        vm.prank(creator1);
        skillNFT.mintSkill("Skill 1", "v1.0", "ipfs://1", true);
        
        assertEq(skillNFT.totalSupply(), 1);
        
        vm.prank(creator2);
        skillNFT.mintSkill("Skill 2", "v1.0", "ipfs://2", true);
        
        assertEq(skillNFT.totalSupply(), 2);
    }
    
    // ============ Pause Tests ============
    
    function testPause() public {
        vm.prank(owner);
        skillNFT.pause();
        
        assertTrue(skillNFT.paused());
    }
    
    function testUnpause() public {
        vm.prank(owner);
        skillNFT.pause();
        
        vm.prank(owner);
        skillNFT.unpause();
        
        assertFalse(skillNFT.paused());
    }
    
    function testOnlyOwnerCanPause() public {
        vm.prank(creator1);
        vm.expectRevert();
        skillNFT.pause();
    }
    
    // ============ Interface Tests ============
    
    function testSupportsInterfaceERC721() public view {
        assertTrue(skillNFT.supportsInterface(0x80ac58cd)); // ERC721
    }
    
    function testSupportsInterfaceERC2981() public view {
        assertTrue(skillNFT.supportsInterface(0x2a55205a)); // ERC2981
    }
    
    function testSupportsInterfaceERC165() public view {
        assertTrue(skillNFT.supportsInterface(0x01ffc9a7)); // ERC165
    }
    
    // ============ Upgrade Tests ============
    
    function testUpgradeAuthorization() public {
        SkillNFT newImplementation = new SkillNFT();
        
        vm.prank(owner);
        skillNFT.upgradeTo(address(newImplementation));
    }
    
    function testOnlyOwnerCanUpgrade() public {
        SkillNFT newImplementation = new SkillNFT();
        
        vm.prank(creator1);
        vm.expectRevert();
        skillNFT.upgradeTo(address(newImplementation));
    }
    
    // ============ Gas Measurement Tests ============
    
    function testGas_MintSkill() public {
        vm.prank(creator1);
        uint256 gasBefore = gasleft();
        skillNFT.mintSkill("Trading Bot", "v1.0", "ipfs://metadata1", true);
        uint256 gasUsed = gasBefore - gasleft();
        
        emit log_named_uint("Gas used for mintSkill", gasUsed);
    }
    
    function testGas_Transfer() public {
        vm.prank(creator1);
        tokenId1 = skillNFT.mintSkill("Trading Bot", "v1.0", "ipfs://metadata1", true);
        
        vm.prank(creator1);
        uint256 gasBefore = gasleft();
        skillNFT.transferFrom(creator1, buyer, tokenId1);
        uint256 gasUsed = gasBefore - gasleft();
        
        emit log_named_uint("Gas used for transfer", gasUsed);
    }
    
    function testGas_BatchMint() public {
        string[] memory names = new string[](10);
        string[] memory versions = new string[](10);
        string[] memory uris = new string[](10);
        bool[] memory transferable = new bool[](10);
        
        for (uint i = 0; i < 10; i++) {
            names[i] = "Skill";
            versions[i] = "v1.0";
            uris[i] = "ipfs://test";
            transferable[i] = true;
        }
        
        vm.prank(creator1);
        uint256 gasBefore = gasleft();
        skillNFT.batchMintSkill(names, versions, uris, transferable);
        uint256 gasUsed = gasBefore - gasleft();
        
        emit log_named_uint("Gas used for batchMint (10)", gasUsed);
    }
    
    // ============ Edge Cases ============
    
    function testManyMints() public {
        uint256 mintCount = 20;
        
        for (uint i = 0; i < mintCount; i++) {
            vm.prank(creator1);
            skillNFT.mintSkill(
                string(abi.encodePacked("Skill ", vm.toString(i))),
                "v1.0",
                string(abi.encodePacked("ipfs://", vm.toString(i))),
                true
            );
        }
        
        assertEq(skillNFT.totalSupply(), mintCount);
    }
    
    function testTransferAndQuery() public {
        vm.prank(creator1);
        tokenId1 = skillNFT.mintSkill("Trading Bot", "v1.0", "ipfs://metadata1", true);
        
        vm.prank(creator1);
        skillNFT.transferFrom(creator1, buyer, tokenId1);
        
        uint256[] memory buyerTokens = skillNFT.getTokensByOwner(buyer);
        assertEq(buyerTokens.length, 1);
        assertEq(buyerTokens[0], tokenId1);
        
        uint256[] memory creatorTokens = skillNFT.getTokensByOwner(creator1);
        assertEq(creatorTokens.length, 0);
    }
    
    function testFullLifecycle() public {
        // Mint
        vm.prank(creator1);
        tokenId1 = skillNFT.mintSkill("Trading Bot", "v1.0", "ipfs://metadata1", true);
        
        // Set custom royalty
        vm.prank(creator1);
        skillNFT.setTokenRoyalty(tokenId1, 500);
        
        // Transfer
        vm.prank(creator1);
        skillNFT.transferFrom(creator1, buyer, tokenId1);
        
        // Verify
        assertEq(skillNFT.ownerOf(tokenId1), buyer);
        (address receiver, uint256 amount) = skillNFT.royaltyInfo(tokenId1, 10000);
        assertEq(receiver, creator1);
        assertEq(amount, 500);
    }
}
