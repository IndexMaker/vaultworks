use alloy_sol_types::sol;

sol! {
    interface ICastle  {
        function initialize(address castle, address admin) external;

        function appointConstable(address constable) external;

        function getFunctionDelegates(bytes4[] memory fun_selectors) external view returns (address[] memory);

        function hasRole(bytes32 role, address attendee) external returns (bool);

        function grantRole(bytes32 role, address attendee) external;

        function revokeRole(bytes32 role, address attendee) external;

        function renounceRole(bytes32 role, address attendee) external;

        function deleteRole(bytes32 role) external returns (bool);

        function getAdminRole() external view returns (bytes32);

        function getRoleAssigneeCount(bytes32 role) external view returns (uint256);

        function getRoleAssignees(bytes32 role, uint256 start_from, uint256 max_len) external view returns (address[] memory);
        
        // -- Events --

        event ProtectedFunctionsCreated(address contract_address, bytes4[] function_selectors);

        event PublicFunctionsCreated(address contract_address, bytes4[] function_selectors);

        event FunctionsRemoved(bytes4[] function_selectors);

        event RoleGranted(bytes32 role, address assignee_address);

        event RoleRevoked(bytes32 role, address assignee_address);

        event RoleRenounced(bytes32 role, address assignee_address);

        event RoleDeleted(bytes32 role);

    }

}
