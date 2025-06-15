use crate::cli::parser::Args;
use crate::commands::{ChatState, dispatcher::CommandDispatcher};
use crate::config::Config;
use crate::core::error::SchatError;
use crate::core::executor::execute_command;
use crate::display::{self, UserChoice};
use crate::input;
use crate::providers::{LLMProvider, Message, Role};
use crate::system::SystemInfo;
use futures::StreamExt;
use is_terminal::IsTerminal;
use std::io::{self, Read, Write};

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

    async fn handle_continuous_chat_mode(&mut self) -> Result<(), SchatError> {
        let mut state = ChatState::new(
            self.provider.clone_provider(),
            &self.args.model.clone().unwrap_or_default(),
        );

        println!(
            "Entering chat mode. Type '/help' for available commands. Press Ctrl+D or type /quit to exit."
        );

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
                            eprintln!("Error executing command: {}", e);
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

            let mut stream = state.provider.get_response_stream(&state.messages).await?;

            io::stdout().flush()?;

            let mut full_response = String::new();
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        if !chunk.is_empty() {
                            let term = console::Term::stdout();
                            term.clear_last_lines(0).ok();
                            print!("{}", &chunk);
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
