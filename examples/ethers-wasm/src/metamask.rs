use ethers::prelude::*;
use wasm_bindgen::prelude::*;
use serde::{Serialize, de::DeserializeOwned, Deserialize};
use std::fmt::Debug;
use thiserror::Error;
use async_trait::async_trait;
use ethers::types::{
    transaction::{eip2718::TypedTransaction, eip712::Eip712},
    Address, Signature,
};


#[derive(Error, Debug)]
pub enum MetamaskError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("no address")]
    NoAddress,
    #[error("{0}")]
    Custom(String),
}

////////// RPC /////////////////
#[derive(Debug)]
pub struct MetamaskRpc { }

#[async_trait(?Send)]
impl JsonRpcClient for MetamaskRpc {

    /// A JSON-RPC Error
    type Error = MetamaskError;

    /// Sends a request with the provided JSON-RPC and parameters serialized as JSON
    async fn request<T, R>(&self, method: &str, params: T) -> Result<R, Self::Error>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned {
            request_params(method, &params).await
        }
}


impl From<MetamaskError> for ProviderError {
    fn from(src: MetamaskError) -> Self {
        ProviderError::JsonRpcClientError(Box::new(src))
    }
}

///////////// SIGNER ////////////////
#[derive(Debug)]
pub struct MetamaskSigner{ 
    chain_id: u64,
    address: Address
}

impl MetamaskSigner {
    async fn new() -> Result<Self, MetamaskError> {
        let chain_id = request("eth_chainId").await?;

        let mut addrs:Vec<String> = request("eth_requestAccounts").await?;
        let selected = addrs.get(0).ok_or(MetamaskError::NoAddress)?;
        let address:Address = selected.parse().map_err(|_| MetamaskError::Custom(format!("couldn't parse the address {}", selected)))?;

        Ok(Self {
            chain_id,
            address
        })
    }
}


#[async_trait(?Send)]
impl Signer for MetamaskSigner {
    type Error = MetamaskError;

    /// Signs the hash of the provided message after prefixing it
    async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> Result<Signature, Self::Error> {
        //https://github.com/MetaMask/metamask-extension/issues/10297#issuecomment-937858286
     
        // I dunno... kinda guessing here
        let ret: String = request_params("personal_sign", &[message.as_ref(), self.address.as_bytes()]).await?;
        unimplemented!("TODO");
        
    }

    /// Signs the transaction
    async fn sign_transaction(&self, message: &TypedTransaction) -> Result<Signature, Self::Error> {
        unimplemented!("TODO");
    }

    /// Signs a EIP712 derived struct
    async fn sign_typed_data<T: Eip712 + Send + Sync>(
        &self,
        payload: &T,
    ) -> Result<Signature, Self::Error> {
        unimplemented!("TODO");
    }

    /// Returns the signer's Ethereum Address
    fn address(&self) -> Address {
        self.address
    }

    fn with_chain_id<T: Into<u64>>(mut self, chain_id: T) -> Self {
        self.chain_id = chain_id.into();
        self
    }

    fn chain_id(&self) -> u64 {
        self.chain_id
    }
}


///////////// BINDINGS ////////////////
///
/// Sends a request with the provided JSON-RPC and parameters serialized as JSON
async fn request_params<T, R>(method: &str, params: &T) -> Result<R, MetamaskError>
where
    T: Serialize,
    R: DeserializeOwned 
{

    #[derive(Serialize)]
    struct Args <'a, P: Serialize>{
        method: &'a str,
        params: &'a P
    }

    _request(JsValue::from_serde(&Args{ method, params: &params }).map_err(MetamaskError::Json)?)
        .await
        .into_serde()
        .map_err(MetamaskError::Json)
}

async fn request<R>(method: &str) -> Result<R, MetamaskError>
where
    R: DeserializeOwned 
{

    #[derive(Serialize)]
    struct Args <'a>{
        method: &'a str,
    }

    _request(JsValue::from_serde(&Args{ method}).map_err(MetamaskError::Json)?)
        .await
        .into_serde()
        .map_err(MetamaskError::Json)
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "ethereum"], js_name = request)]
    async fn _request(obj: JsValue) -> JsValue;
}
