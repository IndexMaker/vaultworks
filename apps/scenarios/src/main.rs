use clap::Parser;
use common::log_msg;

use common_ethers::tx_sender::TxClient;
use ethers::types::Address;
use eyre::{eyre, OptionExt};

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
    keeper_address: Option<String>,

    #[arg(long)]
    collateral_custody: Option<String>,

    #[arg(long)]
    collateral_asset: Option<String>,

    #[arg(short, long, value_delimiter = ',')]
    scenario: Vec<String>,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let cli = Cli::parse();
    let rpc_url = cli.rpc_url.unwrap_or("http://localhost:8547".to_owned());
    let get_private_key = || -> String { cli.private_key.clone() };

    let scenario = cli.scenario;

    let client = TxClient::try_new_from_url(&rpc_url, get_private_key).await?;

    let castle_address = cli.castle_address.map(|x| x.parse::<Address>());
    let keeper_address = cli.keeper_address.map(|x| x.parse::<Address>());
    let collateral_custody = cli.collateral_custody.map(|x| x.parse::<Address>());
    let collateral_asset = cli.collateral_asset.map(|x| x.parse::<Address>());

    for s in scenario {
        match s.as_str() {
            "scenario5" => {
                scenario_5::run_scenario(
                    &client,
                    castle_address.ok_or_eyre("Castle address is required")??,
                    keeper_address.ok_or_eyre("Keeper address is required")??,
                    collateral_custody.ok_or_eyre("Collateral Custody address is required")??,
                    collateral_asset.ok_or_eyre("Collateral Asset address is required")??,
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
