use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ConnectionType {
   Standalone,
   ReplicaSet,
   ShardedCluster,
   DnsSeedList
}