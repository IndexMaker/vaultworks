use alloc::string::String;
use alloy_primitives::U8;
use alloy_sol_types::sol;

pub trait IERC20Metadata {
    fn name(&self) -> String;
    fn symbol(&self) -> String;
    fn decimals(&self) -> U8;
}

sol! {
    interface ICastle  {
        event ProtectedFunctionsCreated(address contract_address, bytes4[] function_selectors);

        event PublicFunctionsCreated(address contract_address, bytes4[] function_selectors);

        event FunctionsRemoved(bytes4[] function_selectors);

        event RoleGranted(bytes32 role, address assignee_address);

        event RoleRevoked(bytes32 role, address assignee_address);

        event RoleRenounced(bytes32 role, address assignee_address);

        event RoleDeleted(bytes32 role);

        function createPublicFunctions(address contract_address, bytes4[] memory function_selectors) external;

        function createProtectedFunctions(address contract_address, bytes4[] memory function_selectors, bytes32 required_role) external;

        function removeFunctions(bytes4[] memory function_selectors) external;

        function getFunctionDelegates(bytes4[] memory function_selectors) external view returns (address[] memory);

        function hasRole(bytes32 role, address attendee) external returns (bool);

        function grantRole(bytes32 role, address attendee) external;

        function revokeRole(bytes32 role, address attendee) external;

        function renounceRole(bytes32 role, address attendee) external;

        function deleteRole(bytes32 role) external;

        function getAdminRole() external view returns (bytes32);

        function getAssignedRoles(address attendee) external view returns (bytes32[] memory);

        function getRoleAssignees(bytes32 role) external view returns (address[] memory);
    }

}
