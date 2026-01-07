use ethers::contract::abigen;

abigen!(
    Clerk,
    r"[
        function fetchVector(uint128 id) external view returns (bytes memory)
        function updateRecords(bytes calldata code, uint128 num_registry) external
    ]"
);

abigen!(
    Banker,
    r"[
        function submitAssets(uint128 vendor_id, bytes calldata market_asset_names) external

        function submitMargin(uint128 vendor_id, bytes calldata asset_names, bytes calldata asset_margin) external

        function submitSupply(uint128 vendor_id, bytes calldata asset_names, bytes calldata asset_quantities_short, bytes calldata asset_quantities_long) external

        function getVendorAssets(uint128 vendor_id) external returns (bytes memory)

        function getVendorMargin(uint128 vendor_id) external returns (bytes memory)

        function getVendorSupply(uint128 vendor_id) external returns (bytes memory, bytes memory)

        function getVendorDemand(uint128 vendor_id) external returns (bytes memory, bytes memory)

        function getVendorDelta(uint128 vendor_id) external returns (bytes memory, bytes memory)
    ]"
);

abigen!(
    Factor,
    r"[
        function submitMarketData(uint128 vendor_id, bytes calldata asset_names, bytes calldata asset_liquidity, bytes calldata asset_prices, bytes calldata asset_slopes) external

        function submitBuyOrder(uint128 vendor_id, uint128 index_id, address trader_address, uint128 collateral_added, uint128 collateral_removed, uint128 max_order_size, bytes calldata asset_contribution_fractions) external returns (bytes memory, bytes memory, bytes memory)

        function submitSellOrder(uint128 vendor_id, uint128 index_id, address trader_address, uint128 collateral_added, uint128 collateral_removed, uint128 max_order_size, bytes calldata asset_contribution_fractions) external returns (uint8[] memory, uint8[] memory, uint8[] memory)

        function submitTransfer(uint128 index_id, address sender, address receiver, uint128 amount) external

        function getMarketData(uint128 vendor_id) external view returns (bytes memory, bytes memory, bytes memory)

        function getIndexAssetsCount(uint128 index_id) external view returns (uint128)

        function getIndexAssets(uint128 index_id) external view returns (bytes memory)

        function getIndexWeights(uint128 index_id) external view returns (bytes memory)

        function getIndexQuote(uint128 index_id, uint128 vendor_id) external view returns (bytes memory)

        function getTraderOrder(uint128 index_id, address trader) external view returns (bytes memory)

        function getTraderCount(uint128 index_id) external view returns (uint128)

        function getTraderAt(uint128 index_id, uint128 offset) external view returns (address)

        function getVendorOrder(uint128 index_id, uint128 vendor_id) external view returns (bytes memory)

        function getVendorCount(uint128 index_id) external view returns (uint128)

        function getVendorAt(uint128 index_id, uint128 offset) external view returns (uint128)

        function getTotalOrder(uint128 index_id) external view returns (bytes memory)
    ]"
);

abigen!(
    Guildmaster,
    r"[
        function submitIndex(uint128 index, bytes calldata asset_names, bytes calldata asset_weights, bytes calldata info) external

        function submitVote(uint128 index, bytes calldata vote) external

        function updateIndexQuote(uint128 vendor_id, uint128 index_id) external

        function updateMultipleIndexQuotes(uint128 vendor_id, uint128[] memory index_ids) external
    ]"
);
