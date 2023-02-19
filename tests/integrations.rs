use anyhow::Result;
use ledger_ethereum::types::{BIP44Path, GetAddressResponse, Signature};
use ledger_ethereum::EthApp;
use ledger_transport_speculos::TransportSpeculosHttp;

// All test are with Speculos defaults, i.e seed is "secret"
fn app() -> EthApp<TransportSpeculosHttp> {
    EthApp::new(TransportSpeculosHttp::new("172.17.0.2", 5000))
}

// 44'/60'/0'/0'/0
fn first_address() -> BIP44Path {
    BIP44Path {
        purpose: 44,
        coin: 60,
        account: 0,
        change: 0,
        index: 0,
    }
}

#[tokio::test]
async fn can_get_address() -> Result<()> {
    let app = app();
    let GetAddressResponse {
        public_key,
        address,
        ..
    } = app.address(&first_address(), None, None).await?;
    let public_key = hex::encode(public_key);
    let address = "0x".to_string() + &String::from_utf8(address)?;
    assert_eq!("04ab60257ee4c966cf7937031a35833fe24bc70227fc96e198a0adbabaa9c527abc6e50e9f66d8d8d24a88edb05d4468fb2346e726b8eec9f92c9529cdbb9ed578", public_key);
    assert_eq!("0xfC0Ee1a8BBbe669d24A00c193646bcd02c2270fB", address);
    Ok(())
}

#[tokio::test]
async fn can_sign_transaction() -> Result<()> {
    let app = app();
    let Signature { v, r, s } = app.sign(&first_address(), &[], None).await?;
    Ok(())
}
