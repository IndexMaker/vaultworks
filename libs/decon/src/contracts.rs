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
