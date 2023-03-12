use ethers::{prelude::*, abi::Error};
use toml::Table;

pub type SignerClient = SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>;

pub fn get_addr_in_config(config: &Table, config_name: &str) -> Result<H160, anyhow::Error> {
    if let Some(x) = config[config_name].as_str() {
        match x.parse::<H160>() {
            Ok(x) => Ok(x),
            _ => panic!("Error parsing H160 into hex form!"),
        }
    } else {
        panic!("Entry doesn't exist!")
    }
}

pub async fn publish_html(client: SignerClient, contract_addr: H160, data: Vec<u8>) -> Result<TransactionReceipt, anyhow::Error> {
    // Send transaction
    let tx = TransactionRequest::new()
        .to(contract_addr)
        .from(client.address())
        .data(data)
        .chain_id(1287);

    // Declare a page
    println!("Storing page in a transaction...");
    if let Some(x) = client.send_transaction(tx, None).await?.await? {
        Ok(x)
    }
    else {
        panic!("There was an error with sending the initial declaration!");
    }
}
