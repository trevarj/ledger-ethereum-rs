use ledger_transport::{APDUCommand, APDUErrorCode, Exchange};
use ledger_zondax_generic::App;

use crate::command::InstructionCode;
use crate::types::{BIP44Path, EthError};
use crate::{EthApp, LedgerAppError};

#[derive(Debug)]
pub struct Address {
    /// Secp256k1 pubkey bytes
    pub public_key: Vec<u8>,
    /// Address bytes in raw UTF-8, without "0x" prefix
    pub address: Vec<u8>,
    /// Optional chain code bytes
    pub chain_code: Option<Vec<u8>>,
}

impl<E> EthApp<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    /// Retrieves the public key and address
    pub async fn address(
        &self,
        path: &BIP44Path,
        enable_display: Option<bool>,
        enabled_chain_code: Option<bool>,
    ) -> Result<Address, EthError<E::Error>> {
        let data = path.serialize_bip44();
        let p1 = enable_display.map_or(0, |v| v as u8);
        let p2 = enabled_chain_code.map_or(0, |v| v as u8);

        let command = APDUCommand {
            cla: Self::CLA,
            ins: InstructionCode::GetAddress as _,
            p1,
            p2,
            data,
        };

        let response = self
            .transport
            .exchange(&command)
            .await
            .map_err(LedgerAppError::TransportError)?;

        let response_data = response.data();
        match response.error_code() {
            Ok(APDUErrorCode::NoError) => {}
            Ok(err) => {
                return Err(EthError::Ledger(LedgerAppError::AppSpecific(
                    err as _,
                    err.description(),
                )))
            }
            Err(err) => {
                return Err(EthError::Ledger(LedgerAppError::AppSpecific(
                    err,
                    "[APDU_ERROR] Unknown".to_string(),
                )))
            }
        }

        let public_key_len: usize = (*response_data
            .first()
            .ok_or(EthError::MissingResponseData("pubkey length".into()))?)
        .into();

        let pubkey_start = 1;
        let pubkey_end = pubkey_start + public_key_len;
        let public_key = response_data
            .get(pubkey_start..pubkey_end)
            .ok_or(EthError::MissingResponseData("public key".into()))?
            .to_vec();

        let address_len: usize = (*response_data
            .get(pubkey_end)
            .ok_or(EthError::MissingResponseData("address length".into()))?)
        .into();
        let address_start = pubkey_end + 1;
        let address_end = address_start + address_len;
        let address = response_data
            .get(address_start..address_end)
            .ok_or(EthError::MissingResponseData("address".into()))?
            .to_vec();

        let chain_code = if let Some(true) = enabled_chain_code {
            let cc_start = address_end + 1;
            let cc_end = address_start + address_len;
            Some(
                response_data
                    .get(cc_start..cc_end)
                    .ok_or(EthError::MissingResponseData("chain code".into()))?
                    .to_vec(),
            )
        } else {
            None
        };
        Ok(Address {
            public_key,
            address,
            chain_code,
        })
    }
}
