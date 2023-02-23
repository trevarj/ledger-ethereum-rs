pub(crate) mod get_address;
pub(crate) mod get_app_configuration;
pub(crate) mod provide_erc20_token_info;
pub(crate) mod sign_transaction;

#[derive(Debug)]
#[repr(u8)]
pub enum InstructionCode {
    GetAddress = 0x02,
    SignTransaction = 0x04,
    GetAppConfiguration = 0x06,
    ProvideErc20TokenInfo = 0x0A,
}
