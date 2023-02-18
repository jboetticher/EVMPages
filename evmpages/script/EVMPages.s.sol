// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Script.sol";
import "../src/EVMPages.sol";

contract EVMPagesDeployScript is Script {
    function setUp() public {}

    function run() public {
        vm.broadcast(deployerPrivateKey);

        new EVMPages();

        vm.stopBroadcast();
    }
}
