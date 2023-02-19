pub mod types;

use ledger_transport::{APDUCommand, APDUErrorCode, Exchange};
use ledger_zondax_generic::{App, AppExt, ChunkPayloadType, LedgerAppError, Version};
use types::{
    BIP44Path, EthError, GetAddressResponse, InstructionCode, LedgerEthTransactionResolution,
    Signature,
};

// https://github.com/LedgerHQ/app-ethereum/blob/develop/doc/ethapp.adoc#general-purpose-apdus
// https://github.com/LedgerHQ/ledger-live/blob/develop/libs/ledgerjs/packages/hw-app-eth/src/Eth.ts
#[derive(Debug)]
pub struct EthApp<E: Exchange> {
    transport: E,
}

impl<E: Exchange> App for EthApp<E> {
    const CLA: u8 = 0xe0;
}

impl<E: Exchange> EthApp<E> {
    /// Create a new [`EthApp`] with the given transport
    pub const fn new(transport: E) -> Self {
        EthApp { transport }
    }
}

impl<E> EthApp<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    /// Retrieve the app version
    pub async fn version(&self) -> Result<Version, EthError<E::Error>> {
        <Self as AppExt<E>>::get_version(&self.transport)
            .await
            .map_err(Into::into)
    }

    /// Retrieves the public key and address
    pub async fn address(
        &self,
        path: &BIP44Path,
        enable_display: Option<bool>,
        enabled_chain_code: Option<bool>,
    ) -> Result<GetAddressResponse, EthError<E::Error>> {
        let serialized_path = path.serialize_bip44();
        let p1 = enable_display.map_or(0, |v| v as u8);
        let p2 = enabled_chain_code.map_or(0, |v| v as u8);

        let command = APDUCommand {
            cla: Self::CLA,
            ins: InstructionCode::GetAddress as _,
            p1,
            p2,
            data: serialized_path,
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
        Ok(GetAddressResponse {
            public_key,
            address,
            chain_code,
        })
    }

    /// Sign a transaction
    pub async fn sign(
        &self,
        path: &BIP44Path,
        raw_tx_hex: &[u8],
        // TODO: come back to this later and see if we can resolve txns instead of blind signing
        _resolution: Option<LedgerEthTransactionResolution>,
    ) -> Result<Signature, EthError<E::Error>> {
        let bip44path = path.serialize_bip44();

        let start_command = APDUCommand {
            cla: Self::CLA,
            ins: InstructionCode::SignTransaction as _,
            p1: ChunkPayloadType::Init as u8,
            p2: 0x00,
            data: bip44path,
        };

        let response =
            <Self as AppExt<E>>::send_chunks(&self.transport, start_command, raw_tx_hex).await?;

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
