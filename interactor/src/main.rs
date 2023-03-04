use ethers::{utils, prelude::*};
use ethers_solc::Solc;
use std::{env, path::Path};
use inquire::Select;

type Client = SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider: Provider<Http> = Provider::<Http>::try_from("https://rpc.api.moonbase.moonbeam.network")?;

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
            "Compile, deploy, and verify contracts"
        ].to_vec()
    );

    match selection_prompt.raw_prompt()?.index {
        0 => {

        },
        1 => {},
        2 => {},
        3 => {
            let source = Path::new(&env!("CARGO_MANIFEST_DIR")). .join("interactor");
            println!("{:?}", source);
        },
        _ => {
            println!("Error! Selection not valid!");
        }
    }

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


// 1. Define an asynchronous function that takes a client provider as input and returns H160
// async fn compile_deploy_contract(client: &Client) -> Result<H160, Box<dyn std::error::Error>> {
//     // 2. Define a path as the directory that hosts the smart contracts in the project
//     let source = Path::new(&env!("CARGO_MANIFEST_DIR")).join("interactor");
//     println!("{:?}", source);

//     // 3. Compile all of the smart contracts
//     let compiled = Solc::default()
//         .compile_source(source)
//         .expect("Could not compile contracts");

//     // 4. Get ABI & Bytecode for Incrementer.sol
//     let (abi, bytecode, _runtime_bytecode) = compiled
//         .find("Incrementer")
//         .expect("could not find contract")
//         .into_parts_or_default();

//     // 5. Create a contract factory which will be used to deploy instances of the contract
//     let factory = ContractFactory::new(abi, bytecode, Arc::new(client.clone()));

//     // 6. Deploy
//     let contract = factory.deploy(U256::from(5))?.send().await?;

//     // 7. Print out the address
//     let addr = contract.address();
//     println!("Incrementer.sol has been deployed to {:?}", addr);

//     // 8. Return the address
//     Ok(addr)
// }
