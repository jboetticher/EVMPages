use ethers::{utils, prelude::*};
use ethers_solc::Solc;
use std::{path::Path, sync::Arc, env};

type Client = SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider: Provider<Http> = Provider::<Http>::try_from("http://localhost:9933")?;

    let key: String = match env::var("PRIVATE_KEY") {
        Ok(v) => v.clone(),
        Err(e) => panic!("PRIVATE_KEY environment variable not found! {}", e)
    };

    println!("{}", key);

    let wallet: LocalWallet = key
        .parse::<LocalWallet>()?
        .with_chain_id(Chain::MoonbeamDev);
    let client = SignerMiddleware::new(provider.clone(), wallet.clone());

    Ok(())
}

// Print the balance of a wallet
async fn print_balances(provider: &Provider<Http>) -> Result<(), Box<dyn std::error::Error>> {
    let address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".parse::<Address>()?;
    let balance = provider.get_balance(address, None).await?;

    println!("{} has {}", address, balance);
    Ok(())
}


// Sends some native currency
async fn send_transaction(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let address_from = "0x6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b".parse::<Address>()?;
    let address_to = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".parse::<Address>()?;

    println!(
        "Beginning transfer of 1 native currency from {} to {}.",
        address_from, address_to
    );
    let tx = TransactionRequest::new()
        .to(address_to)
        .value(U256::from(utils::parse_ether(1)?))
        .from(address_from);
    let tx = client.send_transaction(tx, None).await?.await?;

    println!("Transaction Receipt: {}", serde_json::to_string(&tx)?);

    Ok(())
}
