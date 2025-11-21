use deli::log_msg;
use ethers::{
    abi::{Address, Detokenize},
    contract::FunctionCall,
    middleware::SignerMiddleware,
    providers::{Http, Middleware, PendingTransaction, Provider},
    signers::{LocalWallet, Signer},
    types::{TransactionReceipt, U256},
};
use eyre::{Context, OptionExt};
use futures::future::join_all;
use itertools::Itertools;
use std::borrow::Borrow;
use std::sync::Arc;
use std::{env, str::FromStr};

pub struct TxSender {
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    signed_txs: Vec<ethers::core::types::Bytes>,
    nonce: Option<U256>,
}

impl TxSender {
    pub fn new(client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>) -> Self {
        Self {
            client,
            signed_txs: Vec::new(),
            nonce: None,
        }
    }

    pub fn address(&self) -> Address {
        self.client.address()
    }

    pub fn next_nonce(&mut self) -> eyre::Result<U256> {
        let nonce = self.nonce.ok_or_eyre("Nonce missing")?;
        self.nonce.replace(nonce + 1);
        Ok(nonce)
    }

    pub async fn begin(&mut self) -> eyre::Result<()> {
        log_msg!("begin transactions...");
        let nonce = self
            .client
            .get_transaction_count(self.address(), None)
            .await
            .context("Failed to fetch the current nonce from the Ethereum client")?;
        self.nonce.replace(nonce);
        Ok(())
    }

    pub async fn add<B, M, D>(&mut self, mut call: FunctionCall<B, M, D>) -> eyre::Result<()>
    where
        B: Borrow<M>,
        M: Middleware + 'static,
        D: Detokenize,
    {
        log_msg!("adding transaction...");
        call.tx.set_nonce(self.next_nonce()?);
        // call.tx.set_gas(1_000_000);
        // call.tx.set_gas_price(2_000_000_000);
        // call.tx.set_chain_id(self.client.signer().chain_id());
        // call.tx.set_from(self.client.signer().address());
        self.client
            .fill_transaction(&mut call.tx, call.block)
            .await?;
        log_msg!("\tnonce {:?}", call.tx.nonce());
        log_msg!("\tgas {:?}", call.tx.gas());
        log_msg!("\tgas price {:?}", call.tx.gas_price());
        log_msg!("\tchain_id {:?}", call.tx.chain_id());
        log_msg!("\tfrom {:?}", call.tx.from());
        let signature = self
            .client
            .signer()
            .sign_transaction(&call.tx)
            .await
            .context("Failed to sign tx")?;
        let signed_tx: ethers::core::types::Bytes = call.tx.rlp_signed(&signature);
        self.signed_txs.push(signed_tx);
        Ok(())
    }

    pub async fn end(self) -> eyre::Result<()> {
        log_msg!("sending transactions...");
        let mut pending = TxPending::new();

        for signed_tx in self.signed_txs {
            let pending_tx = self
                .client
                .send_raw_transaction(signed_tx)
                .await
                .context("Failed to send tx")?;
            pending.add_pending_tx(pending_tx);
        }

        pending.get_receipts().await?;
        Ok(())
    }
}

pub struct TxPending<'a> {
    pending_txs: Vec<PendingTransaction<'a, Http>>,
}

impl<'a> TxPending<'a> {
    fn new() -> Self {
        Self {
            pending_txs: Vec::new(),
        }
    }

    fn add_pending_tx(&mut self, pending_tx: PendingTransaction<'a, Http>) {
        self.pending_txs.push(pending_tx);
    }

    pub async fn get_receipts(self) -> eyre::Result<Vec<Option<TransactionReceipt>>> {
        log_msg!("awaiting receipts...");
        let (tx_receipts, send_errors): (Vec<_>, Vec<_>) = join_all(self.pending_txs)
            .await
            .into_iter()
            .partition_result();

        if !send_errors.is_empty() {
            Err(eyre::eyre!(
                "Errors while sending transactions: {:?}",
                send_errors
            ))?;
        }

        log_msg!(
            "{}",
            tx_receipts
                .iter()
                .map(|r| format!("Receipt: {:?}", r)).join("\n")
        );

        Ok(tx_receipts)
    }
}

pub struct TxSendBuilder<B, M, D>
where
    B: Borrow<M>,
    M: Middleware + 'static,
    D: Detokenize,
{
    sender: TxSender,
    calls: Vec<FunctionCall<B, M, D>>,
}

impl<B, M, D> TxSendBuilder<B, M, D>
where
    B: Borrow<M>,
    M: Middleware + 'static,
    D: Detokenize,
{
    pub fn new(client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>) -> Self {
        Self {
            sender: TxSender::new(client),
            calls: Vec::new(),
        }
    }

    pub fn add(mut self, call: FunctionCall<B, M, D>) -> Self {
        self.calls.push(call);
        self
    }

    pub async fn send(mut self) -> eyre::Result<()> {
        self.sender.begin().await?;
        for call in self.calls {
            self.sender.add(call).await?;
        }
        self.sender.end().await?;
        Ok(())
    }
}

pub struct TxClient {
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
}

impl TxClient {
    pub async fn try_new_from_url(
        rpc_url: &str,
        get_private_key: impl Fn() -> String,
    ) -> eyre::Result<Self> {
        let this = Self {
            client: {
                let provider = Provider::<Http>::try_from(rpc_url)?;
                let priv_key = get_private_key();
                let wallet = LocalWallet::from_str(&priv_key)?;
                let chain_id = provider.get_chainid().await?.as_u64();
                Arc::new(SignerMiddleware::new(
                    provider,
                    wallet.clone().with_chain_id(chain_id),
                ))
            },
        };
        Ok(this)
    }

    pub fn client(&self) -> Arc<SignerMiddleware<Provider<Http>, LocalWallet>> {
        self.client.clone()
    }

    pub fn address(&self) -> Address {
        self.client.address()
    }

    pub fn begin_tx<B, M, D>(&self) -> TxSendBuilder<B, M, D>
    where
        B: Borrow<M>,
        M: Middleware + 'static,
        D: Detokenize,
    {
        TxSendBuilder::new(self.client())
    }
}
