use alloy_sol_types::sol;

sol!{
    interface IConstable  {
        function acceptAppointment(address constable) external;

        function appointBanker(address banker) external;

        function appointFactor(address factor) external;

        function appointGuildmaster(address guildmaster) external;

        function appointScribe(address scribe) external;

        function appointWorksman(address worksman) external;
        
        function appointSteward(address steward) external;

        function appointClerk(address clerk) external;

        function getIssuerRole() external view returns (bytes32);

        function getVendorRole() external view returns (bytes32);

        function getKeeperRole() external view returns (bytes32);

        function getVaultRole() external view returns (bytes32);

        function getMaintainerRole() external view returns (bytes32);

        function getVersion() external view returns (uint32);
    }
}