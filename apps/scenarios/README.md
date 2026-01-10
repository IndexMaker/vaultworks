# End to End Flow

## How to setup

### Step 1. Start *Nitro Dev Node*

Follow [instructions](https://github.com/OffchainLabs/nitro-devnode) on *Nitro Dev Node* project page.

### Step 2. Setup environment

Need to set private key:
```
export DEPLOY_PRIVATE_KEY=`cat some-test-key`
```

Address can be obtained using:
```
export DEPLOYER_ADDRESS=`cast wallet address $DEPLOY_PRIVATE_KEY`
```

Also RPC URL is needed for running scenarios:
```
export RPC_URL="http://localhost:8547"
```

### Step 3. Deploy whole *Castle*

First ensure you've built latest version of the codebase:
```
./scripts/check-all.sh
```

Now we can deploy a *Castle* like this:
```
./scripts/castle.sh --no-gates
```

Once deployment completes at the end similar information will show:
```
---------------------------
=== Deployment Complete ===
---------------------------
Castle Target: 0x0444764a212240b69d3ad81b9a77f34945d1b228
Clerk Target: 0x9ae8a390121ba71545e9923b333d60e7e3ccd3bd
---------------------------
```

Copy address of `Castle Target` and export as `$CASTLE` variable, e.g.

```
export CASTLE="0x0444764a212240b69d3ad81b9a77f34945d1b228"
```


### Step 4. Set roles:

Set these three roles:
```
./scripts/roles.sh grant $CASTLE "Castle.ISSUER_ROLE" $DEPLOYER_ADDRESS
./scripts/roles.sh grant $CASTLE "Castle.KEEPER_ROLE" $DEPLOYER_ADDRESS
./scripts/roles.sh grant $CASTLE "Castle.VENDOR_ROLE" $DEPLOYER_ADDRESS
./scripts/roles.sh grant $CASTLE "Castle.VAULT_ROLE" $DEPLOYER_ADDRESS
./scripts/roles.sh grant $CASTLE "Castle.MAINTAINER_ROLE" $DEPLOYER_ADDRESS
```

### Step 5. Add *Vault* to *Worksman* free-list

We need to deploy some *Vault* contract to populate *Worksman* free-list, and we'll use *Vault-Native* option:

```
./scripts/vault.sh full $CASTLE --native
```

Script at the end will show similar output:
```
=== VAULT DEPLOYMENT COMPLETE ===
Vault Requests address: 0xd01207dd6eb9359f7572f658de0cb4ec98858da5
Vault Logic: 0xfb8c3906979fa82ed9e9e18c3ee21995761a13e7
Vault Gate : 0xeff7b46049fc677f58264e0ebb19df1a39195a21
Vault Owner: 0xab8e440727a38bbb180f7032ca4a8009e7b52b80
------------------------------------
```

Copy address of the `Vault Gate` and export as `$VAULT` vailable, e.g.

```
export VAULT=0xeff7b46049fc677f58264e0ebb19df1a39195a21
```

Need to deploy Orders & Claims:
```
./scripts/deploy.sh vault_native_orders
./scripts/deploy.sh vault_native_claims
```

and then install using the addresses deployed orders & claims, e.g.:
```
./scripts/send.sh $VAULT "installOrders(address)" 0xfb8c3906979fa82ed9e9e18c3ee21995761a13e7
./scripts/send.sh $VAULT "installClaims(address)" 0x95e7a50f9bd7189c9e8d52462410c921592e821e
```

For now we can grant *Vault Role* to the *Vault* like:
```
./scripts/roles.sh grant $CASTLE "Castle.VAULT_ROLE" $VAULT
./scripts/roles.sh grant $CASTLE "Castle.KEEPER_ROLE" $VAULT
```

we can deploy Treasury to serve as collateral:
```
./scripts/treasury.sh full
```

```
=== FULL DEPLOYMENT COMPLETE ===
Logic: 0x819c6ea7e7aea2eb95d1926d520a76cd03c53aca
Gate : 0x8571fc20dd9323af25e0d5c3f4795d8954f95498
```

and export to environment:
```
export COLLATERAL=0x8571fc20dd9323af25e0d5c3f4795d8954f95498
```

for custody we can use some address, or *Castle* for now:
```
export CUSTODY=$CASTLE
```

until worksman does it, we can also configure *Vault* like:
```
./scripts/send.sh $VAULT "configureVault(uint128,string,string)" 1001 "Top100" "T100"
./scripts/send.sh $VAULT "configureRequests(uint128,address,address,uint128)" "1" $CUSTODY $COLLATERAL 100000000000000000000
```

and in next command add *Vault* to free-list, which will look like:
```
./scripts/send.sh $CASTLE "addVault(address)" $VAULT
```

This adds that *Gate* to *Workman's* free-list, and then when *Guildmaster*
requests to build a *Vault* *Worksman* will pick next from that free-list.

### Step 6. Setup Keeper Address

For our exercise purposes we can use any address really, e.g.:
```
export VENDOR=0xcb593e5f96363a4919b583f07fe45880a1daf94e
```

Normally this would be an address derived from *Keeper's* wallet.

We want to set our-selves as operator of that *Keeper*, so that we can make calls:
```
./scripts/send.sh $VAULT "setAdminOperator(address,bool)" $VENDOR true
```

**Note** The `setAdminOperator()` function is only available to *Vault* admin.


### Step 7. Run Scenario 5.

Congratulations, you have set-up the environment to run once-and-only-once Scenario 5.

```
cargo run -p scenarios -- --rpc-url $RPC_URL --private-key $DEPLOY_PRIVATE_KEY --castle-address $CASTLE --keeper-address $VENDOR -s scenario5
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

## After Setup

### Basic Queries

Contgratulation, Scenario 5. ran successfully, now we can play.

Inspect ITP meta:
```
./scripts/call.sh $VAULT "symbol()(string)"
./scripts/call.sh $VAULT "name()(string)"
./scripts/call.sh $VAULT "decimals()(uint256)"
./scripts/call.sh $VAULT "collateralAsset()"
```

Check total supply of ITP, and total assets value in ITP:
```
./scripts/call.sh $VAULT "totalSupply()" | ./scripts/parse_amount.py
./scripts/call.sh $VAULT "totalAssetsValue()" | ./scripts/parse_amount.py
```


Check your ITP balance, and assets value:
```
./scripts/call.sh $VAULT "balanceOf(address)" $DEPLOYER_ADDRESS | ./scripts/parse_amount.py
./scripts/call.sh $VAULT "assetsValue(address)" $DEPLOYER_ADDRESS | ./scripts/parse_amount.py
```

Transfer some assets to another address, e.g. *Castle*:
```
./scripts/send.sh $VAULT "transfer(address,uint256)" $CASTLE 1000
```

If you want to know average value of some amount of ITP,
and if you want to know amount of ITP worth of collateral:
```
./scripts/call.sh $VAULT "convertAssetsValue(uint128)" 1000000000000 | ./scripts/parse_amount.py
./scripts/call.sh $VAULT "convertItpAmount(uint128)" 1000000000000 | ./scripts/parse_amount.py
```

Additionally if you want to estimate how much you'd need to pay for ITP,
or you want to know how much ITP you'd get for given collateral:
```
./scripts/call.sh $VAULT "estimateAcquisitionCost(uint128)" 1000000000000  | ./scripts/parse_amount.py
./scripts/call.sh $VAULT "estimateAcquisitionItp(uint128)" 1000000000000 | ./scripts/parse_amount.py
```

And also if you are selling, and you want to know how much you will get for ITP,
and how much ITP you need to sell to get specific amount:
```
./scripts/call.sh $VAULT "estimateDisposalGains(uint128)" 1000000000000 | ./scripts/parse_amount.py
./scripts/call.sh $VAULT "estimateDisposalItpCost(uint128)" 1000000000000 | ./scripts/parse_amount.py
```

### Place BUY order

Let's try placing order!

Mint some collateral token first:
```
./scripts/send.sh $COLLATERAL "mint(address,uint256)" $DEPLOYER_ADDRESS 100000000000000000000000000000000000000000000000000000000000000000000
```

Approve *Vault* to draw from our wallet:
```
./scripts/send.sh $COLLATERAL "approve(address,uint256)" $VAULT 1000000000000000000000
```

Place an order with Instant Fill:
```
./scripts/send.sh $VAULT "placeBuyOrder(uint128,bool,address,address)(uint128,uint128,uint128)" 1000000000000000000000 true $VENDOR $DEPLOYER_ADDRESS
./scripts/send.sh $VAULT "placeSellOrder(uint128,bool,address,address)(uint128,uint128,uint128)" 10000000000000000 true $VENDOR $DEPLOYER_ADDRESS
```


The `placeBuyOrder()` returns a tuple: `(Received ITP, Collateral Spent, Collateral Remain)`, and
the `placeSellOrder()` returns `(Received Amount, ITP Burnt, ITP Remain)`.

Trader can check their pending orders by calling:

```
./scripts/call.sh $VAULT "getPendingOrder(address,address)(uint128,uint128)" $VENDOR $DEPLOYER_ADDRESS
```

This returns a tuple: `(Pending Bid, Pending Ask)`.

The ***Keeper*** service pays gas to push forward pending orders:
```
./scripts/send.sh $VAULT "processPendingBuyOrder(address)(uint128,uint128,uint128)" $VENDOR
./scripts/send.sh $VAULT "processPendingSellOrder(address)(uint128,uint128,uint128)" $VENDOR
```
for pushing forward *Buy* and *Sell* orders correspondingly.

These functions only take *Keeper's* address, and all execution parameters are dictated by *Vault*, 
so that *Keeper* can only choose when to process next batch, but not the quantities, nor which orders.


### Claim ITP

Once *Keeper* pushes orders forwards, there will be some ***claimable*** amount available to get.

Trader can query that ammount by calling:
```
./scripts/call.sh $VAULT "getClaimableAcquisition(address)(uint128,uint128)" $VENDOR
./scripts/call.sh $VAULT "getClaimableDisposal(address)(uint128,uint128)" $VENDOR
```
for *Buy* and *Sell* correspondingly.

If there is some *claimable* amount, trader can claim that amount up to the amount deposited and pending *(use `getPendingOrder()` to see how much is pending)*.

Trader can preview claim amount by calling:
```
./scripts/call.sh $VAULT "claimAcquisition(uint128,address,address)(uint128)" 14093687789581242 $VENDOR $DEPLOYER_ADDRESS
```

and then claim amount by calling:
```
./scripts/send.sh $VAULT "claimAcquisition(uint128,address,address)(uint128)" 14093687789581242 $VENDOR $DEPLOYER_ADDRESS
```

If claim was successful, trader can check their balance:
```
./scripts/call.sh $VAULT "balanceOf(address)(uint256)" $DEPLOYER_ADDRESS
```

### Developer tools

If we want to investigate current state of the order deeper we can double-check the order vectors fot trader:
```
./scripts/call.sh $CASTLE "getTraderOrder(uint128,address)(bytes)" 1001 $DEPLOYER_ADDRESS | ./scripts/parse_vector_bytes.py
```

and for *Keeper*:
```
./scripts/call.sh $CASTLE "getTraderOrder(uint128,address)(bytes)" 1001 $VENDOR | ./scripts/parse_vector_bytes.py 
```

**Note** Trader's order vector would have *0.0* in the first *Collateral Remain* column, and forth *ITP Remain* column, while
*Keeper* would have some amounts there if there was still pending orders to execute.

Additionally we can check *Vendor Delta* with:
```
./scripts/call.sh $CASTLE "getVendorDelta(uint128)(bytes[])" 1
```

or *Vendor* *Supply* and *Demand*:
```
./scripts/call.sh $CASTLE "getVendorSupply(uint128)(bytes[])" 1
./scripts/call.sh $CASTLE "getVendorDemand(uint128)(bytes[])" 1
```



***NOTE*** The `parse_amount.py` and `parse_vector_bytes.py` provided in `./scripts` directory are helper tools that prettify hex data into human friendly decimals and vector of decimals. These scripts require `python3` on your `PATH`.

The format of the ***Trader Order*** vector is: `[Collateral Remain, Collateral Spent, ITP Minted, ITP Locked, ITP Burned, Withdraw Amount]`.

The `Collateral Amount` is the remaining amount of collateral that hasn't yet been processed.

A developer can inspect recently executed quantities *(must have **Maintainer Role** granted)* by calling:
```
./scripts/call.sh $CASTLE "fetchVector(uint128)(bytes)" 1 | ./scripts/parse_vector_bytes.py
./scripts/call.sh $CASTLE "fetchVector(uint128)(bytes)" 2 | ./scripts/parse_vector_bytes.py
```
The `vectorId = 1` for *Asset* quantities and `vectorId = 2` for *Report* in `(Delivered, Received)` format. 

**Note** While these vectors are persisted on-chain, their valid lifetime is limitted.
They can be inspected to troubleshoot issues, however one needs to be aware that
these vectors are reused by various functions as a scratch memory for sharing temporary data with *Vector IL* programs.
Note also that only appointed members of the *Castle* have *write* access to any vectors, and while
we give developers access to fetch them, we do not provide any method to change them directly.
