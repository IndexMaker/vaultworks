use deli::{labels::Labels, log_msg, vector::Vector};
use ethers::types::Address;

use decon::{contracts::Clerk, tx_sender::TxClient};
use icore::vil::{
    add_market_assets::add_market_assets, update_margin::update_margin,
    update_market_data::update_market_data, update_quote::update_quote,
    update_supply::update_supply,
};
use labels_macros::label_vec;
use vector_macros::amount_vec;

pub async fn run_scenario(client: &TxClient, abacus_address: Address) -> eyre::Result<()> {
    log_msg!("Scenario 3.");
    let clerk = Clerk::new(abacus_address, client.client());

    let market_asset_names_id = 101;
    let market_asset_prices_id = 102;
    let market_asset_slopes_id = 103;
    let market_asset_liquidity_id = 104;
    let supply_long_id = 105;
    let supply_short_id = 106;
    let demand_long_id = 107;
    let demand_short_id = 108;
    let delta_long_id = 109;
    let delta_short_id = 110;
    let margin_id = 111;

    let new_market_asset_names_id = 901;

    let asset_names_id = 902;
    let asset_prices_id = 903;
    let asset_slopes_id = 904;
    let asset_liquidity_id = 905;
    let asset_margin_id = 906;
    let asset_quantities_short_id = 907;
    let asset_quantities_long_id = 908;

    let weights_id = 1001;
    let quote_id = 1002;

    log_msg!("Create State");
    client
        .begin_tx()
        .add(clerk.store(market_asset_names_id, label_vec![].to_vec()))
        .add(clerk.store(market_asset_prices_id, amount_vec![].to_vec()))
        .add(clerk.store(market_asset_slopes_id, amount_vec![].to_vec()))
        .add(clerk.store(market_asset_liquidity_id, amount_vec![].to_vec()))
        .add(clerk.store(supply_long_id, amount_vec![].to_vec()))
        .add(clerk.store(supply_short_id, amount_vec![].to_vec()))
        .add(clerk.store(demand_long_id, amount_vec![].to_vec()))
        .add(clerk.store(demand_short_id, amount_vec![].to_vec()))
        .add(clerk.store(delta_long_id, amount_vec![].to_vec()))
        .add(clerk.store(delta_short_id, amount_vec![].to_vec()))
        .add(clerk.store(margin_id, amount_vec![].to_vec()))
        .send()
        .await?;

    log_msg!("Update Assets (1)");
    client
        .begin_tx()
        .add(clerk.store(
            new_market_asset_names_id,
            label_vec![101, 103, 104].to_vec(),
        ))
        .send()
        .await?;

    client
        .begin_tx()
        .add(clerk.execute(
            add_market_assets(
                new_market_asset_names_id,
                market_asset_names_id,
                market_asset_prices_id,
                market_asset_slopes_id,
                market_asset_liquidity_id,
                supply_long_id,
                supply_short_id,
                demand_long_id,
                demand_short_id,
                delta_long_id,
                delta_short_id,
                margin_id,
            ),
            16,
        ))
        .send()
        .await?;

    log_msg!("Update Assets (2)");
    client
        .begin_tx()
        .add(clerk.store(
            new_market_asset_names_id,
            label_vec![102, 104, 105, 106].to_vec(),
        ))
        .send()
        .await?;

    client
        .begin_tx()
        .add(clerk.execute(
            add_market_assets(
                new_market_asset_names_id,
                market_asset_names_id,
                market_asset_prices_id,
                market_asset_slopes_id,
                market_asset_liquidity_id,
                supply_long_id,
                supply_short_id,
                demand_long_id,
                demand_short_id,
                delta_long_id,
                delta_short_id,
                margin_id,
            ),
            16,
        ))
        .send()
        .await?;

    let new_assset_names = Labels::from_vec(clerk.load(market_asset_names_id).call().await?);
    assert_eq!(
        new_assset_names.data,
        label_vec![101, 102, 103, 104, 105, 106].data
    );

    log_msg!("Submit Inputs");
    client
        .begin_tx()
        .add(clerk.store(asset_names_id, label_vec![101, 103, 104].to_vec()))
        .add(clerk.store(asset_prices_id, amount_vec![500.0, 1000.0, 100.0].to_vec()))
        .add(clerk.store(asset_slopes_id, amount_vec![5.0, 10.0, 1.0].to_vec()))
        .add(clerk.store(asset_liquidity_id, amount_vec![20.0, 10.0, 100.0].to_vec()))
        .add(clerk.store(asset_margin_id, amount_vec![10.0, 10.0, 50.0].to_vec()))
        .add(clerk.store(asset_quantities_long_id, amount_vec![1.0, 0, 5.0].to_vec()))
        .add(clerk.store(asset_quantities_short_id, amount_vec![0, 2.0, 0].to_vec()))
        .add(clerk.store(weights_id, amount_vec![4.0, 8.0, 20.0].to_vec()))
        .add(clerk.store(quote_id, amount_vec![0, 0, 0].to_vec()))
        .send()
        .await?;

    log_msg!("Update Margin");
    client
        .begin_tx()
        .add(clerk.execute(
            update_margin(
                asset_names_id,
                asset_margin_id,
                market_asset_names_id,
                margin_id,
            ),
            16,
        ))
        .send()
        .await?;

    log_msg!("Update Market Data");
    client
        .begin_tx()
        .add(clerk.execute(
            update_market_data(
                asset_names_id,
                asset_prices_id,
                asset_slopes_id,
                asset_liquidity_id,
                market_asset_names_id,
                market_asset_prices_id,
                market_asset_slopes_id,
                market_asset_liquidity_id,
            ),
            16,
        ))
        .send()
        .await?;

    log_msg!("Update Supply");
    client
        .begin_tx()
        .add(clerk.execute(
            update_supply(
                asset_names_id,
                asset_quantities_short_id,
                asset_quantities_long_id,
                market_asset_names_id,
                supply_long_id,
                supply_short_id,
                demand_long_id,
                demand_short_id,
                delta_long_id,
                delta_short_id,
            ),
            16,
        ))
        .send()
        .await?;

    log_msg!("Update Quote");
    client
        .begin_tx()
        .add(clerk.execute(
            update_quote(
                asset_names_id,
                weights_id,
                quote_id,
                market_asset_names_id,
                market_asset_prices_id,
                market_asset_slopes_id,
                market_asset_liquidity_id,
            ),
            16,
        ))
        .send()
        .await?;

    let new_margin = Vector::from_vec(clerk.load(margin_id).call().await?);
    assert_eq!(
        new_margin.data,
        amount_vec![
            10.000000000000000000,
            0.000000000000000000,
            10.000000000000000000,
            50.000000000000000000,
            0.000000000000000000,
            0.000000000000000000
        ]
        .data
    );

    let new_market_asset_prices =
        Vector::from_vec(clerk.load(market_asset_prices_id).call().await?);
    let new_market_asset_slopes =
        Vector::from_vec(clerk.load(market_asset_slopes_id).call().await?);
    let new_market_asset_liquidity =
        Vector::from_vec(clerk.load(asset_liquidity_id).call().await?);
    assert_eq!(
        new_market_asset_prices.data,
        amount_vec![
            500.000000000000000000,
            0.000000000000000000,
            1000.000000000000000000,
            100.000000000000000000,
            0.000000000000000000,
            0.000000000000000000
        ]
        .data
    );
    assert_eq!(
        new_market_asset_slopes.data,
        amount_vec![
            5.000000000000000000,
            0.000000000000000000,
            10.000000000000000000,
            1.000000000000000000,
            0.000000000000000000,
            0.000000000000000000
        ]
        .data
    );
    assert_eq!(
        new_market_asset_liquidity.data,
        amount_vec![
            20.000000000000000000,
            10.000000000000000000,
            100.000000000000000000
        ]
        .data
    );

    let new_supply_long = Vector::from_vec(clerk.load(supply_long_id).call().await?);
    let new_supply_short = Vector::from_vec(clerk.load(supply_short_id).call().await?);
    assert_eq!(
        new_supply_long.data,
        amount_vec![
            1.000000000,
            0.000000000,
            0.000000000,
            5.000000000,
            0.000000000,
            0.000000000
        ]
        .data
    );
    assert_eq!(
        new_supply_short.data,
        amount_vec![
            0.000000000,
            0.000000000,
            2.000000000,
            0.000000000,
            0.000000000,
            0.000000000
        ]
        .data
    );

    let new_delta_long = Vector::from_vec(clerk.load(delta_long_id).call().await?);
    let new_delta_short = Vector::from_vec(clerk.load(delta_short_id).call().await?);
    assert_eq!(
        new_delta_long.data,
        amount_vec![
            1.000000000,
            0.000000000,
            0.000000000,
            5.000000000,
            0.000000000,
            0.000000000
        ]
        .data
    );
    assert_eq!(
        new_delta_short.data,
        amount_vec![
            0.000000000,
            0.000000000,
            2.000000000,
            0.000000000,
            0.000000000,
            0.000000000
        ]
        .data
    );

    let new_quote = Vector::from_vec(clerk.load(quote_id).call().await?);
    assert_eq!(
        new_quote.data,
        amount_vec![1.250000000, 12000.000000000, 1120.000000000].data
    );

    Ok(())
}
