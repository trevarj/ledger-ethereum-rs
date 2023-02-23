use ledger_transport::{APDUCommand, APDUErrorCode, Exchange};
use ledger_zondax_generic::{App, LedgerAppError};

use crate::command::InstructionCode;
use crate::types::{BIP44Path, ChunkPayloadType, EthError, LedgerEthTransactionResolution};
use crate::EthApp;

#[derive(Debug)]
pub struct Signature {
    pub v: u8,
    pub r: [u8; 32],
    pub s: [u8; 32],
}

impl<E> EthApp<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    /// Sign a transaction
    pub async fn sign(
        &self,
        path: &BIP44Path,
        raw_tx: &[u8],
        // TODO: come back to this later and see if we can resolve txns instead of blind signing
        _resolution: Option<LedgerEthTransactionResolution>,
    ) -> Result<Signature, EthError<E::Error>> {
        let mut data = vec![];
        let path = path.serialize_bip44();
        data.extend_from_slice(&path);
        data.extend_from_slice(raw_tx);

        let command = APDUCommand {
            cla: Self::CLA,
            ins: InstructionCode::SignTransaction as _,
            p1: ChunkPayloadType::First as u8,
            p2: 0x00,
            data,
        };

        let response = self.send_chunks(command).await?;

        let response_data = response.data();
        match response.error_code() {
            Ok(APDUErrorCode::NoError) if response_data.is_empty() => {
                return Err(EthError::Ledger(LedgerAppError::NoSignature))
            }
            // Last response should contain the answer
            Ok(APDUErrorCode::NoError) if response_data.len() < 3 => {
                return Err(EthError::Ledger(LedgerAppError::InvalidSignature))
            }
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

        let v = response_data
            .first()
            .ok_or(EthError::MissingResponseData(
                "signature v component".into(),
            ))?
            .to_owned();
        let r = response_data
            .get(1..33)
            .ok_or(EthError::MissingResponseData(
                "signature r component".into(),
            ))?
            .try_into() // safe due to get() range
            .unwrap();
        let s = response_data
            .get(33..65)
            .ok_or(EthError::MissingResponseData(
                "signature s component".into(),
            ))?
            .try_into() // safe due to get() range
            .unwrap();
        Ok(Signature { v, r, s })
    }
}
