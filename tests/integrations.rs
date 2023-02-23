use anyhow::Result;
use clarity::{Address as EthAddress, Transaction};
use ledger_ethereum::{Address, BIP44Path, EthApp, Signature};
use ledger_transport_speculos::TransportSpeculosHttp;
use secp256k1::{Message, PublicKey};

const EXPECTED_PUBKEY: [u8; 65] = [
    4, 60, 73, 239, 200, 111, 19, 92, 166, 192, 250, 16, 246, 185, 171, 38, 196, 97, 46, 80, 214,
    92, 247, 242, 143, 159, 171, 17, 123, 172, 102, 98, 255, 12, 19, 112, 46, 16, 14, 149, 110, 17,
    214, 245, 150, 40, 43, 219, 212, 191, 88, 228, 204, 91, 235, 204, 198, 89, 74, 193, 208, 103,
    212, 203, 30,
];

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
    let Address {
        public_key,
        address,
        ..
    } = app.address(&first_address(), None, None).await?;
    let address = "0x".to_string() + &String::from_utf8(address)?;
    assert_eq!(EXPECTED_PUBKEY.as_slice(), public_key);
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
        to: EthAddress::parse_and_validate("0x7562EF289fAf3554eEd27844B6473f165887cd40")?,
        value: 1_000_000_000_000u128.into(),
        data: vec![],
        signature: None,
    };
    let raw_tx = tx.to_bytes()?;
    dbg!(format!("{tx}"));
    let Signature { r, s, .. } = app.sign(&path, &raw_tx, None).await?;
    let sig = secp256k1::ecdsa::Signature::from_compact([r, s].concat().as_slice())?;
    let pubkey = PublicKey::from_slice(&EXPECTED_PUBKEY)?;
    let msg = Message::from_slice(&tx.hash())?;
    sig.verify(&msg, &pubkey)?;
    Ok(())
}

#[tokio::test]
async fn can_get_app_configuration() -> Result<()> {
    let config = dbg!(app().configuration().await?);
    assert_eq!("1.10.2", config.version);
    Ok(())
}

#[tokio::test]
async fn can_provide_erc20_info() -> Result<()> {
    std::env::set_var("RUST_LOG", "DEBUG");
    env_logger::init();
    // https://github.com/LedgerHQ/ledger-live/blob/develop/libs/ledgerjs/packages/cryptoassets/src/data/evm/5/erc20.json
    app()
        .provide_erc20_token_info("usdc", &hex::decode("07865c6E87B9F70255377e024ace6630C1Eaa37F").unwrap(), 6, 5, b"304402202736a1fe050770aa00916f53d90bfee112eea5cb5ad139b8e8829d95cdbdf94602202fb39953c0d6189dd8bb8c69c7e9145a67fb535243fa91e8e82eb38d5edf767f")
        .await?;
    Ok(())
}
