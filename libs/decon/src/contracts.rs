use ethers::contract::abigen;

abigen!(
    Granary,
    r"[
        function initialize(address owner, address clerk) external
        function store(uint128 id, uint8[] memory data) external
        function load(uint128 id) external view returns (uint8[] memory)
        function execute(uint8[] memory code, uint128 num_registry) external returns (uint8[] memory)
    ]"
);

abigen!(
    Banker,
    r"[
        function submitAssets(uint128 vendor_id, uint8[] memory market_asset_names) external
        function submitMargin(uint128 vendor_id, uint8[] memory asset_names, uint8[] memory asset_margin) external
        function submitSupply(uint128 vendor_id, uint8[] memory asset_names, uint8[] memory asset_quantities_short, uint8[] memory asset_quantities_long) external
    ]"
);

abigen!(
    Factor,
    r"[
        function submitMarketData(uint128 vendor_id, uint8[] memory asset_names, uint8[] memory asset_liquidity, uint8[] memory asset_prices, uint8[] memory asset_slopes) external
        function updateIndexQuote(uint128 vendor_id, uint128 index) external
        function updateMultipleIndexQuotes( uint128 vendor_id, uint128[] memory indexes) external
        function submitBuyOrder(uint128 vendor_id, uint128 index, uint128 collateral_added, uint128 collateral_removed, uint128 max_order_size, uint8[] memory asset_contribution_fractions) external returns (uint8[] memory, uint8[] memory, uint8[] memory)
    ]"
);

abigen!(
    Guildmaster,
    r"[
        function submitIndex( uint128 index, uint8[] memory asset_names, uint8[] memory asset_weights, uint8[] memory info) external
        function submitVote( uint128 index, uint8[] memory vote) external
    ]"
);
