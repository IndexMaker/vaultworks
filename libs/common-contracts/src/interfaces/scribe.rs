use alloy_sol_types::sol;

sol! {
    interface IScribe  {
        function acceptAppointment(address scribe) external;

        function verifySignature(bytes calldata data) external returns (bool);
    }
}