use std::io::{self, Write};

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

        if input.trim_end() == "exit" {
            return Ok(());
        }

        if input.trim_end() == "/history" {
            for line in &conversation {
                let role = &line.role;
                let content = &line.content;

                let line = format!("{role}{content}");
                stdout.write_all(line.as_bytes())?;
                stdout.write_all(b"\n")?;
            }

            continue;
        }

        if input.trim_end() == "/clear" {
            conversation.clear();

            stdout.write_all(b"conversation cleared")?;
            stdout.write_all(b"\n")?;

            continue;
        }

        if input.trim().is_empty() {
            continue;
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
