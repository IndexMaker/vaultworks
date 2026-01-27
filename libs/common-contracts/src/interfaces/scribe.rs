use alloy_sol_types::sol;

sol! {
    interface IScribe  {
        function verifySignature(bytes calldata public_key, bytes calldata signature) external returns (bool);
    }
}