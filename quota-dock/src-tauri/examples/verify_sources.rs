use quota_dock_lib::providers::live::{query_google_live, query_openai_live};
#[tokio::main]
async fn main() {
    let (google, openai) = tokio::join!(query_google_live(), query_openai_live());
    let mut snapshots = google.expect("Google live query failed");
    snapshots.extend(openai.expect("OpenAI live query failed"));
    assert!(
        !snapshots.is_empty(),
        "No connected accounts on this machine"
    );
    for s in &snapshots {
        assert!(
            matches!(
                s.status,
                quota_dock_lib::contracts::Availability::Ok
                    | quota_dock_lib::contracts::Availability::Partial
            ),
            "Account query did not return quota"
        );
        assert!(
            s.windows.iter().any(|w| w.remaining_percent.is_some())
                || s.models.iter().any(|m| m.remaining_percent.is_some())
        );
    }
    // Public DTOs only: no credentials or raw provider response.
    println!("{}", serde_json::to_string_pretty(&snapshots).unwrap());
}
