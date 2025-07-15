use alloy_primitives::{Address, address};
use alloy_provider::{Provider, ProviderBuilder};
use alloy_rpc_types::{BlockId, BlockNumberOrTag};
use clap::ValueEnum;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub(crate) enum Network {
    Alpha,
    Delta,
    Gamma,
    Omega,
}

pub(crate) struct Registry {
    pub address: Address,
    pub deployment_block: BlockId,
}

pub(crate) struct Config {
    pub river_rpc_url: String,
    pub registry: Registry,
}

impl Config {
    // river_chain_provider returns an alloy provider for the river chain.
    pub(crate) fn river_chain_provider(&self) -> eyre::Result<impl Provider> {
        let url = self.river_rpc_url.parse()?;
        Ok(ProviderBuilder::new().connect_http(url))
    }
}

pub(crate) fn config(network: Network) -> Config {
    match network {
        Network::Alpha => Config {
            river_rpc_url: "https://testnet.rpc.towns.com/http".to_string(),
            registry: Registry {
                address: address!("0x44354786eacbebf981453a05728e1653bc3c5def"),
                deployment_block: BlockId::Number(BlockNumberOrTag::Number(10499921)),
            },
        },
        Network::Delta => Config {
            river_rpc_url: "https://testnet.rpc.towns.com/http".to_string(),
            registry: Registry {
                address: address!("0x9Db19dB285cEd37099D40d27D51B75C4dFa05652"),
                deployment_block: BlockId::Number(BlockNumberOrTag::Number(15296357)),
            },
        },
        Network::Gamma => Config {
            river_rpc_url: "https://testnet.rpc.towns.com/http".to_string(),
            registry: Registry {
                address: address!("0xf18E98D36A6bd1aDb52F776aCc191E69B491c070"),
                deployment_block: BlockId::Number(BlockNumberOrTag::Number(4577770)),
            },
        },
        Network::Omega => Config {
            river_rpc_url: "https://mainnet.rpc.towns.com/http".to_string(),
            registry: Registry {
                address: address!("0x1298c03Fde548dc433a452573E36A713b38A0404"),
                deployment_block: BlockId::Number(BlockNumberOrTag::Number(134180)),
            },
        },
    }
}
