use amount_macros::amount;
use deli::{labels::Labels, log_msg, vector::Vector};
use ethers::types::Address;
use icore::vil::{
    execute_buy_order::execute_buy_order,
    solve_quadratic::{self, solve_quadratic},
};
use itertools::{chain, Itertools};
use labels_macros::label_vec;
use vector_macros::amount_vec;

use decon::{contracts::Devil, tx_sender::TxClient};

pub async fn run_scenario(client: &TxClient, devil_address: Address) -> eyre::Result<()> {
    log_msg!("Scenario 2.");
    let devil = Devil::new(devil_address, client.client());

    let index_order_id = 10001;
    let executed_asset_quantities_id = 10002;
    let executed_index_quantities_id = 10003;
    let asset_names_id = 1001;
    let weights_id = 1002;
    let quote_id = 1003;
    let market_asset_names_id = 101;
    let supply_long_id = 102;
    let supply_short_id = 103;
    let demand_long_id = 104;
    let demand_short_id = 105;
    let delta_long_id = 106;
    let delta_short_id = 107;
    let margin_id = 108;
    let asset_contribution_fractions_id = 109;
    let solve_quadratic_id = 10;

    let collateral_added = amount!(100.0);
    let collateral_removed = amount!(50.0);
    let max_order_size = amount!(10000.0);

    let asset_names = Labels {
        data: chain!(
            [1, 2, 5, 9, 11, 12, 15, 17, 18, 21],
            (31..41).into_iter(),
            [51, 52, 55, 57, 59, 62, 63, 64, 68, 69],
            (71..81).into_iter(),
            (81..91).into_iter()
        )
        .collect_vec(),
    };
    let asset_vector = |val| Vector {
        data: asset_names.data.iter().map(|_| val).collect_vec(),
    };

    let market_asset_names = Labels {
        data: (1..151).into_iter().collect_vec(),
    };
    let market_vector = |val| Vector {
        data: market_asset_names.data.iter().map(|_| val).collect_vec(),
    };

    client
        .begin_tx()
        .add(devil.submit(asset_names_id, asset_names.to_vec()))
        .add(devil.submit(weights_id, asset_vector(amount!(0.1)).to_vec()))
        .add(devil.submit(asset_contribution_fractions_id, asset_vector(amount!(1.0)).to_vec()))
        .add(devil.submit(quote_id, amount_vec![10.00, 10_000, 100.0].to_vec()))
        .add(devil.submit(index_order_id, amount_vec![950.00, 0, 0].to_vec()))
        .add(devil.submit(market_asset_names_id, market_asset_names.to_vec()))
        .add(devil.submit(demand_short_id, market_vector(amount!(0)).to_vec()))
        .add(devil.submit(demand_long_id, market_vector(amount!(0.1)).to_vec()))
        .add(devil.submit(supply_short_id, market_vector(amount!(0)).to_vec()))
        .add(devil.submit(supply_long_id, market_vector(amount!(0.05)).to_vec()))
        .add(devil.submit(delta_short_id, market_vector(amount!(0)).to_vec()))
        .add(devil.submit(delta_long_id, market_vector(amount!(0)).to_vec()))
        .add(devil.submit(margin_id, market_vector(amount!(20.0)).to_vec()))
        .send()
        .await?;

    let solve_quadratic_code = solve_quadratic();

    log_msg!("Solve Quadratic Code: {:?}", solve_quadratic_code);

    client
        .begin_tx()
        .add(devil.submit(solve_quadratic_id, solve_quadratic_code))
        .send()
        .await?;

    let code = execute_buy_order(
        index_order_id,
        collateral_added.to_u128_raw(),
        collateral_removed.to_u128_raw(),
        max_order_size.to_u128_raw(),
        executed_index_quantities_id,
        executed_asset_quantities_id,
        asset_names_id,
        weights_id,
        quote_id,
        market_asset_names_id,
        supply_long_id,
        supply_short_id,
        demand_long_id,
        demand_short_id,
        delta_long_id,
        delta_short_id,
        margin_id,
        asset_contribution_fractions_id,
        solve_quadratic_id,
    );

    log_msg!("Code: {:?}", code);

    let order_before = Vector::from_vec(devil.get(index_order_id).call().await?);
    let num_registers = 16;

    client
        .begin_tx()
        .add(devil.execute(code, num_registers))
        .send()
        .await?;

    let order_after = Vector::from_vec(devil.get(index_order_id).call().await?);
    let quote = Vector::from_vec(devil.get(quote_id).call().await?);
    let weigths = Vector::from_vec(devil.get(weights_id).call().await?);
    let index_quantites = Vector::from_vec(devil.get(executed_index_quantities_id).call().await?);
    let asset_quantites = Vector::from_vec(devil.get(executed_asset_quantities_id).call().await?);
    let demand_short = Vector::from_vec(devil.get(demand_short_id).call().await?);
    let demand_long = Vector::from_vec(devil.get(demand_long_id).call().await?);
    let delta_short = Vector::from_vec(devil.get(delta_short_id).call().await?);
    let delta_long = Vector::from_vec(devil.get(delta_long_id).call().await?);

    log_msg!("\n-= Program complete =-");
    log_msg!("\n[in] Index Order = {:0.9}", order_before);
    log_msg!("[in] Collateral Added = {:0.9}", collateral_added);
    log_msg!("[in] Collateral Removed = {:0.9}", collateral_removed);
    log_msg!("[in] Index Quote = {:0.9}", quote);
    log_msg!("[in] Asset Weights = {:0.9}", weigths);
    log_msg!("\n[out] Index Order = {:0.9}", order_after);
    log_msg!("[out] Index Quantities = {:0.9}", index_quantites);
    log_msg!("[out] Asset Quantities = {:0.9}", asset_quantites);
    log_msg!("\n[out] Demand Short = {:0.9}", demand_short);
    log_msg!("[out] Demand Long = {:0.9}", demand_long);
    log_msg!("\n[out] Delta Short = {:0.9}", delta_short);
    log_msg!("[out] Delta Long = {:0.9}", delta_long);

    Ok(())
}
