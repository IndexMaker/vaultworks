# Performance

## Scenario 1. *(Small VIL Program)* - Sanity Check

We used small VIL program to test that it will be correctly executed by *DeVIL* deployed to *Nitro-Dev-Node*.

## Scenario 2. *(Index Order Execution)* - Sanity Check

We ran small Index order execution scenario with known expected outcome to verify that this program runs correctly on *DeVIL* deployed to *Nitro-Dev-Node*.

## Scenario 3. *(Update Inventory & Quote)*

We ran small Inventory & Quote update scenario with known expected outcome to verify that this program runs correctly on *DeVIL* deployed to *Nitro-Dev-Node*.

## Scenario 4. *(Index Order Execution)* on Nitro-Dev-Node - Performance Test

We ran *Scenario 4. (Index Order Execution)* is several configurations, and we measured cost of running `execute_buy_order` *VIL* vector program on *Nitro-Dev-Node*.

The [`execute_buy_order`](../../libs//icore/src/vil/execute_buy_order.rs) is a complex VIL program, which:

- loads vectors from blockchain
- solves quadratic equation to compute possible Index quantity for given amount of collateral, and Index price dependent on volume (slope).
- caps possible Index quantity with current Index capacity
- multiplies capped Index quantity by asset weights to compute orders for individual underlying assets
- matches individual asset orders against inventory, first computing new demand vector, and then by updating delta vector
- computes final executed Index quantity and unexecuted quantity remaining for given collateral amount
- stores updated vectors on blockchain

### Results

| Number of Inventory Assets | Number of Index Assets | Gas Used | ETH Cost | USD Cost (¢) |
|---|---|---|---|---|
|100|  5|2'198'896|0.00021989|62.06|62.00|-0.06|
|100| 20|2'285'490|0.00022855|64.51|68.00|3.49|
|100| 50|2'886'139|0.00028861|81.46|80.00|-1.46|
|100| 80|2'865'352|0.00028654|80.87|92.00|11.13|
|100|100|3'457'837|0.00034578|97.59|100.00|2.41|
|150| 50|3'371'702|0.00033717|95.16|95.00|-0.16|


### Conclusion

The results suggest that the cost can be roughly estimated with:
- ***¢30*** fixed charge, 
- ***¢0.4*** per Index Asset, and
- ***¢0.3*** per Inventory Asset.
