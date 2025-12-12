use alloy_sol_types::sol;

sol! {
    interface IScribe  {
        function acceptAppointment(address castle) external;

        function verifySignature(uint8[] memory data) external returns (bool);
    }
}