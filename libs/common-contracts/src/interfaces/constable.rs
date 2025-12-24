use alloy_sol_types::sol;

sol!{
    interface IConstable  {
        function acceptAppointment(address constable) external;

        function appointBanker(address banker) external;

        function appointFactor(address factor) external;

        function appointGuildmaster(address guildmaster) external;

        function appointScribe(address scribe) external;

        function appointWorksman(address worksman) external;

        function appendClerkChamber(address gate_to_clerk_chamber) external;

        function getIssuerRole() external view returns (bytes32);

        function getVendorRole() external view returns (bytes32);

        function getKeeperRole() external view returns (bytes32);
    }
}