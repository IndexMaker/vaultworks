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
