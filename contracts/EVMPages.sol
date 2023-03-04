// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

contract EVMPages {

    string public constant INSTRUCTIONS = "Convert the calldata of the transaction with the hash of the LANDING variable into a utf-8 HTML file.";
    bytes32 public LANDING;

    event MainTemplateUpdated(bytes32);
    event PotentialNewPageCreated(address);
    event NewPageDeclared(address, uint256, bytes32);
    event MainPageSet(address, uint256, bytes32);

    error PageDoesNotExist();
    error OnlyOwnerAllowed();
    error DomainNotOwned();
    error PaymentNotEnough();

    // Map each address to an "array" of transaction deployments.
    // This makes it possible to find each address's page.
    mapping(address => mapping(uint256 => bytes32)) public pages;

    // A series of named packages to retrieve data from
    mapping(string => bytes32) public packages;

    // The number of pages that a person has declared.
    mapping(address => uint256) public pagesDeclared;

    // An address's main page, so that an address can map directly to a page.
    mapping(address => uint256) public mainPage;

    // A domain registration, so that text can map directly to a page.
    mapping(string => bytes32) public domains;

    // Owners of each domain registration
    mapping(string => address) public domainOwners;

    // The owner/maintainer role of the contract
    address public owner;

    // How much someone has to pay to register a domain name
    uint256 domainRegistrationFee = 1 ether;

    constructor() {
        owner = msg.sender;
    }

    fallback() external {
        // You can send sites to this contract directly if you want
        emit PotentialNewPageCreated(msg.sender);
    }

    // Only allows the owner to access the function
    modifier onlyOwner() {
        if(msg.sender != owner) revert OnlyOwnerAllowed();
        _;
    }

    modifier onlyDomainOwner(string calldata d) {
        if(msg.sender != domainOwners[d]) revert OnlyOwnerAllowed();
        _;
    }

    // Sets the landing page for everyone to use.
    function setLanding(bytes32 txHash) external onlyOwner {
        LANDING = txHash;
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

    // Declares a package in the smart contract's registry
    function declarePackage(bytes32 txHash, string calldata name) external onlyOwner {
        packages[name] = txHash;
    }

    // Declares a domain name for a price
    function declareDomain(string calldata domain) external payable {
        if(domainOwners[domain] != address(0)) revert DomainNotOwned();
        if(msg.sender != owner && msg.value < domainRegistrationFee) revert PaymentNotEnough();
        domainOwners[domain] = msg.sender;
    }

    // Allows a user to transfer ownership of a domain to another user
    function transferDomain(string calldata domain, address recipient) onlyDomainOwner(domain) external {
        domainOwners[domain] = recipient;
    }

    // Allows a user to set a txHash to a domain they own
    function setDomain(string calldata domain, bytes32 txHash) onlyDomainOwner(domain) external {
        domains[domain] = txHash;
    }

    // Allows an address to set their own "main page"
    function setMainPage(uint256 pageId) external {
        address sender = msg.sender;
        if(pageId >= pagesDeclared[sender]) revert PageDoesNotExist();
        mainPage[sender] = pageId;

        emit MainPageSet(sender, pageId, pages[msg.sender][pageId]);
    }

    // Returns the hash of a user's main page, instead of the int
    function getMainPageHash(address addr) view external returns(bytes32) {
        return pages[addr][mainPage[addr]];
    }

    // Get the cash money
    function withdraw() external onlyOwner {
        payable(owner).transfer(address(this).balance);
    }
}
