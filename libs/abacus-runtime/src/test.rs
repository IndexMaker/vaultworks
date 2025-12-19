use std::collections::HashMap;

use abacus_formulas::execute_buy_order::execute_buy_order;
use abacus_formulas::solve_quadratic::solve_quadratic;
use abacus_macros::abacus;
use common::{labels::Labels, log_msg, vector::Vector};
use labels_macros::label_vec;
use vector_macros::amount_vec;

use crate::log_stack;
use crate::runtime::*; // Use glob import for tidiness

mod test_utils {
    use common::abacus::program_error::*;

    use super::*;

    pub(super) struct TestVectorIO {
        labels: HashMap<u128, Labels>,
        vectors: HashMap<u128, Vector>,
        codes: HashMap<u128, Vec<u8>>,
    }

    impl TestVectorIO {
        pub(super) fn new() -> Self {
            Self {
                labels: HashMap::new(),
                vectors: HashMap::new(),
                codes: HashMap::new(),
            }
        }

        pub fn store_code(&mut self, id: u128, input: Vec<u8>) -> Result<(), ErrorCode> {
            log_msg!("Storing code {}: {:?}", id, input);
            self.codes.insert(id, input);
            Ok(())
        }
    }

    impl VectorIO for TestVectorIO {
        fn load_labels(&self, id: u128) -> Result<Labels, ErrorCode> {
            let v = self.labels.get(&id).ok_or_else(|| {
                log_msg!("Labels not found: {}", id);
                ErrorCode::NotFound
            })?;
            log_msg!("Loaded labels {}: {}", id, v);
            Ok(Labels {
                data: v.data.clone(),
            })
        }

        fn load_vector(&self, id: u128) -> Result<Vector, ErrorCode> {
            let v = self.vectors.get(&id).ok_or_else(|| {
                log_msg!("Vector not found: {}", id);
                ErrorCode::NotFound
            })?;
            log_msg!("Loaded vector {}: {}", id, v);
            Ok(Vector {
                data: v.data.clone(),
            })
        }

        fn load_code(&self, id: u128) -> Result<Vec<u8>, ErrorCode> {
            let v = self.codes.get(&id).ok_or_else(|| {
                log_msg!("Code not found: {}", id);
                ErrorCode::NotFound
            })?;
            log_msg!("Loaded code {}: {:?}", id, v);
            Ok(v.clone())
        }

        fn store_labels(&mut self, id: u128, input: Labels) -> Result<(), ErrorCode> {
            log_msg!("Storing labels {}: {:0.9}", id, input);
            self.labels.insert(id, input);
            Ok(())
        }

        fn store_vector(&mut self, id: u128, input: Vector) -> Result<(), ErrorCode> {
            log_msg!("Storing vector {}: {:0.9}", id, input);
            self.vectors.insert(id, input);
            Ok(())
        }
    }

    pub(super) struct TestProgram<'a> {
        program: VectorVM<'a, TestVectorIO>,
    }

    impl<'a> TestProgram<'a> {
        pub(super) fn new(vio: &'a mut TestVectorIO) -> Self {
            Self {
                program: VectorVM::new(vio),
            }
        }

        pub(super) fn execute(&mut self, _message: &str, code: Vec<u8>) {
            log_msg!("\nExecute: {}", _message);
            let num_registers = 16;
            let mut stack = Stack::new(num_registers);
            let result = self.program.execute_with_stack(code, &mut stack);
            if let Err(err) = result {
                log_stack!(&stack);
                panic!("Failed to execute test: {:?}", err);
            }
        }
    }
}

mod unit_tests {
    use super::*;
    #[test]
    fn test_transpose() {
        let mut vio = test_utils::TestVectorIO::new();
        let num_registers = 8;

        // --- 1. Setup VIO Inputs ---
        let vector1_id = 100;
        let vector2_id = 101;
        let expected1_id = 102; // T1: [1, 4]
        let expected2_id = 103; // T2: [2, 5]
        let expected3_id = 104; // T3: [3, 6]
        let delta_id = 105;

        vio.store_vector(vector1_id, amount_vec![1, 2, 3]).unwrap();
        vio.store_vector(vector2_id, amount_vec![4, 5, 6]).unwrap();
        vio.store_vector(expected1_id, amount_vec![1, 4]).unwrap();
        vio.store_vector(expected2_id, amount_vec![2, 5]).unwrap();
        vio.store_vector(expected3_id, amount_vec![3, 6]).unwrap();

        // --- 2. VIL Code Execution ---
        let code = abacus![
            // 1. Setup Transposition
            LDV         vector1_id              // Stack: [V1]
            LDV         vector2_id              // Stack: [V1, V2]
            T           2                       // Stack: [T1, T2, T3] (3 vectors)

            // 2. Load Expected Vectors for comparison
            LDV         expected1_id            // [T1, T2, T3, E1]
            LDV         expected2_id            // [T1, T2, T3, E1, E2]
            LDV         expected3_id            // [T1, T2, T3, E1, E2, E3] (6 vectors)

            // 3. D3 = T3 - E3
            SUB         3                       // Stack: [T1, T2, T3, E1, E2, D3]

            // 4. D2 = T2 - E2
            SWAP        1                       // Stack: [T1, T2, T3, E1, D3, E2]
            SUB         4                       // Stack: [T1, T2, T3, E1, D3, D2]

            // 5. D1 = T1 - E1
            SWAP        2                       // Stack: [T1, T2, T3, D2, D3, E1]
            SUB         5                       // Stack: [T1, T2, T3, D2, D3, D1]

            // 6. Compute total delta - should be zero
            ADD         1                       // Stack: [T1, T2, T3, D2, D3, D1 + D3]
            ADD         2                       // Stack: [T1, T2, T3, D2, D3, D1 + D3 + D2]

            // 7. Store the final zero vector
            STV         delta_id
        ];

        let mut stack = Stack::new(num_registers);
        let mut program = VectorVM::new(&mut vio);

        if let Err(err) = program.execute_with_stack(code, &mut stack) {
            log_stack!(&stack);
            panic!("Failed to execute test: {:?}", err);
        }

        // --- 3. Assertion ---
        let delta = vio.load_vector(delta_id).unwrap();

        assert_eq!(delta.data, amount_vec![0, 0].data);
    }
}

mod test_scenarios {
    use abacus_formulas::{
        add_market_assets::add_market_assets, create_market::create_market,
        update_margin::update_margin, update_market_data::update_market_data,
        update_quote::update_quote, update_supply::update_supply,
    };
    use amount_macros::amount;

    use super::*;

    /// All round test verifies that majority of VIL functionality works as expected.
    ///
    /// We test:
    /// - load and store of vectors (externally via VectorIO)
    /// - load and store of values into registry
    /// - invocation of sub-routines w/ parameters and return values
    /// - example implementation of Solve-Quadratic function (vectorised)
    /// - example implementation of index order asset quantity computation from index asset weights
    ///
    /// The purpose of VIL is to allow generic vector operations in Stylus smart-contracts, so that:
    /// - vector data is loaded and stored into blockchain once and only once
    /// - vector data is modified in-place whenever possible, and only duplicated when necessary
    /// - labels and join operations allow sparse vector addition and saturating subtraction
    /// - data and operations live in the same smart-contract without exceeding 24KiB WASM limit
    /// - other smart-contracts can submit VIL code to execute without them-selves exceeding 24KiB WASM limit
    /// - minimisation of gas use by reduction of blockchain operations and executed instructions
    ///
    /// NOTE: while VIL is an assembly language, it is limitted exclusively to perform vector math, and
    /// instruction set is designed to particularly match our requirements to execute index orders and
    /// update market.
    ///
    /// TBD: examine real-life gas usage and limits.
    ///
    #[test]
    fn test_buy_index() {
        let mut vio = test_utils::TestVectorIO::new();
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

        vio.store_labels(asset_names_id, label_vec![51, 53, 54])
            .unwrap();

        vio.store_vector(weights_id, amount_vec![0.100, 1.000, 100.0])
            .unwrap();

        vio.store_vector(asset_contribution_fractions_id, amount_vec![1, 1, 1])
            .unwrap();

        vio.store_vector(quote_id, amount_vec![10.00, 10_000, 100.0])
            .unwrap();

        vio.store_vector(index_order_id, amount_vec![950.00, 0, 0])
            .unwrap();

        vio.store_labels(market_asset_names_id, label_vec![51, 52, 53, 54, 55])
            .unwrap();

        vio.store_vector(demand_short_id, amount_vec![0, 0, 0.01, 0, 0])
            .unwrap();

        vio.store_vector(demand_long_id, amount_vec![0.1, 0.1, 0, 0.01, 0.2])
            .unwrap();

        vio.store_vector(supply_short_id, amount_vec![0, 0, 0, 0, 0])
            .unwrap();

        vio.store_vector(supply_long_id, amount_vec![0.05, 0.05, 0.05, 0.05, 0.05])
            .unwrap();

        vio.store_vector(delta_short_id, amount_vec![0, 0, 0, 0, 0])
            .unwrap();

        vio.store_vector(delta_long_id, amount_vec![0, 0, 0, 0, 0])
            .unwrap();

        vio.store_vector(margin_id, amount_vec![0.2, 0.2, 0.2, 20.0, 0.2])
            .unwrap();

        vio.store_code(solve_quadratic_id, solve_quadratic())
            .unwrap();

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

        let order_before = vio.load_vector(index_order_id).unwrap();

        let num_registers = 16;

        let mut program = VectorVM::new(&mut vio);
        let mut stack = Stack::new(num_registers);
        let result = program.execute_with_stack(code, &mut stack);

        if let Err(err) = result {
            log_stack!(&stack);
            panic!("Failed to execute test: {:?}", err);
        }

        let order_after = vio.load_vector(index_order_id).unwrap();
        let quote = vio.load_vector(quote_id).unwrap();
        let weigths = vio.load_vector(weights_id).unwrap();
        let index_quantites = vio.load_vector(executed_index_quantities_id).unwrap();
        let asset_quantites = vio.load_vector(executed_asset_quantities_id).unwrap();
        let demand_short = vio.load_vector(demand_short_id).unwrap();
        let demand_long = vio.load_vector(demand_long_id).unwrap();
        let delta_short = vio.load_vector(delta_short_id).unwrap();
        let delta_long = vio.load_vector(delta_long_id).unwrap();

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

        assert_eq!(order_before.data, amount_vec![950, 0, 0].data);
        assert_eq!(quote.data, amount_vec![10, 10_000, 100].data);
        assert_eq!(weigths.data, amount_vec![0.1, 1, 100,].data);

        // these are exact expected fixed point decimal values as raw u128
        assert_eq!(
            order_after.data,
            amount_vec![0.000000013986019975, 999.999999986013980025, 0.0999001995].data
        );
        assert_eq!(
            index_quantites.data,
            amount_vec![0.0999001995, 0.000000000].data
        );
        assert_eq!(
            asset_quantites.data,
            amount_vec![0.00999001995, 0.0999001995, 9.99001995].data
        );
        assert_eq!(demand_short.data, amount_vec![0, 0, 0, 0, 0].data);
        assert_eq!(
            demand_long.data,
            amount_vec![0.10999001995, 0.1, 0.0899001995, 10.00001995, 0.2].data
        );
        assert_eq!(
            delta_short.data,
            amount_vec![0.05999001995, 0.05, 0.0399001995, 9.95001995, 0.15].data
        );
        assert_eq!(delta_long.data, amount_vec![0, 0, 0, 0, 0].data);
    }

    #[test]
    fn test_update_assets() {
        let mut vio = test_utils::TestVectorIO::new();

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

        {
            let new_market_asset_names_id = 901;
            vio.store_labels(new_market_asset_names_id, label_vec![101, 103])
                .unwrap();

            let mut program = test_utils::TestProgram::new(&mut vio);
            program.execute(
                "create market",
                create_market(
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
            );
        }
        {
            let new_market_asset_names_id = 901;
            vio.store_labels(new_market_asset_names_id, label_vec![102, 104, 105, 106])
                .unwrap();

            let mut program = test_utils::TestProgram::new(&mut vio);
            program.execute(
                "update assets",
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
            );
        }

        let asset_names_id = 902;
        let asset_prices_id = 903;
        let asset_slopes_id = 904;
        let asset_liquidity_id = 905;
        let asset_margin_id = 906;
        let asset_quantities_short_id = 907;
        let asset_quantities_long_id = 908;

        let weights_id = 1001;
        let quote_id = 1002;

        vio.store_labels(asset_names_id, label_vec![101, 103, 104])
            .unwrap();
        vio.store_vector(asset_prices_id, amount_vec![500.0, 1000.0, 100.0])
            .unwrap();
        vio.store_vector(asset_slopes_id, amount_vec![5.0, 10.0, 1.0])
            .unwrap();
        vio.store_vector(asset_liquidity_id, amount_vec![20.0, 10.0, 100.0])
            .unwrap();
        vio.store_vector(asset_margin_id, amount_vec![10.0, 10.0, 50.0])
            .unwrap();
        vio.store_vector(asset_quantities_long_id, amount_vec![1.0, 0, 5.0])
            .unwrap();
        vio.store_vector(asset_quantities_short_id, amount_vec![0, 2.0, 0])
            .unwrap();
        vio.store_vector(weights_id, amount_vec![4.0, 8.0, 20.0])
            .unwrap();
        vio.store_vector(quote_id, amount_vec![0, 0, 0]).unwrap();

        let mut program = test_utils::TestProgram::new(&mut vio);

        program.execute(
            "update margin",
            update_margin(
                asset_names_id,
                asset_margin_id,
                market_asset_names_id,
                margin_id,
            ),
        );

        program.execute(
            "update market data",
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
        );

        program.execute(
            "update supply",
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
        );

        program.execute(
            "update quote",
            update_quote(
                asset_names_id,
                weights_id,
                quote_id,
                market_asset_names_id,
                market_asset_prices_id,
                market_asset_slopes_id,
                market_asset_liquidity_id,
            ),
        );

        let new_margin = vio.load_vector(margin_id).unwrap();
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

        let new_market_asset_prices = vio.load_vector(market_asset_prices_id).unwrap();
        let new_market_asset_slopes = vio.load_vector(market_asset_slopes_id).unwrap();
        let new_market_asset_liquidity = vio.load_vector(asset_liquidity_id).unwrap();
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

        let new_supply_long = vio.load_vector(supply_long_id).unwrap();
        let new_supply_short = vio.load_vector(supply_short_id).unwrap();
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

        let new_delta_long = vio.load_vector(delta_long_id).unwrap();
        let new_delta_short = vio.load_vector(delta_short_id).unwrap();
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

        let new_quote = vio.load_vector(quote_id).unwrap();
        assert_eq!(
            new_quote.data,
            amount_vec![1.250000000, 12000.000000000, 1120.000000000].data
        )
    }
}
