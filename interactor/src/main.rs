use ethers::{utils, prelude::*};
use ethers_solc::Solc;
use std::{env};

type Client = SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider: Provider<Http> = Provider::<Http>::try_from("https://rpc.api.moonbase.moonbeam.network")?;

    // let key: String = match env::var("PRIVATE_KEY") {
    //     Ok(v) => v.clone(),
    //     Err(e) => panic!("PRIVATE_KEY environment variable not found! {}", e)
    // };
    let key = "";

    let wallet: LocalWallet = key
        .parse::<LocalWallet>()?
        .with_chain_id(Chain::Moonbase);
    let client = SignerMiddleware::new(provider.clone(), wallet.clone());

    send_transaction(&client).await?;

    Ok(())
}


async fn send_transaction(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    // This is the address of the contract
    let address_to = "0x39b165A3141832198cFCba12Eb86471C53Caa6ab".parse::<Address>()?;
    let data = "0x3C646976207374796C653D2277696474683A313030253B6865696768743A313030253B6261636B67726F756E642D636F6C6F723A677265656E3B636F6C6F723A77686974653B223E546869732069732061206D61696E2070616765213C2F6469763E".parse::<Bytes>()?;

    println!("data: {}", data);

    let tx = TransactionRequest::new()
        .to(address_to)
        .from(client.address())
        .data(data)
        .chain_id(1287);

    let tx = client.send_transaction(tx, None).await?.await?;

    println!("Transaction Receipt: {}", serde_json::to_string(&tx)?);

    Ok(())
}

// Print the balance of a wallet
async fn print_balances(provider: &Provider<Http>) -> Result<(), Box<dyn std::error::Error>> {
    let address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".parse::<Address>()?;
    let balance = provider.get_balance(address, None).await?;

    println!("{} has {}", address, balance);
    Ok(())
}