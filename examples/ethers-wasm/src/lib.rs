#![allow(clippy::all)]
#![allow(warnings)]

use std::sync::Arc;

use wasm_bindgen::prelude::*;
use web_sys::console;

use ethers::{
    contract::abigen,
    prelude::{ContractFactory, Provider, SignerMiddleware},
    providers::Ws,
};

use crate::utils::SIMPLECONTRACT_BIN;

pub mod utils;
pub mod metamask;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}
pub(crate) use log;

abigen!(
    SimpleContract,
    "./../contract_abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

#[wasm_bindgen]
pub async fn deploy() {
    utils::set_panic_hook();

    console::log_2(
        &"SimpleContract ABI: ".into(),
        &JsValue::from_serde(&*SIMPLECONTRACT_ABI).unwrap(),
    );

    let client = match get_wallet_kind() {
        WalletKind::Local => {
            log!("local client");
            let wallet = utils::key(0);
            log!("Wallet: {:?}", wallet);

            let endpoint = "ws://127.0.0.1:8545";
            let provider = Provider::new(Ws::connect(endpoint).await.unwrap());
            log!("Provider connected to `{}`", endpoint);
            
            Arc::new(SignerMiddleware::new(provider, wallet))
        },
        WalletKind::Metamask => {
            log!("metamask client");
            //metamask::proof_of_concept().await;
            return;
        }
    };


    let bytecode = hex::decode(SIMPLECONTRACT_BIN).unwrap();
    let factory = ContractFactory::new(SIMPLECONTRACT_ABI.clone(), bytecode.into(), client.clone());

    log!("Deploying contract...");
    let contract = factory.deploy("hello WASM!".to_string()).unwrap().send().await.unwrap();
    let addr = contract.address();
    log!("Deployed contract with address: {:?}", addr);

    let contract = SimpleContract::new(addr, client.clone());

    let value = "bye from WASM!";
    log!("Setting value... `{}`", value);
    let receipt = contract.set_value(value.to_owned()).send().await.unwrap().await.unwrap();
    console::log_2(&"Set value receipt: ".into(), &JsValue::from_serde(&receipt).unwrap());

    log!("Fetching logs...");
    let logs = contract.value_changed_filter().from_block(0u64).query().await.unwrap();

    let value = contract.get_value().call().await.unwrap();

    console::log_2(
        &format!("Value: `{}`. Logs: ", value).into(),
        &JsValue::from_serde(&logs).unwrap(),
    );
}

#[derive(Debug)]
enum WalletKind {
    Metamask,
    Local 
}

fn get_wallet_kind() -> WalletKind {
    let url = web_sys::Url::new(
        &web_sys::window()
            .unwrap()
            .location()
            .href()
            .unwrap()
    ).unwrap();

    match url.search_params().get("wallet") {
        None => WalletKind::Local,
        Some(kind) => if kind == "metamask" { WalletKind::Metamask } else { WalletKind::Local }
    }
}
