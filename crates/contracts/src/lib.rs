use alloy::sol;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc = true, abi = true, all_derives = true, extra_methods = true)]
    #[derive(Debug)]
    StreamsRegistry,
    "abi/StreamsRegistry.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc = true, abi = true, all_derives = true, extra_methods = true)]
    #[derive(Debug)]
    NodeRegistry,
    "abi/NodeRegistry.json"
);

sol! {
    #[allow(missing_docs)]
    #[derive(Debug,Eq,PartialEq)]
    #[sol(rpc=true,abi=true,all_derives=true, extra_methods=true)]
    enum StreamEventType {
        Allocate,
        Create,
        PlacementUpdated,
        LastMiniblockBatchUpdated
    }

    #[allow(missing_docs)]
    #[derive(Debug)]
    #[sol(rpc=true,abi=true,all_derives=true, extra_methods=true)]
    struct Stream {
      bytes32 lastMiniblockHash; // 32 bytes, slot 0
      uint64 lastMiniblockNum; // 8 bytes, part of slot 1
      uint64 reserved0; // 8 bytes, part of slot 1
      uint64 flags; // 8 bytes, part of slot 1
      address[] nodes; // Dynamic array, starts at a new slot
    }

    #[allow(missing_docs)]
    #[derive(Debug)]
    #[sol(rpc=true,abi=true,all_derives=true, extra_methods=true)]
    struct StreamState {
        bytes32 streamId;
        Stream stream;
    }

    #[allow(missing_docs)]
    #[derive(Debug)]
    #[sol(rpc=true,abi=true,all_derives=true,extra_methods=true)]
    struct SetMiniblock {
        bytes32 streamId;
        bytes32 prevMiniBlockHash;
        bytes32 lastMiniblockHash;
        uint64 lastMiniblockNum;
        bool isSealed;
    }

    #[allow(missing_docs)]
    #[derive(Debug)]
    #[sol(rpc=true,abi=true,all_derives=true, extra_methods=true)]
    struct StreamAllocated {
        bytes32 streamId;
        address[] nodes;
        bytes32 genesisMiniblockHash;
        bytes genesisMiniblock;
    }
}

pub type SolArrayOf<T> = sol! { T[] };

pub type SetMiniblockArray = SolArrayOf<SetMiniblock>;

impl StreamsRegistry::Stream {
    pub fn replication_factor(&self) -> u64 {
        let repl_factor = self.reserved0 & 0xFF;
        match repl_factor {
            0 => 1, // backwards compatibility before replicated streams were introduced
            _ => repl_factor,
        }
    }
}

impl StreamState {
    pub fn replication_factor(&self) -> u64 {
        let repl_factor = self.stream.reserved0 & 0xFF;
        match repl_factor {
            0 => 1, // backwards compatibility before replicated streams were introduced
            _ => repl_factor,
        }
    }
}
