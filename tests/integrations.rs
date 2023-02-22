use anyhow::Result;
use clarity::{Address, Transaction};
use ledger_ethereum::types::{BIP44Path, GetAddressResponse, Signature};
use ledger_ethereum::EthApp;
use ledger_transport_speculos::TransportSpeculosHttp;
use secp256k1::{Message, PublicKey};
fn app() -> EthApp<TransportSpeculosHttp> {
    EthApp::new(TransportSpeculosHttp::new("172.17.0.2", 5000))
}

// 44'/60'/0'/0'/0
// seed: 6f0cd08f62d99e62ebb1e15f46df842c02380fd9f2abf987f0b5463adae25caeb564583bd413c9b7cbf0391808308332251e47696dd13688dc96b9edbccd981b
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
    assert_eq!("043c49efc86f135ca6c0fa10f6b9ab26c4612e50d65cf7f28f9fab117bac6662ff0c13702e100e956e11d6f596282bdbd4bf58e4cc5bebccc6594ac1d067d4cb1e", public_key);
    assert_eq!("0x7562EF289fAf3554eEd27844B6473f165887cd40", address);
    Ok(())
}

#[tokio::test]
async fn can_sign_transaction() -> Result<()> {
    std::env::set_var("RUST_LOG", "DEBUG");
    env_logger::init();
    let app = app();
    let path = first_address();
    let tx = Transaction {
        nonce: 0u32.into(),
        gas_price: 1_000_000u32.into(),
        gas_limit: 1_000_000u32.into(),
        to: Address::parse_and_validate("0x7562EF289fAf3554eEd27844B6473f165887cd40")?,
        value: 1_000_000_000_000u128.into(),
        data: vec![],
        signature: None,
    };
    let raw_tx = tx.to_bytes()?;
    dbg!(hex::encode(&raw_tx));
    let Signature { r, s, .. } = app.sign(&path, &raw_tx, None).await?;
    let sig = secp256k1::ecdsa::Signature::from_compact([r, s].concat().as_slice())?;
    let pubkey = PublicKey::from_slice(&hex::decode("043c49efc86f135ca6c0fa10f6b9ab26c4612e50d65cf7f28f9fab117bac6662ff0c13702e100e956e11d6f596282bdbd4bf58e4cc5bebccc6594ac1d067d4cb1e")?)?;
    let msg = Message::from_slice(&tx.hash())?;
    sig.verify(&msg, &pubkey)?;
    Ok(())
}
