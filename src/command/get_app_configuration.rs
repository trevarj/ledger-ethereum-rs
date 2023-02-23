use ledger_transport::{APDUCommand, Exchange};
use ledger_zondax_generic::App;

use crate::command::InstructionCode;
use crate::types::EthError;
use crate::{EthApp, LedgerAppError};

#[derive(Debug)]
pub struct AppConfiguration {
    pub arbitrary_data_enabled: bool,
    pub erc20_provisioning_necessary: bool,
    pub stark_enabled: bool,
    pub stark_v2_supported: bool,
    pub version: String,
}

impl<E> EthApp<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    /// Retrieves the app configuration
    // https://github.com/LedgerHQ/app-ethereum/blob/develop/doc/ethapp.adoc#get-app-configuration
    pub async fn configuration(&self) -> Result<AppConfiguration, EthError<E::Error>> {
        let command = APDUCommand {
            cla: Self::CLA,
            ins: InstructionCode::GetAppConfiguration as u8,
            p1: 0,
            p2: 0,
            data: vec![],
        };
        let response = self
            .transport
            .exchange(&command)
            .await
            .map_err(LedgerAppError::TransportError)?;
        let response_data = response.data();

        Ok(AppConfiguration {
            arbitrary_data_enabled: response_data[0] & 0x01 == 0x01,
            erc20_provisioning_necessary: response_data[0] & 0x02 == 0x02,
            stark_enabled: response_data[0] & 0x04 == 0x04,
            stark_v2_supported: response_data[0] & 0x08 == 0x08,
            version: format!(
                "{}.{}.{}",
                response_data[1], response_data[2], response_data[3]
            ),
        })
    }
}
