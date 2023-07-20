mod handler;
mod server;

pub use crate::handler::EthRequestHandler;

pub type EthRequest = rethnet_eth::remote::methods::MethodInvocation;
