use zipeek_e2e::HarnessResult;

#[tokio::test]
#[ignore = "Pending browser acceptance: picture vocabulary playable completion"]
async fn image_vocabulary_complete_round() -> HarnessResult<()> {
    unimplemented!(
        "Use a deterministic approved-image fixture, score both selection modes, and verify completion"
    )
}

#[tokio::test]
#[ignore = "Pending browser acceptance: speaking recording, cancellation, and retry"]
async fn speaking_recording_and_retry() -> HarnessResult<()> {
    unimplemented!(
        "Use a deterministic microphone fixture, record, cancel, retry, and verify accessible feedback"
    )
}
