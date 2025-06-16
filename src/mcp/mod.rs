pub mod tool;

use crate::core::error::SchatError;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    #[allow(dead_code)]
    fn parameters_schema(&self) -> Value;
    async fn call(&self, args: Value) -> Result<Value, SchatError>;
}

pub struct ToolSet {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolSet {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn add_tool(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    #[allow(dead_code)]
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    pub fn tools(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.values().cloned().collect()
    }

    pub async fn call_tool(&self, name: &str, args: Value) -> Result<Value, SchatError> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| SchatError::ToolNotFound(name.to_string()))?;
        tool.call(args).await
    }
}

impl Default for ToolSet {
    fn default() -> Self {
        Self::new()
    }
}
