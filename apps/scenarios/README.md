# Performance

## Scenario 1. *(Small VIL Program)* - Sanity Check

We used small VIL program to test that it will be correctly executed by *DeVIL* deployed to *Nitro-Dev-Node*.

## Scenario 2. *(Index Order Execution)* - Sanity Check

We ran small Index order execution scenario with known expected outcome to verify that this program runs correctly on *DeVIL* deployed to *Nitro-Dev-Node*.

## Scenario 3. *(Update Inventory & Quote)*

We ran small Inventory & Quote update scenario with known expected outcome to verify that this program runs correctly on *DeVIL* deployed to *Nitro-Dev-Node*.

## Scenario 4. *(Index Order Execution)* on Nitro-Dev-Node - Performance Test

We ran *Scenario 4. (Index Order Execution)* in several configurations, and we measured cost of running `execute_buy_order` *VIL* vector program on *Nitro-Dev-Node*.

The [`execute_buy_order`](../../libs//icore/src/vil/execute_buy_order.rs) is a complex VIL program, which:

- loads vectors from blockchain
- solves quadratic equation to compute possible Index quantity for given amount of collateral, and Index price dependent on volume (slope).
- caps possible Index quantity with current Index capacity
- multiplies capped Index quantity by asset weights to compute orders for individual underlying assets
- matches individual asset orders against inventory, first computing new demand vector, and then by updating delta vector
- computes final executed Index quantity and unexecuted quantity remaining for given collateral amount
- stores updated vectors on blockchain

### Results

| Math Version | Number of Inventory Assets | Number of Index Assets | Gas Used | ETH Cost | USD Cost (¢) |
|-----|---|---|---------|----------|------|
| 1.0 |100|  5|2'198'896|0.00021989| 62.06|
| 1.0 |100| 20|2'285'490|0.00022855| 64.51|
| 1.0 |100| 50|2'886'139|0.00028861| 81.46|
| 1.0 |100| 80|2'865'352|0.00028654| 80.87|
| 1.0 |100|100|3'457'837|0.00034578| 97.59|
| 1.0 |150| 50|3'371'702|0.00033717| 95.16|
| 1.1.0 |150| 50|3'843'558|0.00038436|132.37|
| 1.1.1 |150| 50|3'378'136|0.00033781|121.61|
| 1.1.2 |150| 50|3'381'746|0.00033817|121.74|

#### Math Versions

- **1.0** Initial version
- **1.1** Execute Buy w/ Capacity Limit

See explanation [here...](../../docs/Vectorized%20Index%20Buy%20Order%20Execution%20Algorithm.pdf)


### Conclusion

The results suggest that the cost can be roughly estimated with:
- ***¢30*** fixed charge, 
- ***¢0.4*** per Index Asset, and
- ***¢0.3*** per Inventory Asset.
