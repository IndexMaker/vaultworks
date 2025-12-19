use clap::Parser;
use deli::log_msg;
use ethers::types::Address;

use decon::tx_sender::TxClient;
use eyre::{eyre, OptionExt};

mod scenario_1;
mod scenario_2;
mod scenario_3;
mod scenario_4;
mod scenario_5;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    rpc_url: Option<String>,

    #[arg(short, long)]
    private_key: String,

    #[arg(long)]
    castle_address: Option<String>,

    #[arg(long)]
    clerk_address: Option<String>,

    #[arg(short, long, value_delimiter = ',')]
    scenario: Vec<String>,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let cli = Cli::parse();
    let rpc_url = cli.rpc_url.unwrap_or("http://localhost:8547".to_owned());
    let get_private_key = || -> String { cli.private_key.clone() };

    let castle_address: Option<Address> = if let Some(a) = cli.castle_address {
        Some(a.parse()?)
    } else {
        None
    };

    let clerk_address: Option<Address> = if let Some(a) = cli.clerk_address {
        Some(a.parse()?)
    } else {
        None
    };

    let scenario = cli.scenario;

    let client = TxClient::try_new_from_url(&rpc_url, get_private_key).await?;

    for s in scenario {
        match s.as_str() {
            "scenario1" => {
                scenario_1::run_scenario(
                    &client,
                    clerk_address.ok_or_eyre("Clerk address is required")?,
                )
                .await?;
            }
            "scenario2" => {
                scenario_2::run_scenario(
                    &client,
                    clerk_address.ok_or_eyre("Clerk address is required")?,
                )
                .await?;
            }
            "scenario3" => {
                scenario_3::run_scenario(
                    &client,
                    clerk_address.ok_or_eyre("Clerk address is required")?,
                )
                .await?;
            }
            "scenario4" => {
                scenario_4::run_scenario(
                    &client,
                    clerk_address.ok_or_eyre("Clerk address is required")?,
                )
                .await?;
            }
            "scenario5" => {
                scenario_5::run_scenario(
                    &client,
                    castle_address.ok_or_eyre("Castle address is required")?,
                )
                .await?;
            }
            x => {
                Err(eyre!("No such scenario: {}", x))?;
            }
        }
    }

    log_msg!("Done.");
    Ok(())
}
