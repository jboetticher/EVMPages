# PagesEVM

Etherscan proves to be an extremely expensive tool for EVM protocols to pay for, so let's abuse it a little. Since Etherscan keeps a full historical node of all transactions, all calldata from all transactions are readily available. Since any calldata can be sent to any contract (no function signature needed), you can store an entire website (HTML + JavaScript) inside a function's calldata.  

## Features

- Upload pages via calldata 
- Rust-based CLI that allows you to deploy pages
- Simple explorer for pages (/template/index.html)
- In theory, packages. You can query for multiple pages at a time, and load them in as `<script>` tags.

**TODO (PRs welcome):**
- Functioning demo of a page querying packages
- Functioning dynamic rendering

## Startup

```
source .env
cd interactor && cargo run
```