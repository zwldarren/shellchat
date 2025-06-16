use super::Tool;
use crate::core::error::SchatError;
use async_trait::async_trait;
use rmcp::{
    RoleClient,
    model::{CallToolRequestParam, Tool as McpTool},
    service::RunningService,
};
use serde_json::Value;
use std::sync::Arc;

pub struct RemoteTool {
    name: String,
    description: String,
    parameters: Value,
    client: Arc<RunningService<RoleClient, ()>>,
}

impl RemoteTool {
    pub fn new(client: Arc<RunningService<RoleClient, ()>>, tool: McpTool) -> Self {
        Self {
            name: tool.name.to_string(),
            description: tool.description.unwrap_or_default().to_string(),
            parameters: serde_json::to_value(&tool.input_schema).unwrap_or(serde_json::json!({})),
            client,
        }
    }
}

#[async_trait]
impl Tool for RemoteTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters_schema(&self) -> Value {
        self.parameters.clone()
    }

    async fn call(&self, args: Value) -> Result<Value, SchatError> {
        let arguments = match args {
            Value::Object(map) => Some(map),
            _ => None,
        };

        let result = self
            .client
            .call_tool(CallToolRequestParam {
                name: self.name.clone().into(),
                arguments,
            })
            .await
            .map_err(|e| SchatError::ToolExecution(format!("MCP tool call failed: {}", e)))?;

        Ok(serde_json::json!({
            "is_error": result.is_error,
            "content": result.content
        }))
    }
}

pub async fn get_mcp_tools(
    client: Arc<RunningService<RoleClient, ()>>,
) -> Result<Vec<RemoteTool>, SchatError> {
    let tools = client
        .list_all_tools()
        .await
        .map_err(|e| SchatError::McpConnection(format!("Failed to list tools: {}", e)))?;

    Ok(tools
        .into_iter()
        .map(|tool| RemoteTool::new(client.clone(), tool))
        .collect())
}
