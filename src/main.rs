use std::io::Write;
use std::io::{self, BufRead};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum Role {
    #[serde(rename = "user")]
    User,

    #[serde(rename = "assistant")]
    Forge,

    #[serde(rename = "system")]
    System,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::User => write!(f, "user: "),
            Role::Forge => write!(f, "forge: "),
            Role::System => write!(f, "system: "),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: Role,
    content: String,
}

#[derive(Debug, PartialEq)]
enum Command {
    History,
    Clear,
}

#[derive(Clone)]
struct EchoModel;

#[derive(Debug)]
enum AppError {
    Io(std::io::Error),
    Model(ModelError),
    Reqwest(reqwest::Error),
}

#[derive(Debug)]
struct ModelError {
    message: String,
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Error] {}", self.message)
    }
}

impl From<reqwest::Error> for ModelError {
    fn from(error: reqwest::Error) -> Self {
        ModelError {
            message: error.to_string(),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::Io(error)
    }
}

impl From<ModelError> for AppError {
    fn from(error: ModelError) -> Self {
        AppError::Model(error)
    }
}

trait Model {
    fn respond(&self, conversation: &[Message]) -> Result<Option<Message>, ModelError>;
}

impl Model for EchoModel {
    fn respond(&self, conversation: &[Message]) -> Result<Option<Message>, ModelError> {
        let message_to_echo = conversation.last();
        if message_to_echo.is_none() {
            return Ok(None);
        }

        let forge_answer = Message {
            role: Role::Forge,
            content: message_to_echo.unwrap().content.trim().to_string(),
        };

        Ok(Some(forge_answer))
    }
}

struct OllamaModel {
    endpoint: String,
    model_name: String,
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: String,
    messages: &'a [Message],
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    message: Message,
}

impl OllamaModel {
    fn new() -> Self {
        OllamaModel {
            endpoint: String::from("http://127.0.0.1:11434"),
            model_name: String::from("qwen3.5:9b"),
        }
    }
}

impl Model for OllamaModel {
    fn respond(&self, conversation: &[Message]) -> Result<Option<Message>, ModelError> {
        let request = OllamaRequest {
            model: self.model_name.to_string(),
            messages: conversation,
            stream: false,
        };

        let client = reqwest::blocking::Client::new();
        let response = client
            .post(format!("{}/api/chat", self.endpoint))
            .json(&request)
            .send()?
            .error_for_status()?
            .json::<OllamaResponse>()?;

        Ok(Some(response.message))
    }
}

fn get_command(input: &str) -> Option<Command> {
    if input.trim_end() == "/history" {
        return Some(Command::History);
    }

    if input.trim_end() == "/clear" {
        return Some(Command::Clear);
    }

    None
}

fn get_history<W>(conversation: &[Message], writer: &mut W) -> Result<(), std::io::Error>
where
    W: Write,
{
    for line in conversation {
        if line.role == Role::System {
            continue;
        }

        let role = &line.role;
        let content = &line.content;

        let line = format!("{role}{content}");
        writer.write_all(line.as_bytes())?;
        writer.write_all(b"\n")?;
    }

    Ok(())
}

fn clear<W>(conversation: &mut Vec<Message>, writer: &mut W) -> Result<(), std::io::Error>
where
    W: Write,
{
    conversation.retain(|message| message.role == Role::System);

    writer.write_all(b"conversation cleared")?;
    writer.write_all(b"\n")?;

    Ok(())
}

fn run<R, W, M>(reader: &mut R, writer: &mut W, model: &M) -> Result<(), AppError>
where
    R: BufRead,
    W: Write,
    M: Model,
{
    let input = &mut String::new();
    let mut conversation: Vec<Message> = Vec::new();

    let system_prompt = Message {
        role: Role::System,
        content: "You are Forge, an interactive coding assistant.".to_string(),
    };
    conversation.push(system_prompt);

    loop {
        writer.write_all(b"forge> ")?;
        writer.flush()?;

        input.clear();
        let bytes_read = reader.read_line(input)?;
        if bytes_read == 0 {
            return Ok(());
        }

        match get_command(input) {
            Some(Command::History) => {
                get_history(&conversation, writer)?;
                continue;
            }
            Some(Command::Clear) => {
                clear(&mut conversation, writer)?;
                continue;
            }
            None => {
                if input.trim_end().is_empty() {
                    continue;
                }
            }
        };

        if input.trim_end() == "exit" {
            return Ok(());
        }

        let user_input = Message {
            role: Role::User,
            content: input.trim().to_string(),
        };
        conversation.push(user_input);

        match model.respond(&conversation) {
            Ok(Some(answer)) => {
                writer.write_all(answer.content.as_bytes())?;
                writer.write_all(b"\n")?;

                conversation.push(answer);
            }
            Ok(None) => {}
            Err(err) => {
                writeln!(writer, "forge> {err}")?;
            }
        }
    }
}

fn main() -> Result<(), AppError> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    let stdout = io::stdout();
    let mut writer = stdout.lock();

    let model = OllamaModel::new();

    run(&mut reader, &mut writer, &model)
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::io::BufReader;

    use super::*;
    struct PanicModel;

    impl Model for PanicModel {
        fn respond(&self, _conversation: &[Message]) -> Result<Option<Message>, ModelError> {
            panic!("model should not have been called");
        }
    }

    struct FakeModel;

    impl Model for FakeModel {
        fn respond(&self, _conversation: &[Message]) -> Result<Option<Message>, ModelError> {
            let generated_message = Message {
                role: Role::Forge,
                content: "generated response".to_string(),
            };

            Ok(Some(generated_message))
        }
    }

    #[derive(Default)]
    struct RecordingModel {
        conversations: RefCell<Vec<Vec<Message>>>,
    }

    impl Model for RecordingModel {
        fn respond(&self, conversation: &[Message]) -> Result<Option<Message>, ModelError> {
            self.conversations.borrow_mut().push(conversation.to_vec());

            Ok(Some(Message {
                role: Role::Forge,
                content: "recorded".to_string(),
            }))
        }
    }

    #[derive(Default)]
    struct FailOnceModel {
        calls: Cell<usize>,
        conversations: RefCell<Vec<Vec<Message>>>,
    }

    impl Model for FailOnceModel {
        fn respond(&self, conversation: &[Message]) -> Result<Option<Message>, ModelError> {
            self.conversations.borrow_mut().push(conversation.to_vec());

            let call = self.calls.get();
            self.calls.set(call + 1);

            if call == 0 {
                return Err(ModelError {
                    message: "temporary failure".to_string(),
                });
            }

            Ok(Some(Message {
                role: Role::Forge,
                content: "recovered".to_string(),
            }))
        }
    }

    #[test]
    fn test_normal_conversation() {
        let input = "hello\nexit";
        let mut reader = BufReader::new(input.as_bytes());
        let expected_output = "forge> hello\nforge> ";

        let mut writer: Vec<u8> = Vec::new();

        let model = EchoModel;

        let result = run(&mut reader, &mut writer, &model);

        assert!(result.is_ok());
        assert_eq!(writer, expected_output.as_bytes())
    }

    #[test]
    fn test_non_empty_history_after_message() {
        let input = "hello\nhow are you?\n/history";
        let mut reader = BufReader::new(input.as_bytes());
        let expected_output = "forge> hello\nforge> how are you?\nforge> user: hello\nforge: hello\nuser: how are you?\nforge: how are you?\nforge> ";

        let mut writer: Vec<u8> = Vec::new();

        let model = EchoModel;

        let result = run(&mut reader, &mut writer, &model);

        assert!(result.is_ok());
        assert_eq!(writer, expected_output.as_bytes())
    }

    #[test]
    fn test_history_gets_cleared() {
        let input = "hello\nhow are you?\n/clear\n/history";
        let mut reader = BufReader::new(input.as_bytes());
        let expected_output =
            "forge> hello\nforge> how are you?\nforge> conversation cleared\nforge> forge> ";

        let mut writer: Vec<u8> = Vec::new();

        let model = EchoModel;

        let result = run(&mut reader, &mut writer, &model);

        assert!(result.is_ok());
        assert_eq!(writer, expected_output.as_bytes());
    }

    #[test]
    fn test_blank_input() {
        let input = "  \n";
        let mut reader = BufReader::new(input.as_bytes());
        let expected_output = "forge> forge> ";

        let mut writer: Vec<u8> = Vec::new();

        let model = EchoModel;

        let result = run(&mut reader, &mut writer, &model);

        assert!(result.is_ok());
        assert_eq!(writer, expected_output.as_bytes());
    }

    #[test]
    fn test_eof() {
        let input = "hello\n";
        let mut reader = BufReader::new(input.as_bytes());
        let expected_output = "forge> hello\nforge> ";

        let mut writer: Vec<u8> = Vec::new();

        let model = EchoModel;

        let result = run(&mut reader, &mut writer, &model);

        assert!(result.is_ok());
        assert_eq!(writer, expected_output.as_bytes());
    }

    #[test]
    fn test_get_command_history() {
        let input = "/history";
        let command = get_command(input).unwrap();
        assert_eq!(command, Command::History);
    }

    #[test]
    fn test_get_command_clear() {
        let input = "/clear";
        let command = get_command(input).unwrap();
        assert_eq!(command, Command::Clear);
    }

    #[test]
    fn test_get_command_returns_none() {
        let input = "whats up there";
        let command = get_command(input);
        assert_eq!(command, None);
    }

    #[test]
    fn test_fake_model_different_output() {
        let input = "hello";
        let mut reader = BufReader::new(input.as_bytes());
        let expected_output = "forge> generated response\nforge> ";

        let mut writer: Vec<u8> = Vec::new();

        let model = FakeModel;

        let result = run(&mut reader, &mut writer, &model);

        assert!(result.is_ok());
        assert_eq!(writer, expected_output.as_bytes())
    }

    #[test]
    fn commands_and_empty_input_do_not_call_model() {
        let input = "/history\n/clear\n   \nexit\n";
        let expected_output = "forge> forge> conversation cleared\nforge> forge> ";
        let mut reader = BufReader::new(input.as_bytes());
        let mut writer = Vec::new();
        let model = PanicModel;

        let result = run(&mut reader, &mut writer, &model);

        assert!(result.is_ok());
        assert_eq!(writer, expected_output.as_bytes())
    }

    #[test]
    fn model_receives_system_message_first() {
        let input = "hello\nexit\n";
        let mut reader = BufReader::new(input.as_bytes());
        let mut writer = Vec::new();
        let model = RecordingModel::default();

        run(&mut reader, &mut writer, &model).unwrap();

        let conversations = model.conversations.borrow();
        let first_call = &conversations[0];
        assert!(matches!(first_call[0].role, Role::System));
        assert_eq!(
            first_call[0].content,
            "You are Forge, an interactive coding assistant."
        );
        assert!(matches!(first_call[1].role, Role::User));
        assert_eq!(first_call[1].content, "hello");
    }

    #[test]
    fn clear_preserves_system_message_for_next_model_call() {
        let input = "before\n/clear\nafter\nexit\n";
        let mut reader = BufReader::new(input.as_bytes());
        let mut writer = Vec::new();
        let model = RecordingModel::default();

        run(&mut reader, &mut writer, &model).unwrap();

        let conversations = model.conversations.borrow();
        let second_call = &conversations[1];
        assert_eq!(second_call.len(), 2);
        assert!(matches!(second_call[0].role, Role::System));
        assert_eq!(
            second_call[0].content,
            "You are Forge, an interactive coding assistant."
        );
        assert!(matches!(second_call[1].role, Role::User));
        assert_eq!(second_call[1].content, "after");
    }

    #[test]
    fn history_does_not_display_system_message() {
        let input = "hello\n/history\nexit\n";
        let mut reader = BufReader::new(input.as_bytes());
        let mut writer = Vec::new();
        let model = EchoModel;

        run(&mut reader, &mut writer, &model).unwrap();

        let output = String::from_utf8(writer).unwrap();
        assert!(!output.contains("system:"));
        assert!(!output.contains("You are Forge, an interactive coding assistant."));
        assert!(output.contains("user: hello\nforge: hello\n"));
    }

    #[test]
    fn model_failure_keeps_session_open_and_preserves_user_message() {
        let input = "first\nsecond\nexit\n";
        let mut reader = BufReader::new(input.as_bytes());
        let mut writer = Vec::new();
        let model = FailOnceModel::default();

        let result = run(&mut reader, &mut writer, &model);

        assert!(result.is_ok());
        assert_eq!(model.calls.get(), 2);
        assert_eq!(
            writer,
            b"forge> forge> [Error] temporary failure\nforge> recovered\nforge> "
        );

        let conversations = model.conversations.borrow();
        let second_call = &conversations[1];
        assert_eq!(second_call.len(), 3);
        assert!(matches!(second_call[0].role, Role::System));
        assert!(matches!(second_call[1].role, Role::User));
        assert_eq!(second_call[1].content, "first");
        assert!(matches!(second_call[2].role, Role::User));
        assert_eq!(second_call[2].content, "second");
    }
}
