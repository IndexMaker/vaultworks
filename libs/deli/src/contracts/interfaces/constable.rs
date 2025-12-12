use alloy_sol_types::sol;

sol!{
    interface IConstable  {
        function acceptAppointment(address castle) external;
        function appointWorksman(address worksman) external;
        function appointScribe(address scribe) external;
        function castRoles(address guildmaster, address banker, address factor, address gate_to_granary) external;
    }
}