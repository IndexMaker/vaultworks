# White Papers

1. [Technical Brief - Systematic Risk Controls And Execution Framework](/docs/Technical%20Brief%20-%20Systematic%20Risk%20Controls%20And%20Execution%20Framework.pdf)

1. [Gas-Efficient Vector Processing for On-Chain Index Order Execution](/docs/Gas-Efficient%20Vector%20Processing%20for%20On-Chain%20Index%20Order%20Execution.pdf)

1. [VIL VM Technical Specification](/docs/VIL%20VM%20Technical%20Specification.pdf)

1. [Aligning Market and Index Vectors](/docs/Aligning%20Market%20and%20Index%20Vectors.pdf)

### Note on The White-Papers and The Present Architecture

The white-paper mentions *Daxos* and *Devil* smart-contracts. These were changed during refactoring into multiple smart-contracts, where *Daxos* became *Castle* with *NPCs* *(such as Factor, Banker, Guildmaster)*, and *Devil* became *Clerk Chamber* with *Clerk* and *Abacus*.

The `Abacus` is a smart-contract, while the actual *Vector IL VM* is implemented by `abacus-runtime` library.
The `Clerk` is managing *Clerk Chamber* and he stores vectors and performs *VIL* programs execution on `Abacus`.

The `Castle` is an implementation of the non-standard *Diamond* pattern with **built-in** *ACL*.
The *NPCs* of the `Castle` are smart-contracts, which realise ***only*** business logic, while all numerical data is always stored securely in *Clerk Chamber*,
i.e. there is ***absolutely no numerical data*** outside *Clerks Chamber*, and that data can only be manipulated via *VIL* programs.

This design guarantees ***strict*** separation of conerns via delegation of responsibilities.

The `Castle` is responsible for protecting functions using ***RBAC*** *(Role Based Access Control)*, and hosts *NPCs*:

- The `Factor` is responisble for providing trading functions to *Keeper* off-chain service.
- The `Banker` is responsible for providing supply accounting functions to *Vendor* off-chain service.
- The `Guildmaster` is responsible for providing issuance & governance functions to *Index Issuers*.
- The `Constable` is responsible for creating and managing access to the functions in the `Castle`

The *Clerk Chanber* is a consolidation of two smart-contracts:
- The `Clerk` is responsible for providing vector storage functions to *NPCs* of the `Castle`.
- The `Abacus` is responsible for providing smart-contract enrty point to *Vector IL VM* runtime.

The users, or off-chain services *(Keeper, Vendor)* ***never*** interact directly with any other smart-contracts than ***Gate to Castle***, which
is a proxy to `Castle` smart-contract. The perform all operations making calls on that `Gate` address
using the methods of `IFactor`, `IBanker`, `IGuildmaster` interfaces.

Roles in the *Castle*:
- `Castle.ADMIN_ROLE` - allows granting roles, appointing and calling `Constable` methods.
- `Castle.ISSUER_ROLE` - allows interaction via `IGuildmaster` interface.
- `Castle.KEEPER_ROLE` - allows interaction via `IFactor` interface.
- `Castle.VENDOR_ROLE` - allows interaction via `IBanker` interface.

Storage slots of the *Castle*:
- `Castle.STORAGE_SLOT` - stores `Castle` struct.
- `Keep.STORAGE_SLOT` - stores `Keep` struct.

Storage slots of the *Clerk Chamber*:
- `Clerk.STORAGE_SLOT` - stores `ClerkStorage` struct.

The `Castle` is accessed via `Gate`, which allows for upgrading *Castle* via `UUPS/ERC-1967` pattern.
This gives opportunity to upgrade RBAC mechanism as well as change admin role to new version, which
would automatically revoke all current admin access. Such upgrade would need to be carefuly crafted.

The `Constable` role is to configure the functions and required roles in the `Castle`. 
An upgrade of the functions and roles can be done by appointing new `Constable`.

The *Clerk Chamber* is accessed via `Gate`, which allows upgrading `Clerk` and `Abacus` to newer version
witout loosing vector data.


