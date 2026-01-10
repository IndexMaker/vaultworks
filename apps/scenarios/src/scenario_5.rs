use alloy_primitives::uint;
use amount_macros::amount;
use common::{labels::Labels, log_msg, vector::Vector};
use ethers::types::Address;
use eyre::bail;
use labels_macros::label_vec;
use vector_macros::amount_vec;

use common_ethers::{
    contracts::{Banker, Factor, Guildmaster},
    tx_sender::TxClient,
    ToBytes,
};

pub async fn run_scenario(
    client: &TxClient,
    castle_address: Address,
    keeper_address: Address,
) -> eyre::Result<()> {
    log_msg!("Scenario 5.");

    if keeper_address == client.address() {
        bail!("Keeper must use distinct address")
    }

    let banker = Banker::new(castle_address, client.client());
    let guildmaster = Guildmaster::new(castle_address, client.client());
    let factor = Factor::new(castle_address, client.client());

    let vendor_id = uint!(1u128);
    let index_id = 1001;

    {
        log_msg!("Submit Assets #1");

        let asset_names = label_vec!(101, 102, 104, 105, 106);

        client
            .begin_tx()
            .add(banker.submit_assets(vendor_id, asset_names.to_bytes()))
            .send()
            .await?;
    }

    {
        log_msg!("Submit Assets #2");

        let asset_names = label_vec!(102, 103, 107, 108, 109);

        client
            .begin_tx()
            .add(banker.submit_assets(vendor_id, asset_names.to_bytes()))
            .send()
            .await?;
    }

    {
        log_msg!("Submit Margin");

        let asset_names = label_vec!(101, 102, 103, 104, 105, 106, 107, 108, 109);
        let asset_margin = amount_vec!(2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0);

        client
            .begin_tx()
            .add(banker.submit_margin(vendor_id, asset_names.to_bytes(), asset_margin.to_bytes()))
            .send()
            .await?;
    }

    {
        log_msg!("Submit Supply");

        let asset_names = label_vec!(101, 102, 103, 104, 105, 106, 107, 108, 109);
        let asset_short = amount_vec!(0, 0, 0, 0, 0, 0, 0, 0, 0);
        let asset_long = amount_vec!(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0);

        client
            .begin_tx()
            .add(banker.submit_supply(
                vendor_id,
                asset_names.to_bytes(),
                asset_short.to_bytes(),
                asset_long.to_bytes(),
            ))
            .send()
            .await?;
    }

    {
        log_msg!("Submit Index");

        let asset_names = label_vec!(102, 103, 104, 106, 107);
        let asset_weights = amount_vec!(1.0, 0.5, 0.5, 0.5, 1.5);
        let info = b"Test Index 1001".to_vec();

        client
            .begin_tx()
            .add(guildmaster.submit_index(
                index_id,
                asset_names.to_bytes(),
                asset_weights.to_bytes(),
                info.to_bytes(),
            ))
            .send()
            .await?;
    }

    {
        log_msg!("Submit Vote");

        let vote = vec![];

        client
            .begin_tx()
            .add(guildmaster.submit_vote(index_id, vote.to_bytes()))
            .send()
            .await?;
    }

    {
        log_msg!("Submit Market Data");

        let asset_names = label_vec!(102, 103, 104, 106, 107);
        let asset_liquidity = amount_vec!(0.5, 0.5, 0.5, 0.5, 0.5);
        let asset_prices = amount_vec!(100.0, 50.0, 20.0, 10.0, 1.0);
        let asset_slopes = amount_vec!(1.0, 0.5, 0.2, 0.1, 0.01);

        client
            .begin_tx()
            .add(factor.submit_market_data(
                vendor_id,
                asset_names.to_bytes(),
                asset_liquidity.to_bytes(),
                asset_prices.to_bytes(),
                asset_slopes.to_bytes(),
            ))
            .send()
            .await?;
    }

    {
        log_msg!("Update Index Quote");

        client
            .begin_tx()
            .add(guildmaster.update_index_quote(vendor_id, index_id))
            .send()
            .await?;
    }

    {
        log_msg!("Submit Buy Order");

        let collateral_added = amount!(10.0);
        let max_order_size = amount!(1000.0);

        let result = client
            .begin_tx()
            .add(factor.execute_buy_order(
                vendor_id,
                index_id,
                client.address(),
                keeper_address,
                collateral_added.to_u128_raw(),
                max_order_size.to_u128_raw(),
            ))
            .send()
            .await?;

        log_msg!("Buy order placement result: {:?}", result);
    }

    {
        log_msg!("Submit Sell Order");

        let collateral_added = amount!(0.04);
        let max_order_size = amount!(1000.0);

        let result = client
            .begin_tx()
            .add(factor.execute_sell_order(
                vendor_id,
                index_id,
                client.address(),
                keeper_address,
                collateral_added.to_u128_raw(),
                max_order_size.to_u128_raw(),
            ))
            .send()
            .await?;

        log_msg!("Sell order placement result: {:?}", result);
    }

    Ok(())
}
