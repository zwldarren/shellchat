use crate::cli::parser::Args;
use crate::commands::{ChatState, dispatcher::CommandDispatcher};
use crate::config::Config;
use crate::core::error::SchatError;
use crate::core::executor::execute_command;
use crate::display::{self, UserChoice};
use crate::input;
use crate::mcp::{Tool, ToolSet, tool::get_mcp_tools};
use crate::providers::{LLMProvider, Message, Role};
use crate::system::SystemInfo;
use console;
use futures::StreamExt;
use is_terminal::IsTerminal;
use regex;
use std::io::{self, Read, Write};
use std::sync::Arc;

pub struct Application {
    pub args: Args,
    pub config: Config,
    pub provider: Box<dyn LLMProvider>,
    pub command_dispatcher: CommandDispatcher,
}

impl Application {
    pub fn new(
        args: Args,
        config: Config,
        provider: Box<dyn LLMProvider>,
        command_dispatcher: CommandDispatcher,
    ) -> Result<Self, SchatError> {
        Ok(Self {
            args,
            config,
            provider,
            command_dispatcher,
        })
    }

    pub async fn run(&mut self) -> Result<(), SchatError> {
        let system_info = SystemInfo::new();

        let context = if !std::io::stdin().is_terminal() {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .map_err(|e| SchatError::Input(format!("Failed to read from stdin: {}", e)))?;
            Some(buffer)
        } else {
            None
        };

        if self.args.shell {
            self.handle_shell_mode(&system_info, context).await?;
        } else if self.args.chat {
            self.handle_continuous_chat_mode().await?;
        } else {
            self.handle_chat_mode(context).await?;
        }

        Ok(())
    }

    async fn handle_shell_mode(
        &self,
        system_info: &SystemInfo,
        context: Option<String>,
    ) -> Result<(), SchatError> {
        let prompt = SYSTEM_PROMPT_FOR_SHELL
            .replace("{shell}", &system_info.shell_path)
            .replace("{os_info}", &system_info.os_info);

        let final_query = match (self.args.query.as_deref(), context) {
            (Some(arg_q), Some(stdin_ctx)) => format!("<pipe>{}</pipe>\n\n{}", stdin_ctx, arg_q),
            (None, Some(stdin_ctx)) => format!("<pipe>{}</pipe>", stdin_ctx),
            (Some(arg_q), None) => arg_q.to_string(),
            (None, None) => {
                return Err(SchatError::Input(
                    "Query argument missing for shell mode".to_string(),
                ));
            }
        };
        let messages = vec![
            Message {
                role: Role::System,
                content: prompt,
            },
            Message {
                role: Role::User,
                content: final_query.clone(),
            },
        ];

        let raw_response = self.provider.get_response(&messages).await?;
        let command = crate::providers::process_response(&raw_response);

        display::display_command(&command);

        enum State {
            Initial,
            AfterFirstDescribe(Vec<Message>),
            AfterSecondDescribe(Vec<Message>),
        }

        let mut state = State::Initial;
        let mut execute = self.args.yes || self.config.auto_confirm;

        while !execute {
            let choice = display::prompt_execution_confirmation();

            match choice {
                UserChoice::Execute => {
                    execute = true;
                }
                UserChoice::Describe => {
                    let mut current_messages = match &state {
                        State::Initial => messages.clone(),
                        State::AfterFirstDescribe(msgs) => msgs.clone(),
                        State::AfterSecondDescribe(msgs) => msgs.clone(),
                    };

                    current_messages.push(Message {
                        role: Role::Assistant,
                        content: command.clone(),
                    });
                    current_messages.push(Message {
                        role: Role::User,
                        content: SYSTEM_PROMPT_FOR_DESCRIBE.to_string(),
                    });

                    let describe_response = self.provider.get_response(&current_messages).await?;
                    display::display_response(&describe_response);

                    current_messages.push(Message {
                        role: Role::Assistant,
                        content: describe_response.clone(),
                    });

                    state = match state {
                        State::Initial => State::AfterFirstDescribe(current_messages),
                        State::AfterFirstDescribe(_) => {
                            State::AfterSecondDescribe(current_messages)
                        }
                        State::AfterSecondDescribe(_) => {
                            break;
                        }
                    };
                }
                UserChoice::Abort => {
                    return Ok(());
                }
            }
        }

        if execute {
            execute_command(&command, system_info)?;
        }

        Ok(())
    }

    async fn generate_ai_response(&self, state: &ChatState) -> Result<String, SchatError> {
        let mut stream = state.provider.get_response_stream(&state.messages).await?;
        io::stdout().flush()?;

        let mut full_response = String::new();
        let mut display_buffer = String::new();
        let display_mode = display::get_display_mode();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    if !chunk.is_empty() {
                        full_response.push_str(&chunk);

                        // In hidden mode, check if this looks like a tool call before displaying
                        if matches!(display_mode, display::DisplayMode::Hidden) {
                            display_buffer.push_str(&chunk);

                            // Check if we have a complete tool call pattern
                            let tool_call_pattern =
                                r#""tool"\s*:\s*"([^"]+)"|```json\s*\{\s*"tool"\s*:\s*"([^"]+)"#;
                            let re = regex::Regex::new(tool_call_pattern).unwrap();

                            if re.is_match(&display_buffer) {
                                // This is a tool call, don't display it in hidden mode
                                continue;
                            }

                            // If buffer gets too long without matching, it's probably not a tool call
                            if display_buffer.len() > 500 {
                                let term = console::Term::stdout();
                                term.clear_last_lines(0).ok();
                                print!("{}", &display_buffer);
                                display_buffer.clear();
                            }
                        } else {
                            // Normal display mode - show everything
                            let term = console::Term::stdout();
                            term.clear_last_lines(0).ok();
                            print!("{}", &chunk);
                        }
                    }
                    io::stdout().flush()?;
                }
                Err(e) => {
                    eprintln!("Stream error: {}", e);
                    break;
                }
            }
        }

        // Handle any remaining buffer content in hidden mode
        if matches!(display_mode, display::DisplayMode::Hidden) && !display_buffer.is_empty() {
            let tool_call_pattern =
                r#""tool"\s*:\s*"([^"]+)"|```json\s*\{\s*"tool"\s*:\s*"([^"]+)"#;
            let re = regex::Regex::new(tool_call_pattern).unwrap();

            if !re.is_match(&display_buffer) {
                // Not a tool call, display it
                print!("{}", &display_buffer);
                io::stdout().flush()?;
            }
        }

        if !full_response.ends_with('\n') && !matches!(display_mode, display::DisplayMode::Hidden) {
            println!();
        }

        Ok(full_response)
    }

    async fn handle_tool_calls(
        &self,
        message: &str,
        tool_set: &ToolSet,
    ) -> Result<Vec<Message>, SchatError> {
        let mut tool_messages = Vec::new();

        // Improved tool call detection that handles standard formats
        let tool_call_pattern = r#""tool"\s*:\s*"([^"]+)"|```json\s*\{\s*"tool"\s*:\s*"([^"]+)"#;
        let re = regex::Regex::new(tool_call_pattern).unwrap();

        if let Some(caps) = re.captures(message) {
            let tool_name = caps
                .get(1)
                .or_else(|| caps.get(2))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            if !tool_name.is_empty() {
                display::display_tool_call(&tool_name);

                // Extract JSON arguments
                // Try to parse the entire message as JSON
                let tool_call: serde_json::Value = match serde_json::from_str(message) {
                    Ok(value) => value,
                    Err(_) => {
                        // Look for a JSON code block
                        if let Some(start) = message.find("```json") {
                            let code_start = start + 7; // Skip ```json
                            if let Some(end) = message[code_start..].find("```") {
                                let json_str = &message[code_start..code_start + end];
                                serde_json::from_str(json_str)
                                    .unwrap_or_else(|_| serde_json::json!({}))
                            } else {
                                serde_json::json!({})
                            }
                        }
                        // Look for any code block
                        else if let Some(start) = message.find("```") {
                            let code_start = start + 3; // Skip ```
                            if let Some(end) = message[code_start..].find("```") {
                                let json_str = &message[code_start..code_start + end];
                                serde_json::from_str(json_str)
                                    .unwrap_or_else(|_| serde_json::json!({}))
                            } else {
                                serde_json::json!({})
                            }
                        }
                        // Fallback to extracting first JSON object
                        else {
                            let json_start = message.find('{').unwrap_or(0);
                            let json_end = message
                                .rfind('}')
                                .map(|pos| pos + 1)
                                .unwrap_or(message.len());
                            let json_str = &message[json_start..json_end];
                            serde_json::from_str(json_str).unwrap_or_else(|_| serde_json::json!({}))
                        }
                    }
                };

                let args = if let Some(args_obj) = tool_call.get("arguments") {
                    args_obj.clone()
                } else {
                    println!("No arguments found in tool call");
                    serde_json::json!({})
                };

                let args_str = serde_json::to_string_pretty(&args).unwrap_or_default();
                display::display_tool_arguments(&args_str);

                match tool_set.call_tool(&tool_name, args).await {
                    Ok(result) => {
                        let pretty_result = serde_json::to_string_pretty(&result)
                            .unwrap_or_else(|_| result.to_string());
                        display::display_tool_success(&pretty_result);
                        tool_messages.push(Message {
                            role: Role::User,
                            content: format!("Tool call result: {}", pretty_result),
                        });
                    }
                    Err(e) => {
                        display::display_tool_error(&format!("{}", e));
                        tool_messages.push(Message {
                            role: Role::User,
                            content: format!("Tool call failed: {}", e),
                        });
                    }
                }
            }
        }

        Ok(tool_messages)
    }

    async fn handle_continuous_chat_mode(&mut self) -> Result<(), SchatError> {
        // Display beautiful chat header
        display::display_chat_header();

        // Initialize MCP clients and tools
        print!("{}Initializing tools", console::style("🔧 ").cyan());
        io::stdout().flush()?;

        let mcp_clients = self.config.create_mcp_clients().await?;
        let mut tool_set = ToolSet::new();

        for (name, client) in mcp_clients {
            display::display_mcp_connection(&name);
            match get_mcp_tools(Arc::new(client)).await {
                Ok(tools) => {
                    for tool in tools {
                        display::display_tool_added(tool.name());
                        tool_set.add_tool(Arc::new(tool));
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get tools from {}: {}", name, e);
                }
            }
        }

        // Display initialization complete
        display::display_initialization_complete(tool_set.tools().len());

        let mut state = ChatState::new(
            self.provider.clone_provider(),
            &self.args.model.clone().unwrap_or_default(),
        );

        // Add tool instructions if tools are available
        if !tool_set.tools().is_empty() {
            let mut tool_instructions = String::from("You have access to the following tools:\n");
            for tool in tool_set.tools() {
                tool_instructions.push_str(&format!("- {}: {}\n", tool.name(), tool.description()));
            }
            tool_instructions.push_str(
                "\nWhen you need to use a tool, output a JSON object with these fields:\n",
            );
            tool_instructions.push_str("{\n  \"tool\": \"tool_name\",\n  \"arguments\": {\n    \"param1\": value1,\n    \"param2\": value2\n  }\n}\n");

            state.messages.insert(
                0,
                Message {
                    role: Role::System,
                    content: tool_instructions,
                },
            );
        }

        let mut editor = input::create_editor(self.command_dispatcher.clone())?;

        loop {
            let input_result = input::read_input(&mut editor)?;

            let input = match input_result {
                Some(input) => input.trim().to_string(),
                None => break,
            };

            if input.is_empty() {
                continue;
            }

            if input.starts_with('/') {
                let parts: Vec<&str> = input[1..].split_whitespace().collect();
                if !parts.is_empty() {
                    let command = parts[0];
                    let args = if parts.len() > 1 { &parts[1..] } else { &[] };

                    match self.command_dispatcher.execute(command, args, &mut state) {
                        Ok(Some(output)) => {
                            println!("{}", output);
                        }
                        Ok(None) => {}
                        Err(e) => {
                            eprintln!(
                                "{}Error executing command: {}",
                                console::style("❌ ").red(),
                                e
                            );
                        }
                    }

                    if !state.should_continue {
                        break;
                    }
                }
                continue;
            }

            state.messages.push(Message {
                role: Role::User,
                content: input,
            });

            // Display AI response header
            display::display_ai_response_header();

            // Generate AI response to user input
            let mut full_response = self.generate_ai_response(&state).await?;
            state.messages.push(Message {
                role: Role::Assistant,
                content: full_response.clone(),
            });

            // Process tool calls and continue conversation automatically
            loop {
                let tool_messages = self.handle_tool_calls(&full_response, &tool_set).await?;
                if tool_messages.is_empty() {
                    break;
                }

                // Add tool results to conversation
                state.messages.extend(tool_messages);

                // Generate next AI response automatically
                full_response = self.generate_ai_response(&state).await?;
                state.messages.push(Message {
                    role: Role::Assistant,
                    content: full_response.clone(),
                });
            }

            println!(); // Add spacing after each conversation turn
        }

        input::save_history(&mut editor)?;

        Ok(())
    }

    async fn handle_chat_mode(&self, context: Option<String>) -> Result<(), SchatError> {
        let final_query = match (self.args.query.as_deref(), context) {
            (Some(arg_q), Some(stdin_ctx)) => format!("<pipe>{}</pipe>\n\n{}", stdin_ctx, arg_q),
            (None, Some(stdin_ctx)) => format!("<pipe>{}</pipe>", stdin_ctx),
            (Some(arg_q), None) => arg_q.to_string(),
            (None, None) => {
                return Err(SchatError::Input("No query provided".to_string()));
            }
        };

        let messages = vec![
            Message {
                role: Role::System,
                content: SYSTEM_PROMPT_FOR_CHAT.to_string(),
            },
            Message {
                role: Role::User,
                content: final_query,
            },
        ];
        let response = self.provider.get_response(&messages).await?;

        if response.contains("```")
            || response.contains('*')
            || response.contains('`')
            || response.contains('#')
        {
            display::display_markdown(&response);
        } else {
            display::display_response(&response);
        }

        Ok(())
    }
}

const SYSTEM_PROMPT_FOR_SHELL: &str = "Convert the natural language query to a single command that \
will work on the current system. Only output the bare command without any explanation or markdown \
formatting. Include any necessary flags to make the command compatible with the current shell and OS. \
The current shell is {shell} and the OS is {os_info}.";
const SYSTEM_PROMPT_FOR_CHAT: &str =
    "You are a helpful assistant. Answer the following question in a concise manner: ";
const SYSTEM_PROMPT_FOR_DESCRIBE: &str = "Explain the shell command that was just provided in a concise \
and easy-to-understand way. Describe what the command does, what its main flags/options mean, and \
provide a simple example if applicable.";
