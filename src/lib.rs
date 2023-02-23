pub mod command;
pub mod types;

use ledger_transport::{APDUCommand, APDUErrorCode, Exchange};
use ledger_zondax_generic::{App, LedgerAppError};
use types::ChunkPayloadType;

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
    pub async fn send_chunks(
        &self,
        mut command: APDUCommand<Vec<u8>>,
    ) -> Result<ledger_transport::APDUAnswer<E::AnswerType>, LedgerAppError<E::Error>> {
        let chunks = command
            .data
            .chunks(250)
            .map(|c| c.to_vec())
            .collect::<Vec<Vec<u8>>>();
        match chunks.len() {
            0 => return Err(LedgerAppError::InvalidEmptyMessage),
            n if n > 255 => return Err(LedgerAppError::InvalidMessageSize),
            _ => (),
        }

        let (first, rest) = chunks.split_first().unwrap();
        command.data = first.to_owned();

        if command.p1 != ChunkPayloadType::First as u8 {
            return Err(LedgerAppError::InvalidChunkPayloadType);
        }

        let mut response = self.transport.exchange(&command).await?;
        match response.error_code() {
            Ok(APDUErrorCode::NoError) => {}
            Ok(err) => return Err(LedgerAppError::AppSpecific(err as _, err.description())),
            Err(err) => return Err(LedgerAppError::Unknown(err as _)),
        }

        // Send message chunks
        let p1 = ChunkPayloadType::Subsequent as u8;
        for chunk in rest {
            dbg!(&chunk);
            let command = APDUCommand {
                cla: command.cla,
                ins: command.ins,
                p1,
                p2: 0,
                data: chunk.to_vec(),
            };

            response = self.transport.exchange(&command).await?;
            match response.error_code() {
                Ok(APDUErrorCode::NoError) => {}
                Ok(err) => return Err(LedgerAppError::AppSpecific(err as _, err.description())),
                Err(err) => return Err(LedgerAppError::Unknown(err as _)),
            }
        }

        Ok(response)
    }
}
