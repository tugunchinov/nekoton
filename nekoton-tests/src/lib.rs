extern crate core;

#[cfg(test)]
pub mod tests {
    use ed25519_dalek::ed25519::signature::Signature;
    use ed25519_dalek::Signer;
    use nekoton::core::models::{
        ContractState, Expiration, PendingTransaction, Transaction, TransactionAdditionalInfo,
        TransactionWithData, TransactionsBatchInfo,
    };
    use nekoton::core::ton_wallet::WalletType::HighloadWalletV2;
    use nekoton::core::ton_wallet::*;
    use nekoton::crypto::MnemonicType;
    use nekoton::transport::gql::GqlTransport;
    use nekoton_transport::gql::{GqlClient, GqlNetworkSettings};
    use nekoton_utils::SimpleClock;
    use reqwest::{StatusCode, Url};
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use std::time::Duration;
    use ton_block::{Account, AccountState, AccountStorage, AccountStuff, Deserializable};

    struct TestHandler {}

    impl TonWalletSubscriptionHandler for TestHandler {
        fn on_message_sent(&self, _: PendingTransaction, _: Option<Transaction>) {
            ()
        }

        fn on_message_expired(&self, _: PendingTransaction) {
            ()
        }

        fn on_state_changed(&self, _: ContractState) {
            ()
        }

        fn on_transactions_found(
            &self,
            _: Vec<TransactionWithData<TransactionAdditionalInfo>>,
            _: TransactionsBatchInfo,
        ) {
            ()
        }
    }

    pub async fn get_contract_state(
        contract_address: &str,
    ) -> Option<nekoton::transport::models::ExistingContract> {
        let client = reqwest::Client::new();
        let states_rpc_endpoint =
            Url::parse("https://jrpc.everwallet.net/rpc").expect("Bad rpc endpoint");

        #[derive(Serialize)]
        struct Address {
            pub address: String,
        }

        #[derive(Serialize)]
        struct Test {
            pub jsonrpc: String,
            pub id: u8,
            pub method: String,
            pub params: Address,
        }

        #[derive(Deserialize)]
        struct Response {
            pub result: Option<nekoton::transport::models::ExistingContract>,
        }

        let body = Test {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "getContractState".to_string(),
            params: Address {
                address: contract_address.to_string(),
            },
        };

        let response = client
            .post(states_rpc_endpoint)
            .json(&body)
            .send()
            .await
            .expect("Failed sending request");

        if let StatusCode::OK = response.status() {
            let response: Response = response.json().await.expect("Failed parsing");
            response.result
        } else {
            None
        }
    }

    #[tokio::test]
    async fn prepare_highload_transfer() -> () {
        let client = GqlClient::new(GqlNetworkSettings {
            endpoints: vec![
                "main2.ton.dev".to_string(),
                "main3.ton.dev".to_string(),
                "main4.ton.dev".to_string(),
            ],
            latency_detection_interval: Duration::from_secs(1),
            ..Default::default()
        })
        .expect("Failed to init GQL");
        let clock = Arc::new(SimpleClock);
        let transport = Arc::new(GqlTransport::new(client));

        let test_mnemonic =
            "razor advice advance announce palace decide tone catch fat journey position recipe";
        let key_pair = nekoton::crypto::derive_from_phrase(test_mnemonic, MnemonicType::Labs(0))
            .expect("Failed to derive from mnemonic");

        let handler = Arc::new(TestHandler {});

        let wallet = TonWallet::subscribe(
            clock,
            transport,
            0,
            key_pair.public.clone(),
            HighloadWalletV2,
            handler,
        )
        .await
        .expect("failed wallet");

        //corrupted init data
        //let data = "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";

        //Valid init data
        let data = "te6ccgECDAEAAWAAAnXABhDQdHogSStQjoLW9eQAaoTI1T1xPMcgiEu6n9CR2AgSGIIbwxR37wAAAGIQ6yPxFcR4a2LEu9PTQAMBAVkAAAAAYnnF5eriq+R5WnzkE2NmY3Kbv7TOxaJjG0hQ6ePvBIvGlHKcA6LkwcACABOgMTzjO0sdik9AART/APSkE/S88sgLBAIBIAcFAerygwjXGCDTH9M/+COqH1MgufJj7UTQ0x/TP9P/9ATRU2CAQPQOb6Ex8mBRc7ryogf5AVQQh/kQ8qMC9ATR+AB/jhYhgBD0eG+lIJgC0wfUMAH7AJEy4gGz5luDJaHIQDSAQPRDiuYxyBLLHxPLP8v/9ADJ7VQGADQggED0lm+lMlEQlDBTA7neIJMzNgGSMjDiswIBSAsIAgEgCgkAQb5fl2omhpj5jpn+n/mPoCaKkQQCB6BzfQmMktv8ld0fFAAXvZznaiaGmvmOuF/8AATQMA==";

        let account = Account::construct_from_base64(data).expect("Failed to construct account");

        let account_state = match account.state() {
            Some(state) => state.clone(),
            None => panic!("No account state"),
        };

        let state_init = match account_state.clone() {
            AccountState::AccountActive { state_init } => state_init,
            _ => panic!("Account is not active"),
        };

        let mut gifts = Vec::new();
        for _ in 0..1 {
            let gift = Gift {
                flags: 3,
                bounce: false,
                destination: wallet.address().clone(),
                amount: 10,
                body: None,
                state_init: Some(state_init.clone()),
            };
            gifts.push(gift);
        }

        let address = wallet.address().to_string();

        let state = get_contract_state(&address).await.unwrap();

        let fake_account_stuff = AccountStuff {
            addr: state.account.addr,
            storage_stat: state.account.storage_stat,
            storage: AccountStorage {
                last_trans_lt: state.account.storage.last_trans_lt,
                balance: state.account.storage.balance,
                state: account_state,
                init_code_hash: state.account.storage.init_code_hash,
            },
        };

        let transfer_action = highload_wallet_v2::prepare_transfer(
            &SimpleClock,
            &key_pair.public,
            &fake_account_stuff,
            gifts,
            Expiration::Timeout(1000),
        )
        .expect("Failed to prepare transfer");

        let message = match transfer_action {
            TransferAction::DeployFirst => highload_wallet_v2::prepare_deploy(
                &SimpleClock,
                &key_pair.public,
                0,
                Expiration::Timeout(1000),
            )
            .expect("Failed to prepare deploy"),
            TransferAction::Sign(message) => message,
        };

        let signature = key_pair.sign(message.hash());
        let mut ex_sign = [0u8; 64];
        ex_sign.copy_from_slice(signature.as_bytes());

        message.sign(&ex_sign).expect("Failed to sign message");
    }
}
