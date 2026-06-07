use argdown_mcp::server::ArgdownServer;
use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;

#[tokio::test]
async fn lists_and_calls_the_three_tools() {
    let (client_io, server_io) = tokio::io::duplex(8192);

    // Server side.
    tokio::spawn(async move {
        let server = ArgdownServer.serve(server_io).await.expect("server serves");
        let _ = server.waiting().await;
    });

    // Client side: `()` implements `ClientHandler` (no-capability client).
    let client = ().serve(client_io).await.expect("client connects");

    // list_all_tools → exactly our three.
    let tools = client.list_all_tools().await.expect("list tools");
    let mut names: Vec<String> = tools.iter().map(|t| t.name.to_string()).collect();
    names.sort();
    assert_eq!(names, vec!["dung_extensions", "export_model", "parse"]);

    // parse → ok summary.
    let parsed = client
        .call_tool(
            CallToolRequestParams::new("parse").with_arguments(
                serde_json::json!({ "source": "<A>: a" })
                    .as_object()
                    .cloned()
                    .unwrap(),
            ),
        )
        .await
        .expect("parse call");
    assert_ne!(parsed.is_error, Some(true));

    // export_model → JSON text (not an error).
    let exported = client
        .call_tool(
            CallToolRequestParams::new("export_model").with_arguments(
                serde_json::json!({ "source": "<A>: a\n\n(1) P\n----\n(2) C" })
                    .as_object()
                    .cloned()
                    .unwrap(),
            ),
        )
        .await
        .expect("export call");
    assert_ne!(exported.is_error, Some(true));

    // dung_extensions → IN/OUT/UNDEC partition (not an error).
    let dunged = client
        .call_tool(
            CallToolRequestParams::new("dung_extensions").with_arguments(
                serde_json::json!({ "source": "<A>: a\n\n<B>: b\n  -> <A>" })
                    .as_object()
                    .cloned()
                    .unwrap(),
            ),
        )
        .await
        .expect("dung_extensions call");
    assert_ne!(dunged.is_error, Some(true));

    client.cancel().await.ok();
}
