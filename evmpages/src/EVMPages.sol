// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

contract EVMPages {

    string public constant INSTRUCTIONS = "This is where the instructions will go, so that the ability to read websites will never go away.";

    event MainTemplateUpdated(bytes32);
    event PotentialNewPageCreated(address);
    event NewPageDeclared(address, uint256, bytes32);
    event MainPageSet(address, uint256, bytes32);

    error PageDoesNotExist();

    // Map each address to an "array" of transaction deployments.
    // This makes it possible to find each address's page.
    mapping(address => mapping(uint256 => bytes32)) public pages;

    // The number of pages that a person has declared.
    mapping(address => uint256) public pagesDeclared;

    // An address's main page, so that an address can map directly to a page.
    mapping(address => uint256) public mainPage;

    // Allows an address to set their own "main page"
    function setMainPage(uint256 pageId) external {
        address sender = msg.sender;
        if(pageId >= pagesDeclared[sender]) revert PageDoesNotExist();
        mainPage[sender] = pageId;

        emit MainPageSet(sender, pageId, pages[msg.sender][pageId]);
    }

    // Declares a page in the smart contract's registry
    function declarePage(bytes32 txHash) external returns(uint256) {
        address sender = msg.sender;
        uint256 nextIndex = pagesDeclared[sender];

        // Set the page's txHash
        pages[sender][nextIndex] = txHash;

        // Increase the number of declared pages
        pagesDeclared[sender] = nextIndex + 1;

        emit NewPageDeclared(sender, nextIndex, txHash);
        return nextIndex;
    }

    fallback() external {
        emit PotentialNewPageCreated(msg.sender);
    }
}
