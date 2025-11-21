use ethers::contract::abigen;

abigen!(
    Devil,
    r"[
        function setup(address owner) external
        function submit(uint128 id, uint8[] memory data) external
        function get(uint128 id) external view returns (uint8[] memory)
        function execute(uint8[] memory code, uint128 num_registry) external
    ]"
);
