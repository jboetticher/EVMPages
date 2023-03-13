use dotenvy::dotenv;
use ethers::prelude::*;
use ethers_solc::Solc;
use helpers::publish_html;
use inquire::{CustomType, Select, Text};
use std::{
    env,
    fs::{read_to_string, File},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};
use toml::Table;

mod helpers;
mod selector;
use crate::{
    helpers::{get_addr_in_config, minify_html, SignerClient},
    selector::select_html,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Read private key
    dotenv().ok();
    let key: String = match env::var("PRIVATE_KEY") {
        Ok(v) => v.clone(),
        Err(e) => panic!("PRIVATE_KEY environment variable not found! {}", e),
    };

    // Read config file
    let dir: &Path = Path::new(&env!("CARGO_MANIFEST_DIR"));
    let config_path = dir.join("config.toml");
    let config: Table = read_to_string(config_path.clone())?.parse::<Table>()?;

    // Construct provider & signer
    let provider = Provider::<Http>::try_from(config["rpc"].as_str().unwrap())?;
    let wallet: LocalWallet = key.parse::<LocalWallet>()?.with_chain_id(Chain::Moonbase);
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
            "Compile and deploy contracts",
        ]
        .to_vec(),
    );

    let x = selection_prompt.raw_prompt()?.index;
    match x {
        // Publish a page
        0 => {
            // Publish the HTML/JS & construct the contract object
            let (contract, tx) = publish_and_construct_contract(dir, &config, &client).await?;

            // write t.transaction_hash in the data
            let declared = contract.pages_declared(client.address()).call().await?;
            println!("Declaring page...");
            contract
                .declare_page(tx.transaction_hash.to_fixed_bytes())
                .send()
                .await?
                .await?;
            println!(
                "New page declared for address {} with ID {} with transaction hash {}",
                client.address(),
                declared,
                tx.transaction_hash
            );
        }
        // Publish a package
        1 => {
            // Publish the HTML/JS & construct the contract object
            let (contract, tx) = publish_and_construct_contract(dir, &config, &client).await?;

            // get package name
            let text_prompt = Text::new("Provide a name for the package:");
            let package_name = text_prompt.prompt()?;

            // write t.transaction_hash in the data
            println!("Declaring package...");
            contract
                .declare_package(tx.transaction_hash.to_fixed_bytes(), package_name.clone())
                .send()
                .await?
                .await?;
            println!(
                "New package declared with name {} and transaction hash {}",
                package_name, tx.transaction_hash
            );
        }
        // Set a main page
        2 => {
            // Gets the number of pages that a user has declared
            let addr = get_addr_in_config(&config, "pages")?;
            let contract: EVMPagesSigner = EVMPages::new(addr.clone(), Arc::new(client.clone()));
            let num_declared: u128 = contract
                .pages_declared(client.address())
                .call()
                .await?
                .as_u128();
            println!("You have {} pages declared.", num_declared);

            if num_declared > 0 {
                let mut selection: u128 = u128::MAX;
                while selection == u128::MAX {
                    selection = CustomType::new("Provide a value for the main page:")
                        .with_formatter(&|i: u128| format!("#{i}"))
                        .with_error_message("Please type a valid number")
                        .prompt()
                        .unwrap();
                    if selection >= num_declared {
                        println!("Please type a number between 0 and {}", num_declared - 1);
                        selection = u128::MAX;
                    }
                }

                contract
                    .set_main_page(U256::from(selection))
                    .send()
                    .await?
                    .await?;
                println!("Main page successfully set.");
            } else {
                println!(
                    "This address {} hasn't declared any pages yet, so a main page cannot be set.",
                    client.address()
                );
            }
        }
        // Deploy the smart contract
        3 => {
            let p = Path::new(&env!("CARGO_MANIFEST_DIR")).parent();
            let p: &Path = &p.unwrap().join("contracts");
            println!("Searching for contracts at {:?}", p);
            compile_deploy_contract(&client, p, &config_path, &dir.clone().join("abi.json"))
                .await?;
        }
        _ => {
            println!("Error! Selection not valid!");
        }
    }

    Ok(())
}

async fn publish_and_construct_contract(
    dir: &Path,
    config: &Table,
    client: &SignerClient,
) -> Result<(EVMPagesSigner, TransactionReceipt), anyhow::Error> {
    let r = select_html(dir.clone())?;
    let data = minify_html(r)?;
    let address_to = get_addr_in_config(config, "pages")?;
    let tx = publish_html(client.clone(), address_to, data).await?;

    println!("Page stored: {:?}", format!("{:?}", tx.transaction_hash));

    // get address from config
    let addr = match config["pages"].as_str() {
        Some(a) => a.parse::<H160>(),
        None => panic!("'pages' address not set!"),
    }?;

    // construct the contract
    Ok((EVMPages::new(addr.clone(), Arc::new(client.clone())), tx))
}

// Compile, deploy, and write down the contract
async fn compile_deploy_contract(
    client: &SignerClient,
    source: &Path,
    config_path: &PathBuf,
    write_path: &PathBuf,
) -> Result<H160, anyhow::Error> {
    let compiled = Solc::default()
        .compile_source(source)
        .expect("Could not compile contracts");

    let (abi, bytecode, _runtime_bytecode) = compiled
        .find("EVMPages")
        .expect("could not find contract")
        .into_parts_or_default();

    // Write to a local file
    println!("Writing EVMPages to a new ABI...");
    serde_json::to_writer(&File::create(write_path)?, &abi)?;

    let factory = ContractFactory::new(abi, bytecode, Arc::new(client.clone()));

    println!("Deploying EVMPages...");
    let contract = factory.deploy(())?.send().await?;

    let addr = contract.address();
    println!("EVMPages.sol has been deployed to {:?}", addr);

    // Edit the TOML file
    let mut config = read_to_string(config_path.clone())?
        .parse::<Table>()
        .unwrap();
    config.insert(
        "pages".to_owned(),
        toml::Value::String(format!("{:?}", addr)),
    );
    write!(
        &mut File::create(config_path).unwrap(),
        "{}",
        toml::to_string(&config)?
    )?;

    // // Etherscan client
    // let key: String = match env::var("ETHERSCAN_KEY") {
    //     Ok(v) => v.clone(),
    //     Err(e) => panic!("ETHERSCAN_KEY environment variable not found! {}", e),
    // };
    // let etherscan = Client::new(Chain::Moonbase, key)?;

    // // Verify
    // let v = VerifyContract::new(
    //     H160::from_str("0xD158A7B3e9Ef167DF1fc250c014Ae2A0D9acE949"),
    //     "EVMPages".to_string(),
    //     "".to_string(),
    //     "0.8.18".to_string(),
    // )
    // .constructor_arguments(Some(""))
    // .optimization(true)
    // .runs(200);
    // etherscan
    //     .submit_contract_verification(&v)
    //     .await
    //     .expect("failed to send the request");

    // TODO: set landing

    println!("You should rebuild if any changes to the solidity file was made.");

    Ok(addr)
}

abigen!(
    EVMPages,
    "./abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

type EVMPagesSigner = EVMPages<SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>>;
