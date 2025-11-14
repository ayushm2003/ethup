use serde::Deserialize;
use std::fmt;

pub struct ExecutionStatus {
    pub version: String,
    pub chain_id: u64,
    pub head_block: u64,
    pub sync: ElSyncState,
}

pub enum ElSyncState {
    FullySynced,
    Syncing {
        starting_block: u64,
        current_block: u64,
        highest_block: u64,
        percent: f64,
    },
}

impl fmt::Display for ElSyncState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElSyncState::FullySynced => {
                write!(f, "Fully synced")
            }

            ElSyncState::Syncing {
                starting_block,
                current_block,
                highest_block,
                percent,
            } => {
                if *highest_block == 0 {
                    return write!(f, "execution not started yet");
                }

                write!(
                    f,
                    "{} → {} / {} ({:.2}%)",
                    starting_block, current_block, highest_block, percent
                )
            }
        }
    }
}

pub struct ConsensusStatus {
    pub version: String,
    pub head_slot: u64,
    pub finalized_epoch: Option<u64>,
    pub is_syncing: bool,
    pub health: ClHealth,
}

pub enum ClHealth {
    Healthy,
    Syncing,
    Unhealthy,
    Unknown(u16),
}

impl fmt::Display for ClHealth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClHealth::Healthy => write!(f, "healthy"),
            ClHealth::Syncing => write!(f, "syncing"),
            ClHealth::Unhealthy => write!(f, "unhealthy"),
            ClHealth::Unknown(code) => write!(f, "unknown ({})", code),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ElSyncing {
    NotSyncing(bool),
    Syncing {
        #[serde(rename = "startingBlock")]
        starting_block: String,
        #[serde(rename = "currentBlock")]
        current_block: String,
        #[serde(rename = "highestBlock")]
        highest_block: String,
    },
}

#[derive(Deserialize)]
pub struct ClApi<T> {
    pub data: T,
}

#[derive(Deserialize)]
pub struct ClVersion {
    pub version: String,
}

#[derive(Deserialize)]
pub struct ClSync {
    pub head_slot: String,
    pub is_syncing: bool,
    pub finalized_epoch: Option<String>,
}

#[derive(Deserialize)]
pub struct ClPeers {
    pub connected: u64,
}
