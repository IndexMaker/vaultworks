# Developer End-to-End Guide

## Setting-Up ***Castle** & **Vault***

### Step 1. Start *Nitro Dev Node*

Follow [instructions](https://github.com/OffchainLabs/nitro-devnode) on *Nitro Dev Node* project page.

### Step 2. Setup environment

Need to set private key:
```bash
export DEPLOY_PRIVATE_KEY=`cat some-test-key`
```

Address can be obtained using:
```bash
export DEPLOYER_ADDRESS=`cast wallet address $DEPLOY_PRIVATE_KEY`
```

Also RPC URL is needed for running scenarios:
```bash
export RPC_URL="http://localhost:8547"
```

### Step 3. Deploy ***Castle** & **NPCs** (Diamond Facets)*

First ensure you've built latest version of the codebase:
```bash
./scripts/check-all.sh
```

Now we can deploy a *Castle* like this:
```bash
./scripts/castle.sh --no-gates
```

Once deployment completes at the end similar information will show:
```
======================================================
                Deployment Complete                   
------------------------------------------------------
  * Castle Target:    0x7696e37e86b993ac1ce27feed48fa154cb8b2eda

======================================================
               Diamond Configuration                  
------------------------------------------------------
 Constable:           0x1a3ef0413fde0bf110a363f25c3fe6b527f3a8d4
 Banker:              0xcb593e5f96363a4919b583f07fe45880a1daf94e
 Factor:              0x534465d16b43cb0e0f5277357df77b0006940c95
 Steward:             0x9e607dbb3e6d7458b7570d1b2f6ceb96e597acc2
 Guildmaster:         0xa3bad834e6507566b4d43b0a449e1710b269aa26
 Clerk:               0xb81d32e78506aade6b7823991e2474ddf33c0c3b
 Scribe:              0xcdc02720da9846ca857c34985714e5aa9570ff53
 Worksman:            0x6cf4a18ac8efd6b0b99d3200c4fb9609dd60d4b3
======================================================
```

Copy address of `Castle Target` and export as `$CASTLE` variable, e.g.

```
export CASTLE="0x0444764a212240b69d3ad81b9a77f34945d1b228"
```


### Step 4. Deploy ***Vault*** prototype for ***Worksman***

We need to deploy some *Vault* contract to populate *Worksman* free-list, and we'll use *Vault-Native* option:

```bash
./scripts/vault.sh full $CASTLE
```

Script at the end will show similar output:
```
======================================================
                Deployment Complete                   
------------------------------------------------------
  * Vault Gate:           0xca8c30b482efa1eb0845f7bb00f83825532de928

======================================================
               Diamond Configuration                  
------------------------------------------------------
 Vault Implementation:    0x11ec9349b3c2dedfd2b2916125ee267574c93bf6
 Vault Native:            0x38f3d93349c5e72f8ab4f8fa5785cf680b497f37
 Vault Native Orders:     0x3bee4d202b6eb7fd4f0f7ab4ca0c3c81af619a6a
 Vault Native Claims:     0x514adac2d6baf50b1c349658848d76a9a6ff9484
======================================================
```

Copy address of the `Vault Gate` and the next command set *Vault* as prototype for *Worksman*:
```bash
./scripts/send.sh $CASTLE "setVaultPrototype(address)" 0xca8c30b482efa1eb0845f7bb00f83825532de928
```

This sets that *Prototype Vault* for *Workman's*, and then when *Guildmaster*
requests to build a *Vault* *Worksman* will deploy new *Vault* cloning configuration
from that prototype.


### Step 5. Setup *Vendor-Keeper*

- ***Vendor*** *is* responsible for supplying underlying assets.
- ***Keeper*** *is* responsible for processing large *Index* orders.

In development we can use ***Conveyor***, which implements both roles plus ***Issuer*** role. The *Conveyor* a minimalist *Vendor-Keeper (+Issuer)*  application allowing us to test end-to-end workings of *VaultWorks* smart-contracts.

For our exercise purposes we can use any address really, e.g.:
```bash
export VENDOR=  # Put an address used by Vendor / Keeper
```

**Note** If you're using ***Conveyor*** use its address.


### Step 6. Setup Collateral & Custody

We need to setup two more environment variables:

```bash
export COLLATERAL=     # Use contract address of a token to be used as collateral
export CUSTODY=        # Use custody contract address where collateral will be stored
```


If we're developing locally or in testnet we can use *Treasury* as collateral:

```bash
./scripts/treasury.sh full
```

After deployment it will print:

```
=== FULL DEPLOYMENT COMPLETE ===
Logic: 0x53409ae94aebf3b99a9d3cf41b8e093d0e185e20
Gate : 0xe56bdc533e7ef3388b30c7323c35cbdb55303033
```

Copy *Gate* address and export as `$COLLATERAL` environment variable.

Next you want to mint some of that collateral token to your wallet:

```bash
./scripts/send.sh $COLLATERAL "mint(address,uint256)" $DEPLOYER_ADDRESS 100000000000000000000000000000000000000000000000000000000000000000000
```

## Running End-to-End System

Once *Castle* & *Vault* are deployed and ready we can start [***Conveyor***](https://github.com/IndexMaker/conveyor), an *off-chain* service for developers, which will handle *on-chain* events, process pending orders, and submit supply and market-data.

Follow the [README guidelines](https://github.com/IndexMaker/conveyor?tab=readme-ov-file#conveyor) to setup *Conveyor* service.

Once *Converyor* finishes setting-up we can take address of the deployed *Vault* and export into environment variable:

```bash
export VAULT=0x9d50d88cf9ab84f59796e9424d3ec882eb15bbdc
```

### Basic Queries

Contgratulation, Scenario 5. ran successfully, now we can play.

Inspect ITP meta:
```bash
./scripts/call.sh $VAULT "symbol()(string)"
./scripts/call.sh $VAULT "name()(string)"
./scripts/call.sh $VAULT "decimals()(uint256)"
./scripts/call.sh $VAULT "collateralAsset()(address)"
```

Check total supply of ITP, and total assets value in ITP:
```bash
./scripts/call.sh $VAULT "totalSupply()" | ./scripts/parse_amount.py
./scripts/call.sh $VAULT "totalAssetsValue()" | ./scripts/parse_amount.py
```


Check your ITP balance, and assets value:
```bash
./scripts/call.sh $VAULT "balanceOf(address)" $DEPLOYER_ADDRESS | ./scripts/parse_amount.py
./scripts/call.sh $VAULT "assetsValue(address)" $DEPLOYER_ADDRESS | ./scripts/parse_amount.py
```

If you want to know average value of some amount of ITP,
and if you want to know amount of ITP worth of collateral:
```bash
./scripts/call.sh $VAULT "convertAssetsValue(uint128)" 1000000000000 | ./scripts/parse_amount.py
./scripts/call.sh $VAULT "convertItpAmount(uint128)" 1000000000000 | ./scripts/parse_amount.py
```

Additionally if you want to estimate how much you'd need to pay for ITP,
or you want to know how much ITP you'd get for given collateral:
```bash
./scripts/call.sh $VAULT "estimateAcquisitionCost(uint128)" 1000000000000  | ./scripts/parse_amount.py
./scripts/call.sh $VAULT "estimateAcquisitionItp(uint128)" 1000000000000 | ./scripts/parse_amount.py
```

And also if you are selling, and you want to know how much you will get for ITP,
and how much ITP you need to sell to get specific amount:
```bash
./scripts/call.sh $VAULT "estimateDisposalGains(uint128)" 1000000000000 | ./scripts/parse_amount.py
./scripts/call.sh $VAULT "estimateDisposalItpCost(uint128)" 1000000000000 | ./scripts/parse_amount.py
```

### Place ***Buy** & **Sell*** *Index Orders*

Let's try placing order!

Approve *Vault* to draw from our wallet:
```bash
./scripts/send.sh $COLLATERAL "approve(address,uint256)" $VAULT 1000000000000000000000
```

- Place *Buy* order with Instant Fill:
```bash
./scripts/send.sh $VAULT "placeBuyOrder(uint128,bool,address,address)(uint128,uint128,uint128)" 1000000000000000000000 true $VENDOR $DEPLOYER_ADDRESS
```

- A *Sell* order can be placed later once we acquire some token:
```bash
./scripts/send.sh $VAULT "placeSellOrder(uint128,bool,address,address)(uint128,uint128,uint128)" 10000000000000000 true $VENDOR $DEPLOYER_ADDRESS
```

**Note** The `placeBuyOrder()` returns a tuple: `(Received ITP, Collateral Spent, Collateral Remain)`, and the `placeSellOrder()` returns `(Received Amount, ITP Burnt, ITP Remain)`.

Trader can check their pending orders by calling:

```bash
./scripts/call.sh $VAULT "getPendingOrder(address,address)(uint128,uint128)" $VENDOR $DEPLOYER_ADDRESS
```

This returns a tuple: `(Pending Bid, Pending Ask)`, where:

- ***Pending Bid*** - amount of collateral still pending buy, and 
- ***Pending Ask*** - amount of ITP token pending sell.



### Claim ***ITP Token** & **Collateral***

Once *Keeper* pushes orders forwards, there will be some ***claimable*** amount available to get.

- For *Buy* order:
```bash
./scripts/call.sh $VAULT "getClaimableAcquisition(address)(uint128,uint128)" $VENDOR
```

- For *Sell* order:
```bash
./scripts/call.sh $VAULT "getClaimableDisposal(address)(uint128,uint128)" $VENDOR
```
for *Buy* and *Sell* correspondingly.

If there is some *claimable* amount, trader can claim that amount up to the amount deposited and pending *(use `getPendingOrder()` to see how much is pending)*.

Trader can preview claim amount by calling:
```bash
./scripts/call.sh $VAULT "claimAcquisition(uint128,address,address)(uint128)" 14093687789581242 $VENDOR $DEPLOYER_ADDRESS
```

and then claim amount by calling:
```bash
./scripts/send.sh $VAULT "claimAcquisition(uint128,address,address)(uint128)" 14093687789581242 $VENDOR $DEPLOYER_ADDRESS
```

If claim was successful, trader can check their balance:
```bash
./scripts/call.sh $VAULT "balanceOf(address)" $DEPLOYER_ADDRESS | ./scripts/parse_amount.py
```

## Developer Tools

We need to obtain *ID* of an *Index* and we can call:

```bash
export INDEX_ID=`./scripts/call.sh $VAULT "indexId()(uint128)"`
```

Although we have passed that ID as `--index-id` parameter to *Conveyor*. We will also need to remember ID of the vendor that we passed in as `--vendor-id` parameter.

```bash
export INDEX_ID=    # Use value you passed in as --index-id
export VENDOR_ID=   # Use value you passed in as --vendor-id
```


### Editing Index ***(Admin Mode)***

- ***Admin Mode*** - requires `Castle.ADMIN_ROLE` granted.

First we need to obtain ownership of *Vault* as otherwise *Guildmaster* is an owner:
```bash
./scripts/send.sh $CASTLE "beginEditIndex(uint128)" $INDEX_ID
```

We want to set our-selves as operator of that *Keeper*, so that we can make calls:
```bash
./scripts/send.sh $VAULT "setAdminOperator(address,bool)" $VENDOR true
```

**Note** The `setAdminOperator()` function is only available to *Vault* admin.

During development, using *Admin Mode* we can access directly some of the *Castle* functions, which allow us to bypass *Vault* logic, and put balances stored in *Vault* out-of-sync. We can fix this problem with simple calls:

```bash
./scripts/send.sh $VAULT "syncBalanceOf(address)" $DEPLOYER_ADDRESS
./scripts/send.sh $VAULT "syncTotalSupply()"
```

Once we finish editing, we need to return *Vault* ownership and notify *Guildmaster*:
```bash
./scripts/send.sh $VAULT "transferOwnership(address)" $CASTLE
./scripts/send.sh $CASTLE "finishEditIndex(uint128)" $INDEX_ID
```


### Investigation Tools

If we want to investigate current state of the order deeper we can double-check the order vectors fot trader:
```bash
./scripts/call.sh $CASTLE "getTraderOrder(uint128,address)(bytes)" $INDEX_ID $DEPLOYER_ADDRESS | ./scripts/parse_vector_bytes.py
```

and for *Vendor / Keeper*:
```bash
./scripts/call.sh $CASTLE "getTraderOrder(uint128,address)(bytes)" $INDEX_ID $VENDOR | ./scripts/parse_vector_bytes.py 
```

**Note** Trader's order vector would have *0.0* in the first *Collateral Remain* column, and forth *ITP Remain* column, while
*Keeper* would have some amounts there if there was still pending orders to execute.

Additionally we can check *Vendor Delta* with:
```bash
./scripts/call.sh $CASTLE "getVendorDelta(uint128)(bytes[])" $VENDOR_ID
```

or *Vendor* *Supply* and *Demand*:
```bash
./scripts/call.sh $CASTLE "getVendorSupply(uint128)(bytes[])" $VENDOR_ID
./scripts/call.sh $CASTLE "getVendorDemand(uint128)(bytes[])" $VENDOR_ID
```

to obtain *Vendor Assets*:
```bash
./scripts/call.sh $CASTLE "getVendorAssets(uint128)(bytes)" $VENDOR_ID
```

say it returned:
```
0x65000000000000000000000000000000660000000000000000000000000000006700000000000000000000000000000068000000000000000000000000000000690000000000000000000000000000006a0000000000000000000000000000006b0000000000000000000000000000006c0000000000000000000000000000006d000000000000000000000000000000 
```

then to submit zero supply:
```bash
./scripts/send.sh $CASTLE "submitSupply(uint128,bytes,bytes,bytes)" $VENDOR_ID 0x65000000000000000000000000000000660000000000000000000000000000006700000000000000000000000000000068000000000000000000000000000000690000000000000000000000000000006a0000000000000000000000000000006b0000000000000000000000000000006c0000000000000000000000000000006d000000000000000000000000000000 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
```

and then we can fetch *Vendor* Delta* and submit *Supply*:
```bash
./scripts/call.sh $CASTLE "getVendorDelta(uint128)(bytes[])" $VENDOR_ID
```

say we got output:
```
[0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000, 0x000000000000000000000000000000005c259bb96521a90e0000000000000000ad92cddcb29054070000000000000000ad92cddcb2905407000000000000000000000000000000000000000000000000ad92cddcb2905407000000000000000009b8689618b2fd1500000000000000000000000000000000000000000000000000000000000000000000000000000000]
```

then we can take second vector which is *Delta Short* and use it as *Supply Long* to cover for it:
```bash
./scripts/send.sh $CASTLE "submitSupply(uint128,bytes,bytes,bytes)" $VENDOR_ID 0x65000000000000000000000000000000660000000000000000000000000000006700000000000000000000000000000068000000000000000000000000000000690000000000000000000000000000006a0000000000000000000000000000006b0000000000000000000000000000006c0000000000000000000000000000006d000000000000000000000000000000 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 0x000000000000000000000000000000005c259bb96521a90e0000000000000000ad92cddcb29054070000000000000000ad92cddcb2905407000000000000000000000000000000000000000000000000ad92cddcb2905407000000000000000009b8689618b2fd1500000000000000000000000000000000000000000000000000000000000000000000000000000000
```

if we ask delta now:
```bash
./scripts/call.sh $CASTLE "getVendorDelta(uint128)(bytes[])" $VENDOR_ID
```

we should obtain zero vectors:
```
[0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000, 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000]
```

This is because by first zeroing supply, we made delta equal to inverted demand, and by submitting supply equal to short delta, we zeroed delta.
We would have as well submitted supply equal to inverted to demand.

***NOTE*** This is just for experimenting, not what Vendor would do. Vendor would make orders to exchange to neutralise *Delta*, and 
the algorithm used for that is up to *Vendor*.


***NOTE*** The `parse_amount.py` and `parse_vector_bytes.py` provided in `./scripts` directory are helper tools that prettify hex data into human friendly decimals and vector of decimals. These scripts require `python3` on your `PATH`.

The format of the ***Trader Order*** vector is: `[Collateral Remain, Collateral Spent, ITP Minted, ITP Locked, ITP Burned, Withdraw Amount]`.

The `Collateral Amount` is the remaining amount of collateral that hasn't yet been processed.

Set these *Maintainer Role*:
```bash
./scripts/roles.sh grant $CASTLE "Castle.MAINTAINER_ROLE" $DEPLOYER_ADDRESS
```

A developer can inspect recently executed quantities *(must have **Maintainer Role** granted)* by calling:
```bash
./scripts/call.sh $CASTLE "fetchVector(uint128)(bytes)" 1 | ./scripts/parse_vector_bytes.py
./scripts/call.sh $CASTLE "fetchVector(uint128)(bytes)" 2 | ./scripts/parse_vector_bytes.py
```
The `vectorId = 1` for *Asset* quantities and `vectorId = 2` for *Report* in `(Delivered, Received)` format. 

**Note** While these vectors are persisted on-chain, their valid lifetime is limitted.
They can be inspected to troubleshoot issues, however one needs to be aware that
these vectors are reused by various functions as a scratch memory for sharing temporary data with *Vector IL* programs.
Note also that only appointed members of the *Castle* have *write* access to any vectors, and while
we give developers access to fetch them, we do not provide any method to change them directly.


### Upgrading Castle NPC's

Should we need to upgrade one of the Castle's NPC's, e.g. Factor, we can do that easily as long
as we have *Admin* role granted.

We'll show on example of how to upgrade Factor.

First need to re-build Factor contract:
```bash
./scripts/check.sh factor
```

Next need to deploy it:
```bash
./scripts/deploy.sh factor 
```

That at the end should print line like this one:
```
Contract 'factor' deployed at: 0x956ab88947478591b52e068a81ef2c54906448af by: 0xC0D3Cb0c97CbF87F103a9901100D8f6D3e94D42A
```

We can take that address and call *Constable* method on *Castle* address to performa upgrade:
```bash
./scripts/send.sh $CASTLE "appointFactor(address)" 0x956ab88947478591b52e068a81ef2c54906448af
```

### Upgrading Vault Native

We can upgrade *Vault Native* contract.

First re-build and deploy *Vault Native* contract:
```bash
./scripts/check.sh vault_native
./scripts/deploy.sh vault_native
```

Next we need to re-initialize *Vault* with new implementation:
```bash
./scripts/send.sh $VAULT "initialize(address,address,address)" $DEPLOYER_ADDRESS 0x(deployed vault_native address) $CASTLE
```

### Upgrading Facets of Vault Native

We have already seen how to install *Orders* and *Claims* facets, and we shall use same methods.

First need to re-build either facet, e.g. *Orders*:
```bash
./scripts/check.sh vault_native_orders
```

The deploy:
```bash
./scripts/deploy.sh vault_native_orders
```

And eventually call *Vault Native* method to install *Orders* facet:
```bash
./scripts/send.sh $VAULT "installOrders(address)" 0xfb8c3906979fa82ed9e9e18c3ee21995761a13e7
```