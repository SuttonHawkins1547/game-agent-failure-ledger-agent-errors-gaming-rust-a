use game_agent_failure_ledger::{
    failure_tracker::FailureTracker,
    moderation_queue::AgentFailure,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = FailureTracker::from_env()?;
    let failure = AgentFailure::AssetPolicy {
        asset_id: "ugc-map-1842".into(),
        player_id: "player-73".into(),
        reason: "agent could not establish a moderation verdict".into(),
    };

    let ticket = tracker.capture(&failure).await?;
    println!(
        "captured {} for {:?} moderation",
        ticket.subject_id, ticket.priority
    );
    Ok(())
}

