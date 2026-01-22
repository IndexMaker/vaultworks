use std::collections::HashMap;

use abacus_formulas::execute_buy_order::execute_buy_order;
use abacus_formulas::solve_quadratic_bid::solve_quadratic_bid;
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

    pub(super) fn project(data: &Vector, from_names: &Labels, to_names: &Labels) -> Vector {
        let mut vio = test_utils::TestVectorIO::new();
        let num_registers = 0;

        let data_id = 1;
        let from_names_id = 2;
        let to_names_id = 3;

        vio.store_vector(
            data_id,
            Vector {
                data: data.data.clone(),
            },
        )
        .unwrap();
        vio.store_labels(
            from_names_id,
            Labels {
                data: from_names.data.clone(),
            },
        )
        .unwrap();
        vio.store_labels(
            to_names_id,
            Labels {
                data: to_names.data.clone(),
            },
        )
        .unwrap();

        let code = abacus!(
            LDV     data_id         // D
            LDL     from_names_id   // D, N_from
            LDL     to_names_id     // D, N_from, N_to
            LDD     0               // D, N_from, N_to, N_to
            LUNION  2               // D, N_from, N_to, N_union
            ZEROS   1               // D, N_from, N_to, N_union, Z_union
            JUPD    4   1   3       // D, N_from, N_to, N_union, D_union
            JFLT    1   2
            STV     data_id
        );

        let mut stack = Stack::new(num_registers);
        let mut program = VectorVM::new(&mut vio);

        if let Err(err) = program.execute_with_stack(code.unwrap(), &mut stack) {
            log_stack!(&stack);
            panic!("Failed to project: {:?}", err);
        }

        vio.load_vector(data_id).unwrap()
    }
}

mod unit_tests {
    use super::*;

    #[test]
    fn test_joins() {
        let mut vio = test_utils::TestVectorIO::new();

        let names_1 = label_vec![];
        let names_2 = label_vec![1, 3];
        let names_3 = label_vec![2, 3, 5];

        let data_1 = amount_vec![];
        let data_2 = amount_vec![1.5, 2.5];
        let data_3 = amount_vec![5.5, 6.5, 7.5];

        vio.store_labels(1, names_1).unwrap();
        vio.store_labels(2, names_2).unwrap();
        vio.store_labels(3, names_3).unwrap();

        vio.store_vector(4, data_1).unwrap();
        vio.store_vector(5, data_2).unwrap();
        vio.store_vector(6, data_3).unwrap();

        let code = abacus! {
            LDL     1           //  [1]
            LDL     2           //  [1, 2]
            LDL     3           //  [1, 2, 3]
            LDD     0
            STR     _L3
            LUNION  1           //  [1, 2, (2 u 3)]
            LUNION  2           //  [1, 2, U = (1 u (2 u 3))]
            LDD     0
            STL     10

            LDV     4
            LDV     5
            LDV     6           // [1, 2, U, 4, 5, 6]

            ZEROS   3           // [1, 2, U, 4, 5, 6, Z]
            JUPD    3   4   6   // [1, 2, U, 4, 5, 6, A = Z <- 4]
            JUPD    2   4   5   // [1, 2, U, 4, 5, 6, A <- 5]
            STV     11

            LDM     _L3         // [1, 2, U, 4, 5, 6, 3]
            ZEROS   4           // [1, 2, U, 4, 5, 6, 3, Z]
            JUPD    2   5   1
            STV     12
        };

        let num_registry = 1;
        let mut stack = Stack::new(num_registry);
        let mut program = VectorVM::new(&mut vio);

        if let Err(err) = program.execute_with_stack(code.unwrap(), &mut stack) {
            log_stack!(&stack);
            panic!("Failed to execute test: {:?}", err);
        }

        let result10 = vio.load_labels(10).unwrap();
        assert_eq!(result10.data, label_vec![1, 2, 3, 5].data);

        let result11 = vio.load_vector(11).unwrap();
        assert_eq!(result11.data, amount_vec![1.5, 0, 2.5, 0].data);
        
        let result12 = vio.load_vector(12).unwrap();
        assert_eq!(result12.data, amount_vec![0, 5.5, 6.5, 7.5].data);
    }

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

        if let Err(err) = program.execute_with_stack(code.unwrap(), &mut stack) {
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
        execute_rebalance::execute_rebalance, execute_sell_order::execute_sell_order,
        execute_transfer::execute_transfer, solve_quadratic_ask::solve_quadratic_ask,
        update_margin::update_margin, update_market_data::update_market_data,
        update_quote::update_quote, update_rebalance::update_rebalance,
        update_supply::update_supply,
    };
    use amount_macros::amount;

    use crate::test::test_utils::project;

    use super::*;

    /// All round BUY order test verifies that majority of VIL functionality works as expected.
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
        let vendor_order_id = 10002;
        let total_order_id = 10003;
        let executed_asset_quantities_id = 10010;
        let executed_index_quantities_id = 10011;
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
        let solve_quadratic_bid_id = 10;

        let collateral_added = amount!(100.0);
        let collateral_removed = amount!(50.0);
        let max_order_size = amount!(10000.0);

        vio.store_labels(asset_names_id, label_vec![51, 53, 54])
            .unwrap();

        vio.store_vector(weights_id, amount_vec![0.100, 1.000, 100.0])
            .unwrap();

        vio.store_vector(quote_id, amount_vec![10.00, 10_000, 100.0])
            .unwrap();

        vio.store_vector(index_order_id, amount_vec![950.00, 0, 0])
            .unwrap();

        vio.store_vector(vendor_order_id, amount_vec![1950, 20000, 2.0])
            .unwrap();

        vio.store_vector(total_order_id, amount_vec![2950, 50000, 5.0])
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

        vio.store_code(solve_quadratic_bid_id, solve_quadratic_bid().unwrap())
            .unwrap();

        let code = execute_buy_order(
            index_order_id,
            vendor_order_id,
            total_order_id,
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
            solve_quadratic_bid_id,
        );

        let order_before = vio.load_vector(index_order_id).unwrap();
        let vendor_order_before = vio.load_vector(vendor_order_id).unwrap();
        let total_order_before = vio.load_vector(total_order_id).unwrap();

        let num_registers = 23;

        let mut program = VectorVM::new(&mut vio);
        let mut stack = Stack::new(num_registers);
        let result = program.execute_with_stack(code.unwrap(), &mut stack);

        if let Err(err) = result {
            log_stack!(&stack);
            panic!("Failed to execute test: {:?}", err);
        }

        let order_after = vio.load_vector(index_order_id).unwrap();
        let vendor_order_after = vio.load_vector(vendor_order_id).unwrap();
        let total_order_after = vio.load_vector(total_order_id).unwrap();
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
        log_msg!("[in] Vendor Order = {:0.9}", vendor_order_before);
        log_msg!("[in] Total Order = {:0.9}", total_order_before);
        log_msg!("\n[in] Collateral Added = {:0.9}", collateral_added);
        log_msg!("[in] Collateral Removed = {:0.9}", collateral_removed);
        log_msg!("[in] Index Quote = {:0.9}", quote);
        log_msg!("[in] Asset Weights = {:0.9}", weigths);
        log_msg!("\n[out] Index Order = {:0.9}", order_after);
        log_msg!("[out] Vendor Order = {:0.9}", vendor_order_after);
        log_msg!("[out] Total Order = {:0.9}", total_order_after);
        log_msg!("\n[out] Index Quantities = {:0.9}", index_quantites);
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
            amount_vec![999.999999986013980025, 0.0999001995].data
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

    /// Comprehensive SELL order test verifying VIL execution and market impact logic.
    ///
    /// We test:
    /// - Multi-stage collateral updates (addition and removal) prior to execution.
    /// - Dynamic Capacity Limit (CL) calculation via VMIN over margin-constrained assets.
    /// - Execution throttling where Margin/Capacity constraints override available Collateral.
    /// - Inverse solving for Index Quantity using the vectorized quadratic sub-routine.
    /// - Linear scaling of Asset Quantities based on non-uniform asset weights.
    /// - Market state mutation: sparse Demand Long depletion and overflow into Demand Short.
    /// - Delta imbalance recalculation across the entire market asset set.
    ///
    /// Key VIL features exercised:
    /// - Register-based parameter passing for complex mathematical sub-programs.
    /// - Saturating vector arithmetic (SSB) for calculating net market demand.
    /// - Label-based joins for aligning Index-specific assets with global Market vectors.
    /// - Multi-vector packing and unpacking into single registry slots (Order State).
    ///
    /// The test specifically confirms that the "Slippage" (S) curve correctly reduces
    /// the effective withdrawal amount, and that the VM accurately solves for the
    /// required Index Burn to satisfy the capped output.
    #[test]
    fn test_sell_index() {
        let mut vio = test_utils::TestVectorIO::new();
        let index_order_id = 10001;
        let vendor_order_id = 10002;
        let total_order_id = 10003;
        let executed_asset_quantities_id = 10010;
        let executed_index_quantities_id = 10011;
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
        let solve_quadratic_ask_id = 10;

        let collateral_added = amount!(0.75);
        let collateral_removed = amount!(0.25);
        let max_order_size = amount!(10000.0);

        vio.store_labels(asset_names_id, label_vec![51, 53, 54])
            .unwrap();

        vio.store_vector(weights_id, amount_vec![0.100, 1.000, 100.0])
            .unwrap();

        vio.store_vector(quote_id, amount_vec![10.00, 10_000, 100.0])
            .unwrap();

        vio.store_vector(index_order_id, amount_vec![1.00, 0, 0])
            .unwrap();

        vio.store_vector(vendor_order_id, amount_vec![0, 0, 0])
            .unwrap();

        vio.store_vector(total_order_id, amount_vec![0, 0, 0])
            .unwrap();

        vio.store_labels(market_asset_names_id, label_vec![51, 52, 53, 54, 55])
            .unwrap();

        vio.store_vector(demand_short_id, amount_vec![0, 0, 0.0, 0.0, 1.0])
            .unwrap();

        vio.store_vector(demand_long_id, amount_vec![1.0, 0.0, 1.0, 60.0, 0.0])
            .unwrap();

        vio.store_vector(supply_short_id, amount_vec![0.5, 0, 0, 0, 0])
            .unwrap();

        vio.store_vector(supply_long_id, amount_vec![0, 0, 1.5, 50.0, 0])
            .unwrap();

        vio.store_vector(delta_short_id, amount_vec![0, 0, 0, 0, 0])
            .unwrap();

        vio.store_vector(delta_long_id, amount_vec![0, 0, 0, 0, 0])
            .unwrap();

        vio.store_vector(margin_id, amount_vec![0.5, 0.5, 0.5, 100.0, 0.5])
            .unwrap();

        vio.store_code(solve_quadratic_ask_id, solve_quadratic_ask().unwrap())
            .unwrap();

        let code = execute_sell_order(
            index_order_id,
            vendor_order_id,
            total_order_id,
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
            solve_quadratic_ask_id,
        );

        let order_before = vio.load_vector(index_order_id).unwrap();
        let vendor_order_before = vio.load_vector(vendor_order_id).unwrap();
        let total_order_before = vio.load_vector(total_order_id).unwrap();

        let num_registers = 22;

        let mut program = VectorVM::new(&mut vio);
        let mut stack = Stack::new(num_registers);
        let result = program.execute_with_stack(code.unwrap(), &mut stack);

        if let Err(err) = result {
            log_stack!(&stack);
            panic!("Failed to execute test: {:?}", err);
        }

        let order_after = vio.load_vector(index_order_id).unwrap();
        let vendor_order_after = vio.load_vector(vendor_order_id).unwrap();
        let total_order_after = vio.load_vector(total_order_id).unwrap();
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
        log_msg!("[in] Vendor Order = {:0.9}", vendor_order_before);
        log_msg!("[in] Total Order = {:0.9}", total_order_before);
        log_msg!("\n[in] Collateral Added = {:0.9}", collateral_added);
        log_msg!("[in] Collateral Removed = {:0.9}", collateral_removed);
        log_msg!("[in] Index Quote = {:0.9}", quote);
        log_msg!("[in] Asset Weights = {:0.9}", weigths);
        log_msg!("\n[out] Index Order = {:0.9}", order_after);
        log_msg!("[out] Vendor Order = {:0.9}", vendor_order_after);
        log_msg!("[out] Total Order = {:0.9}", total_order_after);
        log_msg!("\n[out] Index Quantities = {:0.9}", index_quantites);
        log_msg!("[out] Asset Quantities = {:0.9}", asset_quantites);
        log_msg!("\n[out] Demand Short = {:0.9}", demand_short);
        log_msg!("[out] Demand Long = {:0.9}", demand_long);
        log_msg!("\n[out] Delta Short = {:0.9}", delta_short);
        log_msg!("[out] Delta Long = {:0.9}", delta_long);

        assert_eq!(order_before.data, amount_vec![1.00, 0, 0].data);
        assert_eq!(quote.data, amount_vec![10, 10_000, 100].data);
        assert_eq!(weigths.data, amount_vec![0.1, 1, 100,].data);

        // these are exact expected fixed point decimal values as raw u128
        assert_eq!(order_after.data, amount_vec![1.0, 0.5, 4975.0].data);
        assert_eq!(index_quantites.data, amount_vec![0.5, 4975.0].data);
        assert_eq!(asset_quantites.data, amount_vec![0.05, 0.5, 50.0].data);
        assert_eq!(
            demand_long.data,
            amount_vec![0.95, 0.0, 0.5, 10.0, 0.0].data
        );
        assert_eq!(demand_short.data, amount_vec![0.0, 0.0, 0.0, 0.0, 1.0].data);
    }

    #[test]
    fn test_transfer() {
        let mut vio = test_utils::TestVectorIO::new();

        let sender_bid_id = 10001;
        let sender_ask_id = 10002;
        let receiver_bid_id = 10003;
        let transfer_amount = amount!(0.5);

        vio.store_vector(sender_bid_id, amount_vec![500, 250, 2.5])
            .unwrap();

        vio.store_vector(sender_ask_id, amount_vec![0.5, 0.5, 50])
            .unwrap();

        vio.store_vector(receiver_bid_id, amount_vec![200, 100, 1.0])
            .unwrap();

        let sender_bid_before = vio.load_vector(sender_bid_id).unwrap();
        let sender_ask_before = vio.load_vector(sender_ask_id).unwrap();
        let receiver_bid_before = vio.load_vector(receiver_bid_id).unwrap();

        let code = execute_transfer(
            sender_bid_id,
            sender_ask_id,
            receiver_bid_id,
            transfer_amount.to_u128_raw(),
        );

        let num_registers = 6;

        let mut program = VectorVM::new(&mut vio);
        let mut stack = Stack::new(num_registers);
        let result = program.execute_with_stack(code.unwrap(), &mut stack);

        if let Err(err) = result {
            log_stack!(&stack);
            panic!("Failed to execute test: {:?}", err);
        }
        let sender_bid_after = vio.load_vector(sender_bid_id).unwrap();
        let sender_ask_after = vio.load_vector(sender_ask_id).unwrap();
        let receiver_bid_after = vio.load_vector(receiver_bid_id).unwrap();

        log_msg!("\n-= Program complete =-");
        log_msg!("\n[in] Sender Bid = {:0.9}", sender_bid_before);
        log_msg!("[in] Sender Ask = {:0.9}", sender_ask_before);
        log_msg!("[in] Receiver Bid = {:0.9}", receiver_bid_before);
        log_msg!("\n[in] Transfer Amount = {:0.9}", transfer_amount);
        log_msg!("\n[out] Sender Bid = {:0.9}", sender_bid_after);
        log_msg!("[out] Sender Ask = {:0.9}", sender_ask_after);
        log_msg!("[out] Receiver Bid = {:0.9}", receiver_bid_after);
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
                )
                .unwrap(),
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
                )
                .unwrap(),
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
            )
            .unwrap(),
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
            )
            .unwrap(),
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
            )
            .unwrap(),
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
            )
            .unwrap(),
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

    #[test]
    fn test_update_rebalance() {
        let mut vio = test_utils::TestVectorIO::new();

        let total_bid_id = 10003;
        let total_ask_id = 10004;

        let asset_names_id = 1001;
        let asset_weights_id = 1002;

        let new_asset_names_id = 1003;
        let new_weights_id = 1004;

        let rebalance_asset_names_id = 120;
        let rebalance_weights_long_id = 121;
        let rebalance_weights_short_id = 122;

        let old_asset_names = label_vec![51, 53, 54];
        let new_asset_names = label_vec![52, 53, 55];
        let rebalance_asset_names = label_vec![51, 52];

        vio.store_vector(total_bid_id, amount_vec![0.0, 1000.0, 1.0])
            .unwrap();

        vio.store_vector(total_ask_id, amount_vec![0.3, 0.1, 200.0])
            .unwrap();

        vio.store_labels(
            asset_names_id,
            Labels {
                data: old_asset_names.data.clone(),
            },
        )
        .unwrap();

        vio.store_vector(asset_weights_id, amount_vec![0.1, 0.2, 1.0])
            .unwrap();

        vio.store_labels(
            new_asset_names_id,
            Labels {
                data: new_asset_names.data.clone(),
            },
        )
        .unwrap();

        vio.store_vector(new_weights_id, amount_vec![0.2, 0.2, 0.8])
            .unwrap();

        vio.store_labels(
            rebalance_asset_names_id,
            Labels {
                data: rebalance_asset_names.data.clone(),
            },
        )
        .unwrap();

        vio.store_vector(rebalance_weights_long_id, amount_vec![0.04, 0])
            .unwrap();

        vio.store_vector(rebalance_weights_short_id, amount_vec![0, 0.1])
            .unwrap();

        let code = update_rebalance(
            total_bid_id,
            total_ask_id,
            asset_names_id,
            asset_weights_id,
            new_asset_names_id,
            new_weights_id,
            rebalance_asset_names_id,
            rebalance_weights_long_id,
            rebalance_weights_short_id,
        );

        let total_bid = vio.load_vector(total_bid_id).unwrap();
        let total_ask = vio.load_vector(total_ask_id).unwrap();
        let old_weights = vio.load_vector(asset_weights_id).unwrap();
        let rebalance_weights_long_before = vio.load_vector(rebalance_weights_long_id).unwrap();
        let rebalance_weights_short_before = vio.load_vector(rebalance_weights_short_id).unwrap();

        let num_registers = 8;
        let mut program = VectorVM::new(&mut vio);
        let mut stack = Stack::new(num_registers);
        let result = program.execute_with_stack(code.unwrap(), &mut stack);

        if let Err(err) = result {
            log_stack!(&stack);
            panic!("Failed to execute test: {:?}", err);
        }

        let rebalance_weights_long_after = vio.load_vector(rebalance_weights_long_id).unwrap();
        let rebalance_weights_short_after = vio.load_vector(rebalance_weights_short_id).unwrap();
        let rebalance_asset_names_after = vio.load_labels(rebalance_asset_names_id).unwrap();

        let new_names = vio.load_labels(asset_names_id).unwrap();
        let new_weights = vio.load_vector(asset_weights_id).unwrap();

        assert_eq!(new_names.data, new_asset_names.data);

        let all_asset_names = label_vec![51, 52, 53, 54, 55];
        let old_weights = project(&old_weights, &old_asset_names, &all_asset_names);
        let new_weights = project(&new_weights, &new_asset_names, &all_asset_names);
        let rebalance_weights_long_before = project(
            &rebalance_weights_long_before,
            &rebalance_asset_names,
            &all_asset_names,
        );
        let rebalance_weights_short_before = project(
            &rebalance_weights_short_before,
            &rebalance_asset_names,
            &all_asset_names,
        );
        let rebalance_weights_long_after = project(
            &rebalance_weights_long_after,
            &rebalance_asset_names_after,
            &all_asset_names,
        );
        let rebalance_weights_short_after = project(
            &rebalance_weights_short_after,
            &rebalance_asset_names_after,
            &all_asset_names,
        );

        log_msg!("\n-= Program complete =-");

        log_msg!("\n[in] Total Bid = {:0.9}", total_bid);
        log_msg!("[in] Total Ask = {:0.9}", total_ask);

        log_msg!("\n[in] Old Asset Names = {}", old_asset_names);
        log_msg!("[in] Old Weights = {:0.9}", old_weights);

        log_msg!("\n[in] New Asset Names = {}", new_asset_names);
        log_msg!("[in] New Weights = {:0.9}", new_weights);

        log_msg!("\n[in] Rebalance Asset Names = {}", rebalance_asset_names);
        log_msg!(
            "[in] Rebalance Weights Long = {:0.9}",
            rebalance_weights_long_before
        );
        log_msg!(
            "[in] Rebalance Weights Short = {:0.9}",
            rebalance_weights_short_before
        );

        log_msg!(
            "\n[out] Rebalance Asset Names = {}",
            rebalance_asset_names_after
        );
        log_msg!(
            "[out] Rebalance Weights Long = {:0.9}",
            rebalance_weights_long_after
        );
        log_msg!(
            "[out] Rebalance Weights Short = {:0.9}",
            rebalance_weights_short_after
        );

        assert_eq!(
            rebalance_asset_names_after.data,
            label_vec![51, 52, 53, 54, 55].data
        );

        assert_eq!(
            rebalance_weights_long_after.data,
            amount_vec![0, 0.02, 0, 0, 0.48].data
        );

        assert_eq!(
            rebalance_weights_short_after.data,
            amount_vec![0.02, 0, 0, 0.6, 0].data
        )
    }

    #[test]
    fn test_execute_rebalance() {
        let mut vio = test_utils::TestVectorIO::new();

        let market_asset_names_id = 101;

        let supply_long_id = 102;
        let supply_short_id = 103;
        let demand_long_id = 104;
        let demand_short_id = 105;
        let delta_long_id = 106;
        let delta_short_id = 107;

        let margin_id = 108;
        let asset_liquidity_id = 109;

        let executed_assets_long_id = 1;
        let executed_assets_short_id = 2;

        let rebalance_asset_names_id = 120;
        let rebalance_weights_long_id = 121;
        let rebalance_weights_short_id = 122;

        let rebalance_asset_names = label_vec![51, 53, 54];
        let market_asset_names = label_vec![51, 52, 53, 54, 55];

        let capacity_factor = amount!(0.50);

        vio.store_labels(
            rebalance_asset_names_id,
            Labels {
                data: rebalance_asset_names.data.clone(),
            },
        )
        .unwrap();

        vio.store_labels(
            market_asset_names_id,
            Labels {
                data: market_asset_names.data.clone(),
            },
        )
        .unwrap();

        vio.store_vector(rebalance_weights_long_id, amount_vec![0.1, 0, 0.05])
            .unwrap();

        vio.store_vector(rebalance_weights_short_id, amount_vec![0, 0.04, 0])
            .unwrap();

        vio.store_vector(supply_long_id, amount_vec![0.1, 0.2, 0.1, 0.2, 0])
            .unwrap();

        vio.store_vector(supply_short_id, amount_vec![0, 0, 0, 0, 0.1])
            .unwrap();

        vio.store_vector(demand_long_id, amount_vec![0.1, 0.2, 0, 0.4, 0])
            .unwrap();

        vio.store_vector(demand_short_id, amount_vec![0, 0, 0.5, 0, 0])
            .unwrap();

        vio.store_vector(delta_long_id, amount_vec![0, 0, 0, 0, 0])
            .unwrap();

        vio.store_vector(delta_short_id, amount_vec![0, 0, 0, 0, 0])
            .unwrap();

        vio.store_vector(margin_id, amount_vec![0.1, 1, 1, 1, 1])
            .unwrap();

        vio.store_vector(asset_liquidity_id, amount_vec![1, 1, 1, 0.02, 1])
            .unwrap();

        let margin = vio.load_vector(margin_id).unwrap();
        let liquidity = vio.load_vector(asset_liquidity_id).unwrap();

        let demand_short_before = vio.load_vector(demand_short_id).unwrap();
        let demand_long_before = vio.load_vector(demand_long_id).unwrap();

        let rebalance_weights_long_before = vio.load_vector(rebalance_weights_long_id).unwrap();
        let rebalance_weights_short_before = vio.load_vector(rebalance_weights_short_id).unwrap();

        let code = execute_rebalance(
            capacity_factor.to_u128_raw(),
            executed_assets_long_id,
            executed_assets_short_id,
            rebalance_asset_names_id,
            rebalance_weights_long_id,
            rebalance_weights_short_id,
            market_asset_names_id,
            supply_long_id,
            supply_short_id,
            demand_long_id,
            demand_short_id,
            delta_long_id,
            delta_short_id,
            margin_id,
            asset_liquidity_id,
        );

        let num_registers = 12;
        let mut program = VectorVM::new(&mut vio);
        let mut stack = Stack::new(num_registers);
        let result = program.execute_with_stack(code.unwrap(), &mut stack);

        if let Err(err) = result {
            log_stack!(&stack);
            panic!("Failed to execute test: {:?}", err);
        }

        let rebalance_weights_long_after = vio.load_vector(rebalance_weights_long_id).unwrap();
        let rebalance_weights_short_after = vio.load_vector(rebalance_weights_short_id).unwrap();

        let demand_short_after = vio.load_vector(demand_short_id).unwrap();
        let demand_long_after = vio.load_vector(demand_long_id).unwrap();

        let delta_short = vio.load_vector(delta_short_id).unwrap();
        let delta_long = vio.load_vector(delta_long_id).unwrap();

        let executed_asset_long = vio.load_vector(executed_assets_long_id).unwrap();
        let executed_asset_short = vio.load_vector(executed_assets_short_id).unwrap();

        log_msg!("\n-= Program complete =-");

        log_msg!("\n[in] Margin    = {:0.9}", margin);
        log_msg!("[in] Liquidity = {:0.9}", liquidity);

        log_msg!("\n[in] Capacity Factor = {:0.9}", capacity_factor);

        log_msg!(
            "\n[in] Rebalance Asset Names = {:0.9}",
            rebalance_asset_names
        );

        log_msg!(
            "\n[in] Rebalance Weights Long  = {:0.9}",
            rebalance_weights_long_before
        );
        log_msg!(
            "[in] Rebalance Weights Short = {:0.9}",
            rebalance_weights_short_before
        );

        log_msg!("\n[in] Demand Long = {:0.9}", demand_long_before);
        log_msg!("[in] Demand Short= {:0.9}", demand_short_before);

        log_msg!("\n[out] Demand Long  = {:0.9}", demand_long_after);
        log_msg!("[out] Demand Short = {:0.9}", demand_short_after);

        log_msg!("\n[out] Delta Long  = {:0.9}", delta_long);
        log_msg!("[out] Delta Short = {:0.9}", delta_short);

        log_msg!("\n[out] Executed Long  = {:0.9}", executed_asset_long);
        log_msg!("[out] Executed Short = {:0.9}", executed_asset_short);

        log_msg!(
            "\n[out] Rebalance Weights Long  = {:0.9}",
            rebalance_weights_long_after
        );
        log_msg!(
            "[out] Rebalance Weights Short = {:0.9}",
            rebalance_weights_short_after
        );

        assert_eq!(executed_asset_long.data, amount_vec![0.05, 0, 0.01].data);

        assert_eq!(executed_asset_short.data, amount_vec![0, 0.04, 0].data);

        assert_eq!(
            rebalance_weights_long_after.data,
            amount_vec![0.05, 0, 0.04].data
        );
        assert_eq!(
            rebalance_weights_short_after.data,
            amount_vec![0, 0, 0].data
        );

        assert_eq!(
            demand_long_after.data,
            amount_vec![0.15, 0.2, 0, 0.41, 0].data
        );

        assert_eq!(demand_short_after.data, amount_vec![0, 0, 0.54, 0, 0].data);
    }
}
