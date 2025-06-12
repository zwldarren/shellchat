use clap::Parser;
use config::{Config, Provider, ProviderConfig};
use console::style;
use futures::StreamExt;
use is_terminal::IsTerminal;
use std::io::{self, Read, Write};

mod cli;
mod commands;
mod config;
mod display;
mod executor;
mod providers;
mod system;
mod utils;

use crate::cli::Args;
use crate::commands::{ChatState, create_command_registry};
use crate::display::UserChoice;
use crate::executor::execute_command;
use crate::providers::{
    LLMProvider, Message, Role, openai::OpenAIProvider, openrouter::OpenRouterProvider,
    process_response,
};
use crate::system::SystemInfo;

const SYSTEM_PROMPT_FOR_SHELL: &str = "Convert the natural language query to a single command that \
will work on the current system. Only output the bare command without any explanation or markdown \
formatting. Include any necessary flags to make the command compatible with the current shell and OS. \
The current shell is {shell} and the OS is {os_info}.";
const SYSTEM_PROMPT_FOR_CHAT: &str =
    "You are a helpful assistant. Answer the following question in a concise manner: ";
const SYSTEM_PROMPT_FOR_DESCRIBE: &str = "Explain the shell command that was just provided in a concise \
and easy-to-understand way. Describe what the command does, what its main flags/options mean, and \
provide a simple example if applicable.";

fn merge_config_with_args(
    config: &Config,
    args: &Args,
) -> (Provider, String, String, Option<String>) {
    let provider = args
        .provider
        .as_ref()
        .and_then(|p| Provider::from_str(p))
        .or(config.active_provider)
        .unwrap_or_default();

    let default_config: ProviderConfig = ProviderConfig {
        base_url: None,
        model: None,
        api_key: None,
    };
    let provider_config = config.providers.get(&provider).unwrap_or(&default_config);

    let base_url = args
        .base_url
        .clone()
        .or_else(|| provider_config.base_url.clone())
        .unwrap_or_else(|| provider.default_base_url().to_string());

    let model = args
        .model
        .clone()
        .or_else(|| provider_config.model.clone())
        .unwrap_or_else(|| "gpt-4.1-mini".to_string());

    (provider, base_url, model, provider_config.api_key.clone())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = Config::load();
    let (provider_enum, base_url, model, api_key) = merge_config_with_args(&config, &args);
    let system_info = SystemInfo::new();

    let provider: Box<dyn LLMProvider> = match provider_enum {
        Provider::OpenAI => {
            if base_url == provider_enum.default_base_url() {
                Box::new(OpenAIProvider::new(api_key))
            } else {
                Box::new(OpenAIProvider::with_endpoint(base_url.clone(), api_key))
            }
        }
        Provider::OpenRouter => {
            if base_url == provider_enum.default_base_url() {
                Box::new(OpenRouterProvider::new(api_key))
            } else {
                Box::new(OpenRouterProvider::with_endpoint(base_url.clone(), api_key))
            }
        }
    };

    let context = if !std::io::stdin().is_terminal() {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        Some(buffer)
    } else {
        None
    };

    if args.shell {
        handle_shell_mode(&args, &config, provider, &model, &system_info, context).await?;
    } else if args.chat {
        handle_continuous_chat_mode(provider, &model).await?;
    } else {
        handle_chat_mode(&args, provider, &model, context).await?;
    }

    Ok(())
}

async fn handle_shell_mode(
    args: &Args,
    config: &Config,
    provider: Box<dyn LLMProvider>,
    model: &str,
    system_info: &SystemInfo,
    context: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create enhanced system prompt
    let prompt = SYSTEM_PROMPT_FOR_SHELL
        .replace("{shell}", &system_info.shell_path)
        .replace("{os_info}", &system_info.os_info);

    let final_query = match (args.query.as_deref(), context) {
        (Some(arg_q), Some(stdin_ctx)) => format!("<pipe>{}</pipe>\n\n{}", stdin_ctx, arg_q),
        (None, Some(stdin_ctx)) => format!("<pipe>{}</pipe>", stdin_ctx),
        (Some(arg_q), None) => arg_q.to_string(),
        (None, None) => {
            return Err("Query argument missing for shell mode".into());
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

    let raw_response = provider.get_response(&messages, model).await?;
    let command = process_response(&raw_response);

    display::display_command(&command);

    // State machine for command confirmation
    enum State {
        Initial,
        AfterFirstDescribe(Vec<Message>),
        AfterSecondDescribe(Vec<Message>),
    }

    let mut state = State::Initial;
    let mut execute = args.yes || config.auto_confirm;

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

                let describe_response = provider.get_response(&current_messages, model).await?;
                display::display_response(&describe_response);

                current_messages.push(Message {
                    role: Role::Assistant,
                    content: describe_response,
                });

                state = match state {
                    State::Initial => State::AfterFirstDescribe(current_messages),
                    State::AfterFirstDescribe(_) => State::AfterSecondDescribe(current_messages),
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

async fn handle_continuous_chat_mode(
    provider: Box<dyn LLMProvider>,
    model: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ChatState::new(provider, model);
    let command_registry = create_command_registry();

    println!("Entering chat mode. Type '/help' for available commands.");

    loop {
        print!("{}", style("> ").bold().cyan());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        // Check if input is a command
        if input.starts_with('/') {
            let parts: Vec<&str> = input[1..].split_whitespace().collect();
            if !parts.is_empty() {
                let command = parts[0];
                let args = if parts.len() > 1 { &parts[1..] } else { &[] };

                match command_registry.execute(command, args, &mut state) {
                    Ok(Some(output)) => {
                        println!("{}", output);
                    }
                    Ok(None) => {}
                    Err(e) => {
                        eprintln!("Error executing command: {}", e);
                    }
                }

                if !state.should_continue {
                    break;
                }
            }
            continue;
        }

        // Normal message processing
        state.messages.push(Message {
            role: Role::User,
            content: input.to_string(),
        });

        let mut stream = state
            .provider
            .get_response_stream(&state.messages, &state.model)
            .await?;

        io::stdout().flush()?;

        let mut full_response = String::new();
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    if !chunk.is_empty() {
                        let term = console::Term::stdout();
                        term.clear_last_lines(0).ok(); // Ensure clean rendering
                        display::display_markdown(&chunk);
                    }
                    io::stdout().flush()?;
                    full_response.push_str(&chunk);
                }
                Err(e) => {
                    eprintln!("Stream error: {}", e);
                    break;
                }
            }
        }

        if !full_response.ends_with('\n') {
            println!();
        }

        state.messages.push(Message {
            role: Role::Assistant,
            content: full_response,
        });
    }

    Ok(())
}

async fn handle_chat_mode(
    args: &Args,
    provider: Box<dyn LLMProvider>,
    model: &str,
    context: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let final_query = match (args.query.as_deref(), context) {
        (Some(arg_q), Some(stdin_ctx)) => format!("<pipe>{}</pipe>\n\n{}", stdin_ctx, arg_q),
        (None, Some(stdin_ctx)) => format!("<pipe>{}</pipe>", stdin_ctx),
        (Some(arg_q), None) => arg_q.to_string(),
        (None, None) => {
            return Err("No query provided".into());
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
    let response = provider.get_response(&messages, model).await?;

    // Display AI response with markdown formatting if needed
    if response.contains("```") || response.contains('*') || response.contains('`') || response.contains('#') {
        display::display_markdown(&response);
    } else {
        display::display_response(&response);
    }

    Ok(())
}
