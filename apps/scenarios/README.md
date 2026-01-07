# End to End Flow

## How to setup

1. Start *Nitro Dev Node*

Follow [instructions](https://github.com/OffchainLabs/nitro-devnode) on *Nitro Dev Node* project page.

2. Setup environment

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
Clerk Target: 0x9ae8a390121ba71545e9923b333d60e7e3ccd3bd
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
./scripts/roles.sh grant $CASTLE "Castle.VAULT_ROLE" $DEPLOYER_ADDRESS
```

5. Add *Vault* to *Worksman* free-list

We need to deploy some *Vault* contract to populate *Worksman* free-list:

```
./scripts/vault.sh full $CASTLE 
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

for now we can grant *Vault Raole* to the *Vault* like:
```
./scripts/roles.sh grant $CASTLE "Castle.VAULT_ROLE" $VAULT
```

until worksman does it, we can also configure *Vault* like:
```
./scripts/send.sh $VAULT "configureVault(uint128,string,string)" 1001 "Top100" "T100"
./scripts/send.sh $VAULT "configureRequests(uint128,address,address,uint128)" "1" $DEPLOYER_ADDRESS $DEPLOYER_ADDRESS 10000000000000000000000
```

and in next command add *Vault* to free-list, which will look like:
```
./scripts/send.sh $CASTLE "addVault(address)" $VAULT
```

This adds that *Gate* to *Workman's* free-list, and then when *Guildmaster*
requests to build a *Vault* *Worksman* will pick next from that free-list.


6. Run Scenario 5.

Congratulations, you have set-up the environment to run once-and-only-once Scenario 5.

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
