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


# End to End Flow (Scenario 5.)

## How to setup

1. Start *Nitro Dev Node*

2. Setup environment

Need to set private key:
```
export DEPLOY_PRIVATE_KEY=`cat some-test-key`
```

Address can be obtained using:
```
export DEPLOYER_ADDRESS=`cast wallet address DEPLOY_PRIVATE_KEY`
```

Also RPC URL is needed for running scenarios:
```
export RPC_URL="http://localhost:8547"
```

3. Deploy whole *Castle*

```
./scripts/castle.sh
```

Once deployment completes at the end similar information will show:
```
---------------------------
=== Deployment Complete ===
---------------------------
Castle Target: 0x0444764a212240b69d3ad81b9a77f34945d1b228
Granary Target: 0x9ae8a390121ba71545e9923b333d60e7e3ccd3bd
---------------------------
```

Copy address of `Castle Target` and export as `$CASTLE` variable, e.g.

```
export CASTLE="0x0444764a212240b69d3ad81b9a77f34945d1b228"
```


4. Set roles:

Set these three roles:
```
./scripts/roles.sh grant $CASTLE "Castle.ISSUER_ROLE" $DEPLOYER_ADDRESS
./scripts/roles.sh grant $CASTLE "Castle.KEEPER_ROLE" $DEPLOYER_ADDRESS
./scripts/roles.sh grant $CASTLE "Castle.VENDOR_ROLE" $DEPLOYER_ADDRESS
```

5. Add *Vault* to *Worksman* free-list

Currently there is no *Vault* contract, we can use *Treasury* instead.

```
./scripts/treasury.sh full
```

Script at the end will show similar output:
```
=== FULL DEPLOYMENT COMPLETE ===
Logic: 0x0fb6856c36c25e01190d6a8f2ebbe28aca05a341
Gate : 0x9b2db8135222d7b05aea29b54ae0317e8640d6b0
```

Copy address of `Gate` and in next command, which will look like:
```
./scripts/send.sh $CASTLE "addVault(address)" 0x9b2db8135222d7b05aea29b54ae0317e8640d6b0
```

This adds that *Gate* to *Workman's* free-list, and then when *Guildmaster*
requests to build a *Vault* *Worksman* will pick next from that free-list.

6. Run Scenario 5.

Congratulations, you have set-up the eonvironment to run once-and-only-once Scenario 5.

```
cargo run -p scenarios -- --rpc-url $RPC_URL --private-key $DEPLOY_PRIVATE_KEY --castle-address $CASTLE -s scenario5
```

This will run Scenario 5. which:

- Create Vendor's account
- Submit list of assets traded by Vendor
- Submit trading margin for that Vendor (max open position at any time)
- Submit supply from Vendor
- Create new Index w/ asset weights
- Simulate voting for Index
- Submit Market Data from Vendor
- Update Index pricing (quote)
- Submit Buy order
