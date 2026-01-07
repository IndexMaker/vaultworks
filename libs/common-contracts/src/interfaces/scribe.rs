use alloy_sol_types::sol;

sol! {
    interface IScribe  {
        function verifySignature(bytes calldata data) external returns (bool);
    }
}