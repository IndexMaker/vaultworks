use abacus_macros::abacus;
use amount_macros::amount;
use common::{log_msg, vector::Vector};
use ethers::types::Address;

use common_ethers::{ToBytes, contracts::{Abacus, Clerk}, tx_sender::TxClient};

pub async fn run_scenario(client: &TxClient, abacus_address: Address) -> eyre::Result<()> {
    log_msg!("Scenario 1.");
    let clerk = Clerk::new(abacus_address, client.client());
    let abacus = Abacus::new(abacus_address, client.client());

    let asset_prices_id = 101;
    let asset_slopes_id = 102;
    let asset_weights_id = 103;
    let index_quote_id = 104;
    let asset_prices = Vector::from_vec_u128(vec![amount!(2.5).to_u128_raw(); 100]); //amount_vec![1.5, 2.5];
    let asset_slopes = Vector::from_vec_u128(vec![amount!(0.0625).to_u128_raw(); 100]); //amount_vec![0.25, 0.0625];
    let asset_weights = Vector::from_vec_u128(vec![amount!(4.0).to_u128_raw(); 100]); //amount_vec![2.0, 4.0];

    log_msg!("Sending index parameters...");

    client
        .begin_tx()
        .add(clerk.store(asset_prices_id, asset_prices.to_bytes()))
        .add(clerk.store(asset_slopes_id, asset_slopes.to_bytes()))
        .add(clerk.store(asset_weights_id, asset_weights.to_bytes()))
        .send()
        .await?;

    client
        .begin_tx()
        .add(abacus.execute(
            abacus![
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
            ].to_bytes(),
            16,
        ))
        .send()
        .await?;

    log_msg!("Getting index quote...");

    let _index_quote = Vector::from_vec(
        clerk
            .load(index_quote_id)
            .call()
            .await
            .expect("Failed to get index quote"),
    );
    log_msg!("Index Quote: {:0.9}", _index_quote);
    Ok(())
}
