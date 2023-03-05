use ethers::{prelude::{*, verify::VerifyContract}, etherscan::Client};
use ethers_solc::Solc;
use inquire::Select;
use std::{env, path::Path, sync::Arc};
use dotenvy::dotenv;

type SignerClient = SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let key: String = match env::var("PRIVATE_KEY") {
        Ok(v) => v.clone(),
        Err(e) => panic!("PRIVATE_KEY environment variable not found! {}", e)
    };
    let provider = Provider::<Http>::try_from("https://rpc.api.moonbase.moonbeam.network")?;
    let wallet: LocalWallet = key
        .parse::<LocalWallet>()?
        .with_chain_id(Chain::Moonbase);
    let client: SignerClient = SignerMiddleware::new(provider.clone(), wallet.clone());
    

    /*
    1. Publish pages
    2. Publish packages
    3. Set main page
    4. Deploy & verify EVMPages scripts
    */
    let selection_prompt = Select::new(
        "What would you like to do?",
        [
            "Publish a page",
            "Publish a package",
            "Set main page",
            "Compile, deploy, and verify contracts",
        ]
        .to_vec(),
    );

    let x = selection_prompt.raw_prompt()?.index;
    match x {
        0 => {}
        1 => {}
        2 => {}
        3 => {
            let p = Path::new(&env!("CARGO_MANIFEST_DIR")).parent();
            let p: &Path = &p.unwrap().join("contracts");
            println!("Searching for contracts at {:?}", p);
            compile_deploy_contract(&client, p).await?;
        }
        _ => {
            println!("Error! Selection not valid!");
        }
    }

    // let wallet: LocalWallet = key
    //     .parse::<LocalWallet>()?
    //     .with_chain_id(Chain::Moonbase);
    // let client = SignerMiddleware::new(provider.clone(), wallet.clone());

    // send_transaction(&client).await?;

    Ok(())
}

async fn send_transaction(client: &SignerClient) -> Result<(), Box<dyn std::error::Error>> {
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

async fn compile_deploy_contract(
    client: &SignerClient,
    source: &Path,
) -> Result<H160, Box<dyn std::error::Error>> {
    let compiled = Solc::default()
        .compile_source(source)
        .expect("Could not compile contracts");

    let (abi, bytecode, _runtime_bytecode) = compiled
        .find("EVMPages")
        .expect("could not find contract")
        .into_parts_or_default();

    let factory = ContractFactory::new(abi, bytecode, Arc::new(client.clone()));

    println!("Deploying EVMPages...");
    let contract = factory.deploy(())?.send().await?;

    let addr = contract.address();
    println!("EVMPages.sol has been deployed to {:?}", addr);

    // Etherscan client
    // let key: String = match env::var("ETHERSCAN_KEY") {
    //     Ok(v) => v.clone(),
    //     Err(e) => panic!("ETHERSCAN_KEY environment variable not found! {}", e)
    // };
    // let etherscan = Client::new(Chain::Moonbase, key)?;

    // Verify
    // VerifyContract::new(addr, "EVMPages".to_string(), "".to_string(), "0.8.18".to_string());

    Ok(addr)
}
