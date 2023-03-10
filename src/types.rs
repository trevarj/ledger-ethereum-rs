use ledger_zondax_generic::LedgerAppError;

/// Ethereum Ledger Error
#[derive(Debug, thiserror::Error)]
pub enum EthError<E: std::error::Error> {
    #[error("Ledger | {0}")]
    /// Common Ledger errors
    Ledger(#[from] LedgerAppError<E>),

    /// Missing response data part
    #[error("Missing response data: {0}")]
    MissingResponseData(String),

    /// Miscellaneous error
    #[error("{0}")]
    Other(String),
}

/// Chunk payload type
pub enum ChunkPayloadType {
    /// First chunk
    First = 0x00,
    /// Subsequent chunk
    Subsequent = 0x80,
}

/// BIP44 Path
#[derive(Debug)]
pub struct BIP44Path {
    /// Purpose
    pub purpose: u32,
    /// Coin
    pub coin: u32,
    /// Account
    pub account: u32,
    /// Change
    pub change: u32,
    /// Address Index
    pub index: u32,
}

impl BIP44Path {
    /// Serialize a [`BIP44Path`] in the format used in the app
    pub fn serialize_bip44(&self) -> Vec<u8> {
        use byteorder::{BigEndian, WriteBytesExt};
        let mut m = Vec::new();

        m.write_u8(5).unwrap(); // number of path components
        m.write_u32::<BigEndian>(self.purpose).unwrap();
        m.write_u32::<BigEndian>(self.coin).unwrap();
        m.write_u32::<BigEndian>(self.account).unwrap();
        m.write_u32::<BigEndian>(self.change).unwrap();
        m.write_u32::<BigEndian>(self.index).unwrap();

        m
    }
}

#[derive(Debug)]
pub struct LedgerEthTransactionResolution {
    /// Device serialized data that contains ERC20 data (hex format)
    pub erc20_tokens: Vec<String>,
    /// Device serialized data that contains NFT data (hex format)
    pub nfts: Vec<String>,
    /// Device serialized data that contains external plugin data (hex format)
    pub external_plugins: Vec<ExternalPluginData>,
    /// Device serialized data that contains plugin data (hex format)
    pub plugin: Vec<String>,
}

#[derive(Debug)]
pub struct ExternalPluginData {
    pub payload: String,
    pub signature: String,
}
