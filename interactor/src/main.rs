use dotenvy::dotenv;
use ethers::{
    etherscan::Client,
    prelude::{verify::VerifyContract, *}, abi::AbiEncode,
};
use ethers_solc::Solc;
use inquire::Select;
use minify_html::{minify, Cfg};
use toml::Table;
use std::{
    env,
    fs::{read, File, read_to_string},
    io::{Write, self},
    path::{Path, PathBuf},
    sync::Arc, str::FromStr, fmt::write

};

mod selector;
use crate::selector::select_html;

type SignerClient = SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read private key
    dotenv().ok();
    let key: String = match env::var("PRIVATE_KEY") {
        Ok(v) => v.clone(),
        Err(e) => panic!("PRIVATE_KEY environment variable not found! {}", e),
    };

    // Read config file
    let dir: &Path = Path::new(&env!("CARGO_MANIFEST_DIR"));
    let config_path = dir.join("config.toml");
    let config = read_to_string(config_path.clone())?.parse::<Table>().unwrap();

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
            let r = select_html(dir.clone())?;
            let data = minify_html(r)?;

            let evm_pages = match config["pages"].as_str() {
                Some(x) => Ok(x),
                None => Err("EVMPages deployment configured incorrectly!")
            };
            let address_to = evm_pages?.parse::<H160>()?;

            // Send transaction
            let tx = TransactionRequest::new()
                .to(address_to)
                .from(client.address())
                .data(data)
                .chain_id(1287);


            // Declare a page
            let tx = client.send_transaction(tx, None).await?.await?;
            match tx {
                Some(t) => {
                    // get address from config
                    // construct the contract
                    // write t.transaction_hash in the data
                },
                None => {
                    println!("There was an error with sending the initial declaration!");
                }
            }
        }
        1 => {}
        2 => {}
        // Deploy the smart contract
        3 => {
            let p = Path::new(&env!("CARGO_MANIFEST_DIR")).parent();
            let p: &Path = &p.unwrap().join("contracts");
            println!("Searching for contracts at {:?}", p);
            compile_deploy_contract(&client, p, &config_path, &dir.clone().join("abi.json")).await?;
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

abigen!(
    EVMPages,
    "./abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

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

// Minify HTML
fn minify_html(r: PathBuf) -> io::Result<Vec<u8>> {
    // Build minimum file name
    let mut min_file_name = r.file_stem().unwrap().to_str().unwrap().to_owned();
    min_file_name.push_str(".min.html");
    let min_file_name = r.parent().unwrap().join(min_file_name);

    // Read the code of the file
    let code = read(r)?;

    // Minify
    let mut cfg = Cfg::new();
    cfg.keep_comments = false;
    cfg.minify_css = true;
    cfg.minify_js = true;
    let minified = minify(&code, &cfg);

    // Write to new file
    let mut min_file = File::create(min_file_name).unwrap();
    min_file.write(&minified)?;

    return Ok(minified);
}

async fn compile_deploy_contract(
    client: &SignerClient,
    source: &Path,
    config_path: &PathBuf,
    write_path: &PathBuf
) -> Result<H160, Box<dyn std::error::Error>> {
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

    let addr = H160::from_str("0x8928cb8cff09682a87275a770879df568dd00c2d")?;

    // Edit the TOML file
    let mut config = read_to_string(config_path.clone())?.parse::<Table>().unwrap();
    config.insert("pages".to_owned(), toml::Value::String(format!("{:?}", addr)));
    write!(&mut File::create(config_path).unwrap(), "{}", toml::to_string(&config)?)?;
    
    
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
