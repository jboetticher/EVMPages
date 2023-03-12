use ethers::prelude::*;
use minify_html::{minify, Cfg};
use std::{
    fs::{read, File},
    io::{self, Write},
    path::PathBuf,
};
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

pub async fn publish_html(
    client: SignerClient,
    contract_addr: H160,
    data: Vec<u8>,
) -> Result<TransactionReceipt, anyhow::Error> {
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
    } else {
        panic!("There was an error with sending the initial declaration!");
    }
}

// Minify HTML
pub fn minify_html(r: PathBuf) -> io::Result<Vec<u8>> {
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
