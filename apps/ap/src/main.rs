use amount_macros::amount;
use clap::Parser;
use deli::{labels::Labels, log_msg, vector::Vector};
use devil_macros::devil;
use ethers::{
    middleware::SignerMiddleware,
    prelude::abigen,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::Address,
};
use std::sync::Arc;
use std::{env, str::FromStr};
use vector_macros::amount_vec;

use crate::tx_sender::TxSendBuilder;

pub mod tx_sender;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    rpc_url: Option<String>,

    #[arg(long)]
    devil_address: String,
}

fn get_private_key() -> String {
    env::var("AP_PRIVATE_KEY").expect("AP_PRIVATE_KEY not found in environment")
}

macro_rules! devil_bytes {
    ($($vil_instruction:tt)*) => {{
        Labels::from_vec_u128(devil! { $($vil_instruction)* }).to_vec()
    }};
}

abigen!(
    Devil,
    r"[
        function setup(address owner) external
        function submit(uint128 id, uint8[] memory data) external
        function get(uint128 id) external view returns (uint8[] memory)
        function execute(uint8[] memory code, uint128 num_registry) external
    ]"
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let cli = Cli::parse();
    let rpc_url = cli.rpc_url.unwrap_or("http://localhost:8547".to_owned());
    let devil_address: Address = cli.devil_address.parse()?;

    let client = {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        let priv_key = get_private_key();
        let wallet = LocalWallet::from_str(&priv_key)?;
        let chain_id = provider.get_chainid().await?.as_u64();
        Arc::new(SignerMiddleware::new(
            provider,
            wallet.clone().with_chain_id(chain_id),
        ))
    };

    let devil = Devil::new(devil_address, client.clone());

    let asset_prices_id = 101;
    let asset_slopes_id = 102;
    let asset_weights_id = 103;
    let index_quote_id = 104;
    let asset_prices = Vector::from_vec_u128(vec![amount!(2.5).to_u128_raw(); 100]); //amount_vec![1.5, 2.5];
    let asset_slopes = Vector::from_vec_u128(vec![amount!(0.0625).to_u128_raw(); 100]); //amount_vec![0.25, 0.0625];
    let asset_weights = Vector::from_vec_u128(vec![amount!(4.0).to_u128_raw(); 100]); //amount_vec![2.0, 4.0];

    // log_msg!("Setting up...");

    // devil
    //     .setup(client.address())
    //     .send()
    //     .await
    //     .expect("Failed to send setup")
    //     .await
    //     .expect("Failed to obtain setup receipt");

    log_msg!("Sending index parameters...");

    TxSendBuilder::new(client.clone())
        .add(devil.submit(asset_prices_id, asset_prices.to_vec()))
        .add(devil.submit(asset_slopes_id, asset_slopes.to_vec()))
        .add(devil.submit(asset_weights_id, asset_weights.to_vec()))
        .send()
        .await?;

    TxSendBuilder::new(client.clone())
        .add(devil.execute(
            devil_bytes![
                LDV asset_weights_id
                LDV asset_prices_id
                MUL 1
                VSUM
                LDV asset_slopes_id
                SWAP 2
                MUL 0
                MUL 2
                VSUM
                PKV 2
                STV index_quote_id
            ],
            16,
        ))
        .send()
        .await?;

    log_msg!("Getting index quote...");

    let _index_quote = Vector::from_vec(
        devil
            .get(index_quote_id)
            .call()
            .await
            .expect("Failed to get index quote"),
    );
    log_msg!("Index Quote: {:0.9}", _index_quote);

    log_msg!("Done.");
    Ok(())
}
