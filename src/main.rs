use std::io::Write;
use std::io::{self, BufRead};

#[derive(Debug)]
enum Role {
    User,
    Forge,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::User => write!(f, "user: "),
            Role::Forge => write!(f, "forge: "),
        }
    }
}

#[derive(Debug)]
struct Message {
    role: Role,
    content: String,
}

#[derive(Debug, PartialEq)]
enum Command {
    History,
    Clear,
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

fn get_history<W>(conversation: &Vec<Message>, writer: &mut W) -> Result<(), std::io::Error>
where
    W: Write,
{
    for line in conversation {
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
    conversation.clear();

    writer.write_all(b"conversation cleared")?;
    writer.write_all(b"\n")?;

    Ok(())
}

fn run<R, W>(reader: &mut R, writer: &mut W) -> Result<(), std::io::Error>
where
    R: BufRead,
    W: Write,
{
    let input = &mut String::new();
    let mut conversation: Vec<Message> = Vec::new();

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

        writer.write_all(input.as_bytes())?;

        let user_input = Message {
            role: Role::User,
            content: input.trim().to_string(),
        };
        conversation.push(user_input);

        let forge_answer = Message {
            role: Role::Forge,
            content: input.trim().to_string(),
        };
        conversation.push(forge_answer);
    }
}

fn main() -> Result<(), std::io::Error> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    let stdout = io::stdout();
    let mut writer = stdout.lock();

    run(&mut reader, &mut writer)
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn test_normal_conversation() {
        let input = "hello\nexit";
        let mut reader = BufReader::new(input.as_bytes());
        let expected_output = "forge> hello\nforge> ";

        let mut writer: Vec<u8> = Vec::new();

        let result = run(&mut reader, &mut writer);

        assert!(result.is_ok());
        assert_eq!(writer, expected_output.as_bytes())
    }

    #[test]
    fn test_non_empty_history_after_message() {
        let input = "hello\nhow are you?\n/history";
        let mut reader = BufReader::new(input.as_bytes());
        let expected_output = "forge> hello\nforge> how are you?\nforge> user: hello\nforge: hello\nuser: how are you?\nforge: how are you?\nforge> ";

        let mut writer: Vec<u8> = Vec::new();

        let result = run(&mut reader, &mut writer);

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

        let result = run(&mut reader, &mut writer);

        assert!(result.is_ok());
        assert_eq!(writer, expected_output.as_bytes());
    }

    #[test]
    fn test_blank_input() {
        let input = "  \n";
        let mut reader = BufReader::new(input.as_bytes());
        let expected_output = "forge> forge> ";

        let mut writer: Vec<u8> = Vec::new();

        let result = run(&mut reader, &mut writer);

        assert!(result.is_ok());
        assert_eq!(writer, expected_output.as_bytes());
    }

    #[test]
    fn test_eof() {
        let input = "hello\n";
        let mut reader = BufReader::new(input.as_bytes());
        let expected_output = "forge> hello\nforge> ";

        let mut writer: Vec<u8> = Vec::new();

        let result = run(&mut reader, &mut writer);

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
}
