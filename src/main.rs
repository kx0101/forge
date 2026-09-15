use std::io::{self, Stdout, Write};

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

fn get_history(conversation: &Vec<Message>, stdout: &mut Stdout) -> Result<(), std::io::Error> {
    for line in conversation {
        let role = &line.role;
        let content = &line.content;

        let line = format!("{role}{content}");
        stdout.write_all(line.as_bytes())?;
        stdout.write_all(b"\n")?;
    }

    Ok(())
}

fn clear(conversation: &mut Vec<Message>, stdout: &mut Stdout) -> Result<(), std::io::Error> {
    conversation.clear();

    stdout.write_all(b"conversation cleared")?;
    stdout.write_all(b"\n")?;

    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let input = &mut String::new();

    let mut conversation: Vec<Message> = Vec::new();

    loop {
        stdout.write_all(b"forge> ")?;
        stdout.flush()?;

        input.clear();
        let bytes_read = stdin.read_line(input)?;
        if bytes_read == 0 {
            return Ok(());
        }

        match get_command(input) {
            Some(Command::History) => {
                get_history(&conversation, &mut stdout)?;
                continue;
            }
            Some(Command::Clear) => {
                clear(&mut conversation, &mut stdout)?;
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

        stdout.write_all(input.as_bytes())?;

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

#[cfg(test)]
mod tests {
    use super::*;

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
