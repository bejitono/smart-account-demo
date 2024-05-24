uniffi::include_scaffolding!("lib");
use std::{fmt, sync::Arc};
use tokio::sync::RwLock;
use ethers::{
    abi::{Function, Param, ParamType, StateMutability, Token}, core::rand, providers::{Http, Provider}, signers::Wallet, types::{Address, Bytes, Chain, TransactionRequest, H256, U256}, utils::hex
};

use ethers::signers::{LocalWallet, Signer};
use aa_sdk_rs::{smart_account::{BaseAccount, SafeStandardAccount, SmartAccountMiddleware, SmartAccountProvider}, types::{ExecuteCall, UserOperationRequest}};
use url::Url;

use eyre::Result;
// use aa_sdk_rs::smart_account::st


const ENTRYPOINT_ADDRESS: &str = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789";
const SAFE_PROXY_FACTORY_ADDRESS: &str = "0x4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67";
const SAFE_4337_MODULE_ADDRESS: &str = "0xa581c4A4DB7175302464fF3C06380BC3270b4037";

const RPC_URL: &str = "https://base-sepolia.g.alchemy.com/v2/IVqOyg3PqHzBQJMqa_yZAfyonF9ne2Gx";

#[uniffi::export]
pub fn make_smart_account_demo_provider(wallet: Vec<u8>, account_address: Option<String>) -> Arc<SmartAccountDemoProvider> {
    let provider = SmartAccountDemoProvider::new(wallet, account_address);
    Arc::new(provider)
}

#[derive(uniffi::Object)]
pub struct SmartAccountDemoProvider {
    provider: SmartAccountProvider<Http, SafeStandardAccount>,
    account: SafeStandardAccount,
}

impl SmartAccountDemoProvider {
    fn new(wallet: Vec<u8>, account_address: Option<String>) -> Self {
        let factory_address: Address = SAFE_PROXY_FACTORY_ADDRESS.parse().unwrap();
        let factory_address: Address = SAFE_PROXY_FACTORY_ADDRESS.parse().unwrap();
        let entry_point_address: Address = ENTRYPOINT_ADDRESS.parse().unwrap();
        // let wallet: LocalWallet = wallet
        //     .parse()
        //     .unwrap();
        let wallet = LocalWallet::from_bytes(&wallet).unwrap();

        let account_address: Option<Address> = match account_address {
            Some(addr) => Some(addr.parse().unwrap()),
            None => None,
        };

        let cloned_http_provider = Provider::<Http>::try_from(RPC_URL).unwrap();

        let cloned_account: SafeStandardAccount = SafeStandardAccount::new(
            Arc::new(cloned_http_provider),
            vec![wallet.address()],
            U256::one(),
            RwLock::new(account_address),
            factory_address,
            entry_point_address,
            SAFE_4337_MODULE_ADDRESS.parse().unwrap(),
            RwLock::new(account_address.is_some()),
            Chain::BaseSepolia,
        );

        let provider = make_provider(cloned_account);

        let http_provider = Provider::<Http>::try_from(RPC_URL).unwrap();

        // TODO: Make Cloneable
        let account: SafeStandardAccount = SafeStandardAccount::new(
            Arc::new(http_provider),
            vec![wallet.address()],
            U256::one(),
            RwLock::new(account_address),
            factory_address,
            entry_point_address,
            SAFE_4337_MODULE_ADDRESS.parse().unwrap(),
            RwLock::new(account_address.is_some()),
            Chain::BaseSepolia,
        );

        Self {
            provider,
            account
         }
    }

}

#[uniffi::export(async_runtime = "tokio")]
impl SmartAccountDemoProvider {

    pub async fn address(&self) -> Vec<u8>  {
        self.account.get_account_address().await.unwrap().as_bytes().into()
    }

    pub async fn send_transaction(&self, wallet: Vec<u8>, value: u64, to_address: String) -> Result<SendTransactionResult, SmartAccountDemoError> {
        // let wallet: LocalWallet = wallet
        //     .parse()
        //     .unwrap();
        let wallet = LocalWallet::from_bytes(&wallet).unwrap();

        let to_address: Address = to_address//"0xde3e943a1c2211cfb087dc6654af2a9728b15536"
            .parse()
            .unwrap();

        let sender: Address = self.account.get_account_address()
            .await
            .map_err(|_| SmartAccountDemoError::ProviderError)?;

        let call = ExecuteCall::new(
            to_address,
            value,
            Bytes::new(),
        );

        let encoded_call: Vec<u8> = self.account.encode_execute(call).await.unwrap();

        let req = UserOperationRequest::new()
            .call_data(encoded_call)
            .sender(sender);

        let updated_user_op: UserOperationRequest = self.account.get_paymaster_and_data(req)
            .await
            .map_err(|_| SmartAccountDemoError::ProviderError)?;

        println!("updated_user_op {:?}", updated_user_op);
        
        let result = self.provider.send_user_operation(updated_user_op, &wallet)
            .await
            .map_err(|_| SmartAccountDemoError::ProviderError)?;

        let result = SendTransactionResult {
            user_op_hash: result.0.to_vec()
        };

        Ok(result)
    }

    pub async fn mint_tokens(&self, wallet: Vec<u8>, amount: u64, contract_address: String) -> Result<SendTransactionResult, SmartAccountDemoError> {
        let wallet = LocalWallet::from_bytes(&wallet).unwrap();//wallet.parse().unwrap();

        let contract_address: Address = contract_address.parse().unwrap();

        let mint_fn = mint_function_abi();

        let call_data = mint_fn.encode_input(&[Token::Uint(U256::from(amount))]).unwrap();

        let sender: Address = self.account.get_account_address()
            .await
            .map_err(|_| SmartAccountDemoError::ProviderError)?;

        let call = ExecuteCall::new(
            contract_address,
            0,
            call_data,
        );

        let encoded_call: Vec<u8> = self.account.encode_execute(call).await.unwrap();

        let req = UserOperationRequest::new()
            .call_data(encoded_call)
            .sender(sender);

        let updated_user_op: UserOperationRequest = self.account.get_paymaster_and_data(req)
            .await
            .map_err(|_| SmartAccountDemoError::ProviderError)?;

        println!("updated_user_op {:?}", updated_user_op);
        
        let result = self.provider.send_user_operation(updated_user_op, &wallet)
            .await
            .map_err(|_| SmartAccountDemoError::ProviderError)?;

        let result = SendTransactionResult {
            user_op_hash: result.0.to_vec()
        };

        Ok(result)
    }

    pub async fn get_user_operation_receipt(&self, user_op_hash: Vec<u8>) -> Result<Option<bool>, SmartAccountDemoError> {
        let raw_hash = H256::from_slice(&user_op_hash);//H256::from(&user_op_hash).map_err(|_| SmartAccountDemoError::ProviderError)?;

        match self.provider.get_user_operation_receipt(raw_hash).await {
            Ok(receipt) => {
                let Some(receipt) = receipt else { return Ok(None) };
                
                Ok(Some(receipt.success))
            },
            Err(_) => Err(SmartAccountDemoError::ProviderError)
        }
    }
}

fn make_provider(
    account: SafeStandardAccount,
) -> SmartAccountProvider<Http, SafeStandardAccount> {
    let url: Url = RPC_URL.try_into().unwrap();
    let http_provider = Http::new(url);

    let account_provider = SmartAccountProvider::new(http_provider, account);

    account_provider
}

fn mint_function_abi() -> Function {
    Function {
        name: "mint".to_owned(),
        inputs: vec![Param {
            name: "_amount".to_owned(),
            kind: ParamType::Uint(256),
            internal_type: Some("uint256".to_owned()),
        }],
        outputs: vec![],
        constant: Some(false),
        state_mutability: StateMutability::NonPayable,
    }
}

#[uniffi::export]
pub fn make_new_wallet() -> Vec<u8> {
    let wallet = LocalWallet::new(&mut rand::thread_rng());
    wallet.signer().to_bytes().as_slice().into()
}

#[derive(uniffi::Record, Debug, Clone)]
pub struct SendTransactionResult {
    pub user_op_hash: Vec<u8>
}

#[derive(uniffi::Error, Debug)]
pub enum SmartAccountDemoError {
    ProviderError,
}

impl fmt::Display for SmartAccountDemoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SmartAccountDemoError {}

// // ... and much more! For more information about bindings, read the UniFFI book: https://mozilla.github.io/uniffi-rs/udl_file_spec.html