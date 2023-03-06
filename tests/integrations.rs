#![feature(once_cell)]
use std::sync::LazyLock;

use anyhow::Result;
use byteorder::{BigEndian, WriteBytesExt};
use ethereum_tx_sign::{LegacyTransaction, Transaction};
use ledger_ethereum::{Address, BIP44Path, EthApp, Signature};
use ledger_transport_speculos::api::Button;
use ledger_transport_speculos::TransportSpeculosHttp;
use rlp::RlpStream;
use secp256k1::hashes::sha256::Hash;
use secp256k1::{Message, PublicKey};
use serial_test::serial;
use tiny_keccak::{Hasher, Keccak};
use tokio::spawn;

static LOGGER: LazyLock<()> = LazyLock::new(|| {
    std::env::set_var("RUST_LOG", "DEBUG");
    env_logger::init();
});

const EXPECTED_PUBKEY: [u8; 65] = [
    4, 60, 73, 239, 200, 111, 19, 92, 166, 192, 250, 16, 246, 185, 171, 38, 196, 97, 46, 80, 214,
    92, 247, 242, 143, 159, 171, 17, 123, 172, 102, 98, 255, 12, 19, 112, 46, 16, 14, 149, 110, 17,
    214, 245, 150, 40, 43, 219, 212, 191, 88, 228, 204, 91, 235, 204, 198, 89, 74, 193, 208, 103,
    212, 203, 30,
];

fn app() -> EthApp<TransportSpeculosHttp> {
    let _ = LOGGER;
    EthApp::new(TransportSpeculosHttp::new("127.0.0.1", 5000))
}

fn api_client() -> TransportSpeculosHttp {
    TransportSpeculosHttp::new("127.0.0.1", 5000)
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
#[serial]
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
#[serial]
async fn can_sign_transaction() -> Result<()> {
    let app = app();
    let path = first_address();
    let tx = LegacyTransaction {
        chain: 5,
        nonce: 0,
        to: Some(
            hex::decode("7562EF289fAf3554eEd27844B6473f165887cd40")?
                .try_into()
                .unwrap(),
        ),
        value: 1_000_000_000_000,
        gas_price: 1_000_000,
        gas: 1_000_000,
        data: vec![],
    };

    let mut rlp_stream = RlpStream::new();
    let rlp = tx.rlp_parts();
    rlp_stream.begin_unbounded_list();
    for r in rlp.iter() {
        rlp_stream.append(r);
    }
    rlp_stream.append(&tx.chain);
    rlp_stream.append(&vec![]);
    rlp_stream.append(&vec![]);
    rlp_stream.finalize_unbounded_list();
    let raw_tx = rlp_stream.out().to_vec();
    let client = api_client();

    let _raw_tx = raw_tx.clone();
    let handle = spawn(async move { app.sign(&path, &_raw_tx, None).await });
    client.button(Button::Right).await?;
    client.button(Button::Right).await?;
    client.button(Button::Right).await?;
    client.button(Button::Right).await?;
    client.button(Button::Both).await?;
    let Signature { r, s, .. } = handle.await??;
    let sig = secp256k1::ecdsa::Signature::from_compact([r, s].concat().as_slice())?;
    let pubkey = PublicKey::from_slice(&EXPECTED_PUBKEY)?;
    let msg = Message::from_slice(&keccak256_hash(&raw_tx))?;
    sig.verify(&msg, &pubkey)?;
    Ok(())
}

fn keccak256_hash(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    hasher.update(bytes);
    let mut resp: [u8; 32] = Default::default();
    hasher.finalize(&mut resp);
    resp
}

#[tokio::test]
#[serial]
async fn can_get_app_configuration() -> Result<()> {
    let config = dbg!(app().configuration().await?);
    assert_eq!("1.10.2", config.version);
    Ok(())
}

#[tokio::test]
#[serial]
async fn can_provide_erc20_info() -> Result<()> {
    let sig1 =
    hex::decode("3045022100fde9f713cb999780a504b8eda31fe0195930935d8b0ad836e183b5c56b5e342d02202c15a0d1ad00b0dacae588524cf4db145934e10c8ba0e89da366e1793f723f70")?;
    let sig = secp256k1::ecdsa::Signature::from_der(&sig1)?;
    let ledger_pubkey_bytes = vec![
        0x04, 0x5e, 0x6c, 0x10, 0x20, 0xc1, 0x4d, 0xc4, 0x64, 0x42, 0xfe, 0x89, 0xf9, 0x7c, 0x0b,
        0x68, 0xcd, 0xb1, 0x59, 0x76, 0xdc, 0x24, 0xf2, 0x4c, 0x31, 0x6e, 0x7b, 0x30, 0xfe, 0x4e,
        0x8c, 0xc7, 0x6b, 0x14, 0x89, 0x15, 0x0c, 0x21, 0x51, 0x4e, 0xbf, 0x44, 0x0f, 0xf5, 0xde,
        0xa5, 0x39, 0x3d, 0x83, 0xde, 0x53, 0x58, 0xcd, 0x09, 0x8f, 0xce, 0x8f, 0xd0, 0xf8, 0x1d,
        0xaa, 0x94, 0x97, 0x91, 0x83,
    ];
    let ledger_pubkey = PublicKey::from_slice(&ledger_pubkey_bytes)?;
    dbg!(hex::encode(ledger_pubkey.serialize_uncompressed()));
    let message = Message::from_hashed_data::<Hash>(&[
        85, 83, 68, 67, 7, 134, 92, 110, 135, 185, 247, 2, 85, 55, 126, 2, 74, 206, 102, 48, 193,
        234, 163, 127, 0, 0, 0, 6, 0, 0, 0, 5,
    ]);
    dbg!(hex::encode(message.as_ref()));
    sig.verify(&message, &ledger_pubkey)?;
    // https://github.com/LedgerHQ/ledger-live/blob/develop/libs/ledgerjs/packages/cryptoassets/src/data/evm/5/erc20.json
    let mut payload = vec![];
    payload.write_u8(4)?;
    payload.extend_from_slice(b"USDC");
    payload.extend_from_slice(&hex::decode("07865c6E87B9F70255377e024ace6630C1Eaa37F")?);
    payload.write_u32::<BigEndian>(6)?;
    payload.write_u32::<BigEndian>(5)?;
    payload.extend_from_slice(&sig1);
    dbg!(hex::encode(&payload));
    app().provide_erc20_token_info(&payload).await?;
    Ok(())
}

#[tokio::test]
#[ignore = "must build eth app without CHAIN=goerli"]
async fn can_test_known_erc20() -> Result<()> {
    let ticker = hex::decode("5a5258")?;
    let addr = hex::decode("e41d2489571d322189246dafa5ebde1f4699f498")?;
    let dec = 18;
    let chain_id = 1;
    let sig = hex::decode("304402200ae8634c22762a8ba41d2acb1e068dcce947337c6dd984f13b820d396176952302203306a49d8a6c35b11a61088e1570b3928ca3a0db6bd36f577b5ef87628561ff7")?;
    let mut payload = vec![];
    payload.write_u8(ticker.len() as u8)?;
    payload.extend_from_slice(&ticker);
    payload.extend_from_slice(&addr);
    payload.write_u32::<BigEndian>(dec)?;
    payload.write_u32::<BigEndian>(chain_id)?;
    payload.extend_from_slice(&sig);

    app().provide_erc20_token_info(&payload).await?;
    Ok(())
}
