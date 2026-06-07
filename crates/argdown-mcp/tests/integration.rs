use argdown_mcp::server::ArgdownServer;
use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;

const B6B_CANONICAL: &str = "<A>: a\n\n<B>: b\n  -> <A>";

fn titles_from_partition(value: &serde_json::Value) -> (Vec<String>, Vec<String>, Vec<String>) {
    let titles = |key: &str| {
        value[key]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|a| a["title"].as_str().map(str::to_string))
            .collect::<Vec<_>>()
    };
    (titles("in"), titles("out"), titles("undec"))
}

#[tokio::test]
async fn grounded_cross_check_matches_b6b_probe_via_mcp() {
    let (client_io, server_io) = tokio::io::duplex(8192);

    tokio::spawn(async move {
        let server = ArgdownServer.serve(server_io).await.expect("server serves");
        let _ = server.waiting().await;
    });

    let client = ().serve(client_io).await.expect("client connects");

    let dunged = client
        .call_tool(
            CallToolRequestParams::new("dung_extensions").with_arguments(
                serde_json::json!({ "source": B6B_CANONICAL })
                    .as_object()
                    .cloned()
                    .unwrap(),
            ),
        )
        .await
        .expect("dung_extensions call");
    assert_ne!(dunged.is_error, Some(true));
    let dung_json = dunged
        .structured_content
        .expect("structured_content on dung_extensions");
    let (in_, out, undec) = titles_from_partition(&dung_json);
    assert_eq!(in_, vec!["B"]);
    assert_eq!(out, vec!["A"]);
    assert!(undec.is_empty());

    let grounded = client
        .call_tool(
            CallToolRequestParams::new("extensions").with_arguments(
                serde_json::json!({ "source": B6B_CANONICAL, "semantics": "grounded" })
                    .as_object()
                    .cloned()
                    .unwrap(),
            ),
        )
        .await
        .expect("extensions call");
    assert_ne!(grounded.is_error, Some(true));
    let ext = grounded
        .structured_content
        .expect("structured_content on extensions");
    let ext_in: Vec<String> = ext["extension_sets"][0]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|a| a["title"].as_str().map(str::to_string))
        .collect();
    assert_eq!(ext_in, vec!["B"]);

    client.cancel().await.ok();
}

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

    // list_all_tools → our six tools.
    let tools = client.list_all_tools().await.expect("list tools");
    let mut names: Vec<String> = tools.iter().map(|t| t.name.to_string()).collect();
    names.sort();
    assert_eq!(
        names,
        vec![
            "accepts",
            "dung_extensions",
            "export_model",
            "extensions",
            "inspect_af",
            "parse",
            "qbaf_evaluate",
        ]
    );

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

#[tokio::test]
async fn list_tools_includes_extensions_and_inspect_af() {
    let (client_io, server_io) = tokio::io::duplex(8192);

    tokio::spawn(async move {
        let server = ArgdownServer.serve(server_io).await.expect("server serves");
        let _ = server.waiting().await;
    });

    let client = ().serve(client_io).await.expect("client connects");
    let tools = client.list_all_tools().await.expect("list tools");
    let names: Vec<String> = tools.iter().map(|t| t.name.to_string()).collect();

    assert!(names.contains(&"extensions".to_string()));
    assert!(names.contains(&"inspect_af".to_string()));
    assert!(names.contains(&"accepts".to_string()));
    assert!(names.contains(&"dung_extensions".to_string()));
    assert!(names.contains(&"qbaf_evaluate".to_string()));

    client.cancel().await.ok();
}
