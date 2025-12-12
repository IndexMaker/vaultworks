use alloy_sol_types::sol;

sol!{
    interface IConstable  {
        function acceptAppointment(address castle) external;
        function castRoles(address castle, address guildmaster, address banker, address factor, address gate_to_granary) external;
    }
}