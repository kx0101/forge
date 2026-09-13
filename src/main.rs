use std::io::{self, Write};

fn main() -> Result<(), std::io::Error> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let input = &mut String::new();

    let mut conversation: Vec<String> = Vec::new();

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
                stdout.write_all(line.as_bytes())?;
                stdout.write_all(b"\n")?;
            }

            continue;
        }

        if input.trim().is_empty() {
            continue;
        }

        stdout.write_all(input.as_bytes())?;

        let mut user_input = "user: ".to_owned();
        user_input.push_str(input.trim());

        conversation.push(user_input);

        let mut forge_answer = "forge: ".to_owned();
        forge_answer.push_str(input.trim());
        conversation.push(forge_answer);
    }
}
