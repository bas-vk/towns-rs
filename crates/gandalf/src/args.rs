use alloy_primitives::{Address, U256};
use alloy_rpc_types::{BlockId, BlockNumberOrTag};
use crate::{config, stream};
use alloy_provider::Provider;
use clap::{Args, Parser, Subcommand, value_parser};
use towns_protocol_contracts::{NodeRegistry, StreamsRegistry};
use towns_protocol_types::StreamId;
use eyre::WrapErr;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub(crate) struct Opts {
    #[arg(short,long,value_enum, default_value_t = config::Network::Omega, env = "TOWNS_GANDALF_NETWORK")]
    pub network: config::Network,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Stream(StreamArgs),
    Miniblock(MiniblockArgs),
    Node(NodeArgs),
}

#[derive(Debug, Args)]
#[command(
    args_conflicts_with_subcommands = true,
    about = "All stream related commands."
)]
pub(crate) struct StreamArgs {
    #[command(subcommand)]
    pub command: StreamCommands,
}

impl StreamArgs {
    pub(crate) async fn execute(self, cfg: &config::Config) -> eyre::Result<()> {
        match self.command {
            StreamCommands::Inception { stream_id } => stream::inception(cfg, stream_id).await,
            StreamCommands::Details { stream_id, river_block } => stream::details(cfg, stream_id, river_block).await,
            StreamCommands::Count {} => stream::count(cfg).await,
            StreamCommands::Updates {
                stream_id,
                scroll_back_river_blocks,
            } => stream::updates(cfg, stream_id, scroll_back_river_blocks).await,
            StreamCommands::ActiveStreams { scroll_back_hours, stream_types, hot_duration_hours } => stream::active_streams(cfg, scroll_back_hours, &stream_types, hot_duration_hours).await,
        }
    }
}   
#[derive(Debug, Subcommand)]
pub(crate) enum StreamCommands {
    #[command(about = "Print stream inception details")]
    Inception {
        #[arg(value_parser=value_parser!(StreamId))]
        stream_id: StreamId,
    },
    #[command(about = "Print stream details")]
    Details {
        #[arg(value_parser=value_parser!(StreamId))]
        stream_id: StreamId,
        river_block: Option<u64>,
    },
    #[command(about = "Print total number of streams")]
    Count {},

    #[command(
        about = "Print stream updates in the last n river blocks, if not given defaults to 10000"
    )]
    Updates {
        #[arg(value_parser=value_parser!(StreamId))]
        stream_id: StreamId,
        #[arg(short,long,help="the number of river blocks to scroll back, defaults to 10000", value_parser=value_parser!(u64), default_value_t = 10000)]
        scroll_back_river_blocks: u64,
    },
    #[command(about = "Print the number of streams that got miniblocks in the last n river blocks")]
    ActiveStreams {
        #[arg(short,long,help="the number of hours to scroll back, defaults to 168 (1 week)", value_parser=value_parser!(u64), default_value_t = 168)]
        scroll_back_hours: u64,
        #[arg(short='t',long,help="the stream types to filter by, defaults to all", value_parser=value_parser!(u8))]
        stream_types: Vec<u8>,
        #[arg(short='d',long,help="how many hours before a stream is considered cold (default 4)", value_parser=value_parser!(u64))]
        hot_duration_hours: Vec<u64>,
    }
}

#[derive(Debug, Args)]
#[command(
    args_conflicts_with_subcommands = true,
    about = "Get, validate miniblocks."
)]
pub(crate) struct MiniblockArgs {
    #[command(subcommand)]
    pub command: MiniblockCommands,
}

impl MiniblockArgs {
    pub(crate) async fn execute(self, _cfg: &config::Config) -> eyre::Result<()> {
        todo!()
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum MiniblockCommands {
    #[command(about = "Get miniblock")]
    Get {
        #[arg(value_parser=value_parser!(StreamId))]
        stream_id: StreamId,
        miniblock_hash: String,
    },
}

#[derive(Debug, Args)]
#[command(
    args_conflicts_with_subcommands = true,
    about = "Get node information."
)]
pub(crate) struct NodeArgs {
    #[command(subcommand)]
    pub command: NodeCommands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum NodeCommands {
    #[command(about = "Print total number of streams on a node")]
    NodeStreamCount { node_addr: String },
    #[command(about = "Print total number of streams on all nodes")]
    AllNodeStreamCount {},
}

struct NodeStreamCount {
    address: Address,
    operator: Address,
    stream_count: U256,
    status: u8,
    url: String,
}

impl NodeArgs {
    pub(crate) async fn execute(self, cfg: &config::Config) -> eyre::Result<()> {
        match self.command {
            NodeCommands::AllNodeStreamCount {} => self.all_node_stream_count(&cfg).await,
            NodeCommands::NodeStreamCount { .. } => todo!()
        }
    }

    pub(crate) async fn all_node_stream_count(&self, cfg: &config::Config) -> eyre::Result<()> {
        let provider = cfg
            .river_chain_provider()
            .wrap_err("Invalid River chain RPC URL") ?;
        let node_registry = NodeRegistry::new(cfg.registry.address, & provider);
        let stream_registry = StreamsRegistry::new(cfg.registry.address, & provider);
        let block = BlockId::Number(BlockNumberOrTag::Number(
            provider
                .get_block_number()
                .await
                .wrap_err("Failed to get block number")?,
        ));

        let total_stream_count = stream_registry.getStreamCount()
            .block(block)
            .call()
            .await.wrap_err("Failed to get total stream count")?;

        let nodes = node_registry.getAllNodes()
            .block(block)
            .call()
            .await.wrap_err("Failed to get all nodes")?;

        let mut result = Vec::new();

        for node in nodes {
            let count = stream_registry.getStreamCountOnNode(node.nodeAddress)
                .block(block)
                    .call()
                .await.wrap_err("Failed to get node count")?;

            result.push(NodeStreamCount{
                address: node.nodeAddress,
                operator: node.operator,
                stream_count: count,
                status: node.status,
                url: node.url
            })
        }

        result.sort_by(|a, b| b.stream_count.cmp(&a.stream_count));

        println!("{:<10}{:<45}{:<45}{:<10}{}", "#streams", "node", "operator", "status", "url");

        let mut total = U256::from(0);
        for node in result.iter() {
            total += node.stream_count;
            println!("{:<10}{:<45}{:<45}{:<10}{}",
                     node.stream_count, node.address.to_string(), node.operator.to_string(), node.status, node.url);
        }

        println!("--------------------------------------------------");
        println!("river block: {} | total streams: {total_stream_count} | incl replicated: {total}", block.as_u64().unwrap());

        Ok(())
    }
}