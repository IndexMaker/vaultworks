use amount_macros::amount;
use deli::{labels::Labels, log_msg, vector::Vector};
use devil_macros::devil;
use ethers::types::Address;

use decon::{contracts::Devil, tx_sender::TxClient};

pub async fn run_scenario(client: &TxClient, devil_address: Address) -> eyre::Result<()> {
    log_msg!("Scenario 3.");
    let devil = Devil::new(devil_address, client.client());
    Ok(())
}
