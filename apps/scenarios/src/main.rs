use std::env;

use clap::Parser;
use deli::log_msg;
use ethers::types::Address;

use decon::{contracts::Granary, tx_sender::TxClient};
use eyre::eyre;

mod scenario_1;
mod scenario_2;
mod scenario_3;
mod scenario_4;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    rpc_url: Option<String>,

    #[arg(long)]
    granary_address: String,

    #[arg(short, long, value_delimiter = ',')]
    scenario: Vec<String>,
}

fn get_private_key() -> String {
    env::var("AP_PRIVATE_KEY").expect("AP_PRIVATE_KEY not found in environment")
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let cli = Cli::parse();
    let rpc_url = cli.rpc_url.unwrap_or("http://localhost:8547".to_owned());
    let granary_address: Address = cli.granary_address.parse()?;
    let scenario = cli.scenario;

    let client = TxClient::try_new_from_url(&rpc_url, get_private_key).await?;

    let granary = Granary::new(granary_address, client.client());

    for s in scenario {
        match s.as_str() {
            "scenario1" => {
                scenario_1::run_scenario(&client, granary_address).await?;
            }
            "scenario2" => {
                scenario_2::run_scenario(&client, granary_address).await?;
            }
            "scenario3" => {
                scenario_3::run_scenario(&client, granary_address).await?;
            }
            "scenario4" => {
                scenario_4::run_scenario(&client, granary_address).await?;
            }
            x => {
                Err(eyre!("No such scenario: {}", x))?;
            }
        }
    }

    log_msg!("Done.");
    Ok(())
}
