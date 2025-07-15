use crate::config;
use alloy_primitives::{Address, Bytes, FixedBytes};
use alloy_provider::Provider;
use alloy_rpc_types::{BlockId, BlockNumberOrTag, Filter, Log};
use alloy_sol_types::{SolEvent, SolType};
use eyre::WrapErr;
use towns_protocol_contracts::{
    SetMiniblockArray, StreamEventType, StreamState, StreamsRegistry::{self, StreamUpdated}
};
use towns_protocol_types::{StreamId, TownsError};
use std::{cmp::max, collections::{BTreeMap, HashSet}};

/// Get stream inception event
pub(crate) async fn inception(cfg: &config::Config, stream_id: StreamId) -> eyre::Result<()> {
    let provider = cfg
        .river_chain_provider()
        .wrap_err("Invalid River chain RPC URL")?;
    let streams_registry = StreamsRegistry::new(cfg.registry.address, &provider);
    let low = cfg.registry.deployment_block;
    let high = BlockId::Number(BlockNumberOrTag::Number(
        provider
            .get_block_number()
            .await
            .wrap_err("Failed to get block number")?,
    ));

    // binary search for the stream inception block
    let mut low = low.as_u64().unwrap();
    let mut high = high.as_u64().unwrap();

    loop {
        if low > high {
            break;
        }

        let mid = (low + high) / 2;

        let stream = streams_registry
            .getStream(stream_id.as_fixed_bytes32())
            .block(BlockId::Number(BlockNumberOrTag::Number(mid)))
            .call()
            .await;

        if stream.is_err() {
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }

    let block_number = BlockNumberOrTag::Number(low);
    let query = Filter::new()
        .address(cfg.registry.address)
        .from_block(block_number)
        .to_block(block_number);

    let logs = provider.get_logs(&query).await?;

    let print_inception = |stream_id: StreamId,
                           log: &Log,
                           nodes: &[Address],
                           genesis_hash: &FixedBytes<32>,
                           genesis_block: &Option<Bytes>| {
        println!("         stream: {}", stream_id);
        println!("    river block: #{}", log.block_number.unwrap());
        println!("     block hash: {}", log.block_hash.unwrap());
        println!("    transaction: {}", log.transaction_hash.unwrap());
        println!("inititial nodes: {:?}", nodes);
        println!("   genesis hash: {}", genesis_hash);
        if let Some(genesis_block) = genesis_block {
            println!("genesis miniblock:");
            println!("{}", genesis_block);
        }
    };

    let stream_id_fixed_bytes32 = stream_id.as_fixed_bytes32();

    for log in &logs {
        // unified event model with StreamAllocated encoded in StreamUpdated event.
        if let Ok(stream_update) = log.log_decode::<StreamsRegistry::StreamUpdated>() {
            let stream_update_event = stream_update.into_inner();
            if stream_update_event.eventType == 0u8 {
                let stream_state = StreamState::abi_decode_params(&stream_update_event.data.data)?;
                let mut genesis_block = None;

                if stream_state.streamId == stream_id_fixed_bytes32 {
                    if let Ok(stream) = streams_registry
                        .getStreamWithGenesis(stream_id.as_fixed_bytes32())
                        .block(BlockId::Number(block_number))
                        .call()
                        .await
                    {
                        genesis_block = Some(stream._2);
                    }

                    print_inception(
                        stream_id,
                        log,
                        &stream_state.stream.nodes,
                        &stream_state.stream.lastMiniblockHash,
                        &genesis_block,
                    );

                    return Ok(());
                }
            }
        }

        // old event model that emites StreamsRegistry::StreamAllocated
        if matches!(
            log.topic0(),
            Some(&StreamsRegistry::StreamAllocated::SIGNATURE_HASH)
        ) {
            let stream_allocated_event =
                towns_protocol_contracts::StreamAllocated::abi_decode_params(&log.data().data)?;
            if stream_allocated_event.streamId == stream_id.as_fixed_bytes32() {
                print_inception(
                    stream_id,
                    log,
                    &stream_allocated_event.nodes,
                    &stream_allocated_event.genesisMiniblockHash,
                    &Some(stream_allocated_event.genesisMiniblock.clone()),
                );

                return Ok(());
            }
        }
    }

    Err(TownsError::NotFound.into())
}

/// Get stream details
pub(crate) async fn details(cfg: &config::Config, stream_id: StreamId, river_block: Option<u64>) -> eyre::Result<()> {
    let provider = cfg
        .river_chain_provider()
        .wrap_err("Invalid River chain RPC URL")?;
    let streams_registry = StreamsRegistry::new(cfg.registry.address, &provider);
    let block_number = river_block.unwrap_or(provider.get_block_number().await?);

    let stream = streams_registry
        .getStream(stream_id.as_fixed_bytes32())
        .block(BlockId::Number(BlockNumberOrTag::Number(block_number)))
        .call()
        .await
        .wrap_err("Failed to get stream")?;

    println!("     stream: {}", stream_id);
    println!("  miniblock: {}", stream.lastMiniblockNum);
    println!("       hash: {}", stream.lastMiniblockHash);
    println!("      nodes: {:?}", stream.nodes);
    println!("repl factor: {}", stream.replication_factor());
    println!("river block: {}", block_number);

    Ok(())
}

/// Get total number of streams
pub(crate) async fn count(cfg: &config::Config) -> eyre::Result<()> {
    let provider = cfg
        .river_chain_provider()
        .wrap_err("Invalid River chain RPC URL")?;
    let streams_registry = StreamsRegistry::new(cfg.registry.address, &provider);
    let block_number = provider.get_block_number().await?;

    let count = streams_registry
        .getStreamCount()
        .block(BlockId::Number(BlockNumberOrTag::Number(block_number)))
        .call()
        .await
        .wrap_err("Failed to get stream count")?;

    println!("    streams: {}", count);
    println!("river block: {}", block_number);

    Ok(())
}

/// Get stream updates in the last n river blocks
pub(crate) async fn updates(
    cfg: &config::Config,
    stream_id: StreamId,
    scroll_back_river_blocks: u64,
) -> eyre::Result<()> {
    let provider = cfg
        .river_chain_provider()
        .wrap_err("Invalid River chain RPC URL")?;
    let mut to = provider.get_block_number().await?;
    let block_range: u64 = 2_500;
    let river_blocks = scroll_back_river_blocks;
    let first_river_block_to_check = max(0, to - river_blocks);

    loop {
        let from = max(0, to - block_range);
        let from = max(first_river_block_to_check, from);

        let filter = Filter::new()
            .address(cfg.registry.address)
            .from_block(from)
            .to_block(to);

        let logs = provider
            .get_logs(&filter)
            .await
            .wrap_err("failed to get logs")?;

        let stream_id_as_fixed_bytes32 = stream_id.as_fixed_bytes32();

        for log in logs.iter().rev() {
            if let Ok(stream_update) = log.log_decode::<StreamsRegistry::StreamUpdated>() {
                let stream_update_event = stream_update.into_inner();

                let event_type = match StreamEventType::try_from(stream_update_event.eventType) {
                    Ok(event_type) => event_type,
                    Err(_) => continue,
                };

                // println!("tx: {:?} got stream update event: {:?}", log.transaction_hash, event_type);

                match event_type {
                    StreamEventType::Allocate => {
                        let stream_state =
                            StreamState::abi_decode_params(&stream_update_event.data.data)?;

                        if stream_state.streamId != stream_id_as_fixed_bytes32 {
                            continue;
                        }

                        println!(
                            "StreamAllocated river block #{} / tx: {}",
                            log.block_number.unwrap(),
                            log.transaction_hash.unwrap()
                        );

                        return Ok(());
                    }
                    StreamEventType::Create => {
                        let stream_state =
                            StreamState::abi_decode_params(&stream_update_event.data.data)?;

                        if stream_state.streamId != stream_id_as_fixed_bytes32 {
                            continue;
                        }

                        println!(
                            "StreamCreated river block #{} / tx: {}",
                            log.block_number.unwrap(),
                            log.transaction_hash.unwrap()
                        );

                        return Ok(());
                    }
                    StreamEventType::PlacementUpdated => {
                        let stream_state =
                            StreamState::abi_decode_params(&stream_update_event.data.data)?;

                        if stream_state.streamId != stream_id_as_fixed_bytes32 {
                            continue;
                        }

                        println!(
                            "PlacementUpdate nodes: {:?} / replication factor: {} / river block #{} / tx: {} ",
                            stream_state.stream.nodes,
                            stream_state.replication_factor(),
                            log.block_number.unwrap(),
                            log.transaction_hash.unwrap(),
                        );

                        continue;
                    }
                    StreamEventType::LastMiniblockBatchUpdated => {
                        let miniblock_updates =
                            SetMiniblockArray::abi_decode_params(&stream_update_event.data.data)
                                .map_err(|e| {
                                    TownsError::InvalidStreamUpdatedEvent(
                                        e.to_string(),
                                        log.transaction_hash.unwrap(),
                                        log.log_index.unwrap(),
                                    )
                                })?;

                        miniblock_updates.iter().for_each(|mb| {
                            let update_stream_id = StreamId::from(&mb.streamId);
                            if update_stream_id != stream_id {
                                return;
                            }

                            println!("MiniblockUpdated miniblock_num: {} miniblock_hash: {} / river block #{} / tx: {} ",
                                mb.lastMiniblockNum,
                                mb.lastMiniblockHash.to_string(),
                                log.block_number.unwrap(),
                                log.transaction_hash.unwrap(),
                            );
                        });

                        continue;
                    }
                    _ => {
                        return Err(TownsError::InvalidStreamUpdatedEvent(
                            "Invalid stream update event type".to_string(),
                            log.transaction_hash.unwrap(),
                            log.log_index.unwrap(),
                        )
                        .into());
                    }
                }
            }

            let update = StreamUpdated::decode_log_data(log.data());
            if update.is_err() {
                continue;
            }

            let update = update.unwrap();
            let event_type = match StreamEventType::try_from(update.eventType) {
                Ok(event_type) => event_type,
                Err(_) => continue,
            };

            match event_type {
                StreamEventType::Allocate => {
                    let stream_state =
                        StreamState::abi_decode_params(&update.data).map_err(|e| {
                            TownsError::InvalidStreamUpdatedEvent(
                                e.to_string(),
                                log.transaction_hash.unwrap(),
                                log.log_index.unwrap(),
                            )
                        })?;

                    if stream_state.streamId != stream_id.as_fixed_bytes32() {
                        continue;
                    }

                    println!(
                        "StreamAllocated river block #{} / tx: {}",
                        log.block_number.unwrap(),
                        log.transaction_hash.unwrap()
                    );

                    return Ok(());
                }
                StreamEventType::Create => {
                    let stream_state =
                        StreamState::abi_decode_params(&update.data).map_err(|e| {
                            TownsError::InvalidStreamUpdatedEvent(
                                e.to_string(),
                                log.transaction_hash.unwrap(),
                                log.log_index.unwrap(),
                            )
                        })?;

                    if stream_state.streamId != stream_id.as_fixed_bytes32() {
                        continue;
                    }

                    println!(
                        "StreamCreated river block #{} / tx: {}",
                        log.block_number.unwrap(),
                        log.transaction_hash.unwrap()
                    );

                    return Ok(());
                }
                StreamEventType::PlacementUpdated => {
                    let stream_state =
                        StreamState::abi_decode_params(&update.data).map_err(|e| {
                            TownsError::InvalidStreamUpdatedEvent(
                                e.to_string(),
                                log.transaction_hash.unwrap(),
                                log.log_index.unwrap(),
                            )
                        })?;

                    if stream_state.streamId != stream_id.as_fixed_bytes32() {
                        continue;
                    }

                    println!(
                        "PlacementUpdate nodes: {:?} / replication factor: {} / river block #{} / tx: {} ",
                        stream_state.stream.nodes,
                        stream_state.replication_factor(),
                        log.block_number.unwrap(),
                        log.transaction_hash.unwrap(),
                    );
                }
                StreamEventType::LastMiniblockBatchUpdated => {
                    let miniblock_updates =
                        SetMiniblockArray::abi_decode_params(&update.data).map_err(|e| {
                            TownsError::InvalidStreamUpdatedEvent(
                                e.to_string(),
                                log.transaction_hash.unwrap(),
                                log.log_index.unwrap(),
                            )
                        })?;

                    miniblock_updates.iter().for_each(|mb| {
                        let update_stream_id = StreamId::from(&mb.streamId);
                        if update_stream_id != stream_id {
                            return;
                        }

                        println!("MiniblockUpdated miniblock_num: {} miniblock_hash: {} / river block #{} / tx: {} ",
                            mb.lastMiniblockNum,
                            mb.lastMiniblockHash.to_string(),
                            log.block_number.unwrap(),
                            log.transaction_hash.unwrap(),
                        );
                    });
                }
                _ => {
                    return Err(TownsError::InvalidStreamUpdatedEvent(
                        "invalid stream update event type".to_string(),
                        log.transaction_hash.unwrap(),
                        log.log_index.unwrap(),
                    )
                    .into());
                }
            }
        }

        if from == 0 || from == first_river_block_to_check {
            break;
        }

        to = to - block_range - 1;
    }

    Ok(())
}

pub(crate) async fn active_streams(
    cfg: &config::Config, 
    scroll_back_hours: u64, 
    stream_types: &Vec<u8>, 
    mut hot_duration_hours: Vec<u64>,
) -> eyre::Result<()> {
    if hot_duration_hours.is_empty() {
        hot_duration_hours = vec![4];
    }

    let provider = cfg
        .river_chain_provider()
        .wrap_err("Invalid River chain RPC URL")?;
    
    let highest_hot_duration_h = hot_duration_hours.iter().cloned().fold(0, u64::max);
    let block_range_1h = 1800;
    let last = (provider.get_block_number().await? / block_range_1h) * block_range_1h;
    let history = block_range_1h * (scroll_back_hours + highest_hot_duration_h + 1);
    let first = if history < last { last-history } else { 0 };
    
    let mut river_block_buckets: BTreeMap<u64, HashSet<StreamId>> = BTreeMap::new();
    
    for from in (first..last).step_by(block_range_1h as usize) {
        let to = from+block_range_1h -1;

        let filter = Filter::new()
            .address(cfg.registry.address)
            .from_block(from)
            .to_block(to);

        let logs = provider
            .get_logs(&filter)
            .await
            .wrap_err("failed to get logs")?;

        eprintln!("from: {} / to: {} / logs: {}", from, to, logs.len());

        for log in logs.iter() {
            if let Ok(stream_update) = log.log_decode::<StreamsRegistry::StreamUpdated>() {
                let stream_update_event = stream_update.into_inner();
                let event_type = match StreamEventType::try_from(stream_update_event.eventType) {
                    Ok(event_type) => event_type,
                    Err(_) => continue,
                };

                if event_type == StreamEventType::LastMiniblockBatchUpdated {
                    let miniblock_updates = SetMiniblockArray::abi_decode_params(&stream_update_event.data.data)
                        .wrap_err("failed to decode miniblock updates")?;

                    miniblock_updates.iter().for_each(|mb| {
                        let stream_id: StreamId = match StreamId::try_from(&mb.streamId)   {
                            Ok(stream_id) => stream_id,
                            Err(err) => panic!("invalid stream id: {}", err),
                        };

                        if stream_types.is_empty() || stream_types.contains(&stream_id.stream_type()) {
                            let bucket_key = block_range_1h * (log.block_number.unwrap() / block_range_1h);
                            if let Some(streams) = river_block_buckets.get_mut(&bucket_key) {
                                streams.insert(stream_id);
                            } else {
                                river_block_buckets.insert(bucket_key, HashSet::from([stream_id]));
                            }
                        }
                    });
                }
            }
        }
    }

    print!("river_block");
    for hot_duration_h in hot_duration_hours.iter() {
        print!(",hot_duration_{}_h", hot_duration_h);
    }
    println!();

    for (block, streams) in river_block_buckets.iter().rev().take(scroll_back_hours as usize) {
        print!("{block}",);
        for hot_duration_h in hot_duration_hours.iter() {
            // get the unique streams that have seen activity in the last hot_duration buckets (hot_duration hour).
            let mut unique_streams = streams.clone();

            for i in 1..*hot_duration_h {
                let bucket_key = block - (i*block_range_1h);
                if let Some((_, bucket)) = river_block_buckets.get_key_value(&bucket_key) {
                    unique_streams = unique_streams.union(bucket).cloned().collect();
                }
            }
            print!(",{}", unique_streams.len());
        }
        println!();
    }

    Ok(())
}
