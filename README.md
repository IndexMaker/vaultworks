<img src="./docs/VaultWorks.jpg">

# VaultWorks

### High-fidelity decentralized engine for the synthesis and settlement of institutional-grade financial indices on Arbitrum.

**VaultWorks** provides a strong core for the next generation of asset management. By replacing legacy financial intermediation with a system of deterministic mathematical laws, we enable the creation and settlement of indices with absolute precision and cryptographic sovereignty.

Powered by *Arbitrum Stylus (Rust)*, VaultWorks achieves ***near-native performance***, allowing ***complex vector-based financial products*** to operate at the speed and scale required by global capital markets.

---

## 🏰 The Architecture of the Castle and Vaults

The ecosystem is structured as a **Castle**—a high-integrity environment—where the protocol itself autonomously guards the ***Access Control List (ACL)*** and secures the perimeter against unauthorized execution.

### The Inhabitants of the Castle

The system logic is managed by dedicated NPCs (Non-Player Characters), each serving as a specialized inhabitant of the Castle to maintain the integrity of the realm.

| Role | Focus | Function |
|---|---|---|
| **Factor** | **Order Execution** | Orchestrates the lifecycle of buy and sell orders, managing the flow of collateral between traders and operators. |
| **Banker** | **Market Equilibrium** | Ingests vendor's supply, market data, asset margins, and liquidity slopes to synchronize precise quotes across multiple indices. |
| **Clerk** | **Registry Maintenance** | Diligently manages the formal update of protocol records and the registry of system components. |
| **Steward** | **State Observability** | Serves as the protocol’s eyes and ears, distilling complex vectors of market supply, vendor deltas, and real-time index quotes. |
| **Guildmaster** | **Index Governance** | Presides over the creation and modification of indices, officiating asset weights and the democratic submission of votes. |
| **Constable** | **System Authority** | Acts as the foundational architect, appointing sovereign roles and defining the permissions for Issuers, Vendors, and Keepers. |
| **Worksman** | **Vault Fabrication** | Maintains the architectural prototypes and executes the deployment of sovereign Vault instances for the system. |
| **Scribe** | **Truth Verification** | Operates as the gatekeeper of authenticity by performing cryptographic verification of signatures for protocol data. |

### The Sovereign Vault & Its Facets

| Facet | Focus | Function |
|---|---|---|
| **Vault** | **Identity & ERC-20** | The base vessel for the Index; implements the standard **ERC-20** interface for tokenized identity while managing custody and metadata. |
| **Vault Native** | **Valuation Engine** | An implementation inspired by ERC-4626, but optimized for the protocol's high-dimensional vector math. Handles asset valuation and quote conversions.. |
| **Vault Orders** | **Order Lifecycle** | Inspired by *ERC-7540*; manages the asynchronous placement and processing of buy/sell orders. |
| **Vault Claims** | **Settlement** | Inspired by *ERC-7540*; manages the claimable state and final settlement of processed acquisitions and disposals. |

---

## 📜 The Core: Clerk and Abacus Runtime

**VaultWorks** separates computational execution from top-level business logic to ensure a disciplined financial state.

* **The Vaults:** They implement ***ERC-20*** and inspired by *ERC-7540 / ERC-4626*. High-security attachments that house ***Index definitions***, asset weights, and user orders. They are built by the Worksman on command of the Guildmaster.
* **The Gate:** An implementation of the ***Proxy (ERC-1967)*** pattern providing secure, structured access points. The architecture utilizes individual Gates for each **Vault**.
* **The Clerk:** The Clerk smart contract orchestrates the recording of the final state in stored vectors and triggers the execution of deterministic financial logic.
* **Abacus-Runtime:** The foundational library powering the computational engine. It performs high-velocity, zero-copy mathematics, bypassing the gas overhead of standard EVM implementations. Also known as the ***VIL VM: Decentralized Vector Intermediate Language Virtual Machine***.
* **Abacus-Formulas:** The dedicated library containing the *Vector Programs* and mathematical definitions used by the protocol to align market and index vectors.

---

## ⚡ Designed For Performance

**VaultWorks** is engineered for the highest standards of accuracy, transparency, and efficiency.

1. **Speed & Precision:** Data is represented as ***vectors of 128-bit decimals with 256-bit computational precision***, ensuring exacting accuracy while maintaining WASM binaries under the 24KiB limit.
2. **Gas Efficiency:** By utilizing a custom-built VIL VM, we eliminate the overhead of standard blockchain `SLOAD`/`SSTORE` operations, ensuring the ***core remains strong under high-frequency load***.
3. **Financial Accuracy:** Trading is driven by maximizing available margin (reducing liability through incremental pegging) using the math of equity, assets, and liability, represented here as ***Supply***, ***Demand***, and ***Delta*** vectors.

---

## 📖 White Papers

- [Technical Brief - Systematic Risk Controls And Execution Framework](/docs/Technical%20Brief%20-%20Systematic%20Risk%20Controls%20And%20Execution%20Framework.pdf)

- [Gas-Efficient Vector Processing for On-Chain Index Order Execution](/docs/Gas-Efficient%20Vector%20Processing%20for%20On-Chain%20Index%20Order%20Execution.pdf)

- [VIL VM Technical Specification](/docs/VIL%20VM%20Technical%20Specification.pdf)

- [Aligning Market and Index Vectors](/docs/Aligning%20Market%20and%20Index%20Vectors.pdf)

---

## 🏁 Developer Guide

To **start building** with us and read **[more here](DEVELOPER.md)**.

Read [more here](/libs/abacus-runtime/README.md) to learn about *Vector Intermediate Language Virtual Machine (VIL VM)*.
