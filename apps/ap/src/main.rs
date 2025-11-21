use std::env;

use clap::Parser;
use deli::log_msg;
use ethers::types::Address;

use decon::{contracts::Devil, tx_sender::TxClient};

mod scenario_1;
mod scenario_2;
mod scenario_3;

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

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let cli = Cli::parse();
    let rpc_url = cli.rpc_url.unwrap_or("http://localhost:8547".to_owned());
    let devil_address: Address = cli.devil_address.parse()?;

    let client = TxClient::try_new_from_url(&rpc_url, get_private_key).await?;
    
    let devil = Devil::new(devil_address, client.client());

    log_msg!("Setting up...");

    devil
        .setup(client.address())
        .send()
        .await
        .expect("Failed to send setup")
        .await
        .expect("Failed to obtain setup receipt");

    scenario_1::run_scenario(&client, devil_address).await?;
    scenario_2::run_scenario(&client, devil_address).await?;
    scenario_3::run_scenario(&client, devil_address).await?;

    log_msg!("Done.");
    Ok(())
}
