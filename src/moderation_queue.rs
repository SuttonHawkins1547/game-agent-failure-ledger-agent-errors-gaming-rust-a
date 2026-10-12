use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentFailure {
    AssetPolicy {
        asset_id: String,
        player_id: String,
        reason: String,
    },
    LiveEventSafety {
        event_id: String,
        player_id: String,
        reason: String,
    },
    ModelDecision {
        run_id: String,
        reason: String,
    },
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueuePriority {
    Normal,
    Critical,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ModerationTicket {
    pub subject_id: String,
    pub priority: QueuePriority,
}

impl AgentFailure {
    pub fn moderation_ticket(&self) -> ModerationTicket {
        match self {
            Self::LiveEventSafety { event_id, .. } => ModerationTicket {
                subject_id: event_id.clone(),
                priority: QueuePriority::Critical,
            },
            Self::AssetPolicy { asset_id, .. } => ModerationTicket {
                subject_id: asset_id.clone(),
                priority: QueuePriority::Normal,
            },
            Self::ModelDecision { run_id, .. } => ModerationTicket {
                subject_id: run_id.clone(),
                priority: QueuePriority::Normal,
            },
        }
    }

    pub fn idempotency_key(&self) -> String {
        let ticket = self.moderation_ticket();
        format!("agent-failure:{}", ticket.subject_id)
    }
}

impl fmt::Display for AgentFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AssetPolicy { asset_id, player_id, reason } => write!(
                f,
                "asset moderation failed: asset_id={asset_id} player_id={player_id} reason={reason}"
            ),
            Self::LiveEventSafety { event_id, player_id, reason } => write!(
                f,
                "live event moderation failed: event_id={event_id} player_id={player_id} reason={reason}"
            ),
            Self::ModelDecision { run_id, reason } => {
                write!(f, "agent decision failed: run_id={run_id} reason={reason}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AgentFailure, ModerationTicket, QueuePriority};

    #[test]
    fn live_safety_failure_preempts_player_asset_review() {
        let asset = AgentFailure::AssetPolicy {
            asset_id: "asset-42".into(),
            player_id: "player-7".into(),
            reason: "text review inconclusive".into(),
        };
        let live = AgentFailure::LiveEventSafety {
            event_id: "event-9".into(),
            player_id: "player-8".into(),
            reason: "voice review inconclusive".into(),
        };

        assert_eq!(
            asset.moderation_ticket(),
            ModerationTicket { subject_id: "asset-42".into(), priority: QueuePriority::Normal }
        );
        assert_eq!(live.moderation_ticket().priority, QueuePriority::Critical);
    }
}

