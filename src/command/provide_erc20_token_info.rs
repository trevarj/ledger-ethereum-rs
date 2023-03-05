use ledger_transport::{APDUCommand, APDUErrorCode, Exchange};
use ledger_zondax_generic::App;

use crate::command::InstructionCode;
use crate::types::EthError;
use crate::{EthApp, LedgerAppError};

impl<E> EthApp<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    /// This command provides a trusted description of an ERC 20 token to
    /// associate a contract address with a ticker and number of decimals.
    /// It shall be run immediately before performing a transaction involving a
    /// contract calling this contract address to display the proper token
    /// information to the user if necessary, as marked in GET APP CONFIGURATION
    /// flags. The signature is computed on
    /// ticker || address || number of decimals (uint4be) || chainId (uint4be)
    /// signed by the following secp256k1 public key
    /// 0482bbf2f34f367b2e5bc21847b6566f21f0976b22d3388a9a5e446ac62d25cf725b62a2555b2dd464a4da0ab2f4d506820543af1d242470b1b1a969a27578f353
    pub async fn provide_erc20_token_info(&self, data: &[u8]) -> Result<(), EthError<E::Error>> {
        let command = APDUCommand {
            cla: Self::CLA,
            ins: InstructionCode::ProvideErc20TokenInfo as _,
            p1: 0,
            p2: 0,
            data,
        };
        let response = self
            .transport
            .exchange(&command)
            .await
            .map_err(LedgerAppError::TransportError)?;
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
        Ok(())
    }
}
