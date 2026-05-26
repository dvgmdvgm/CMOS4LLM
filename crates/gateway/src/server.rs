use std::path::PathBuf;
use std::sync::Arc;

use rust_mcp_sdk::error::SdkResult;
use rust_mcp_sdk::mcp_server::{server_runtime, McpServerOptions};
use rust_mcp_sdk::schema::*;
use rust_mcp_sdk::{McpServer, StdioTransport, ToMcpServerHandler, TransportOptions};

use cmos_memory::l1::{WorkingMemory, WorkingMemoryConfig};

use crate::handler::{CmosHandler, CmosState};

pub async fn start_mcp_server(data_root: PathBuf) -> SdkResult<()> {
    let state = Arc::new(CmosState {
        working_memory: WorkingMemory::new(WorkingMemoryConfig::default()),
        data_root,
    });

    let server_info = InitializeResult {
        server_info: Implementation {
            name: "cmos".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            title: Some("CMOS Memory Server".into()),
            description: Some(
                "Contextual Memory Orchestration System — persistent memory for LLM sessions"
                    .into(),
            ),
            icons: vec![],
            website_url: Some("https://github.com/dvgmdvgm/CMOS4LLM".into()),
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        protocol_version: ProtocolVersion::V2025_03_26.into(),
        instructions: Some(
            "CMOS provides persistent memory across LLM sessions. Use cmos_assemble_context \
             to get relevant context for your current task. Use cmos_write_memory to store \
             important decisions and facts. Use cmos_query_memory to look up specific knowledge."
                .into(),
        ),
        meta: None,
    };

    let transport = StdioTransport::new(TransportOptions::default())?;
    let handler = CmosHandler { state };

    let options = McpServerOptions {
        server_details: server_info,
        transport,
        handler: handler.to_mcp_server_handler(),
        task_store: None,
        client_task_store: None,
        message_observer: None,
    };

    let server = server_runtime::create_server(options);
    server.start().await
}
