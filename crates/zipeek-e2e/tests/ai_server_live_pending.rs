use zipeek_e2e::HarnessResult;

#[tokio::test]
#[ignore = "Pending browser E2E: requires a controlled zipeek-ai-server endpoint and model fixture"]
async fn browser_translation_via_zipeek_ai_server() -> HarnessResult<()> {
    unimplemented!(
        "Configure Settings to target zipeek-ai-server, complete a Practice translation, and verify persisted results"
    )
}
