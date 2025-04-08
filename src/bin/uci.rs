use itertools::Itertools;

fn main() -> std::io::Result<()> {
    let stdin = std::io::stdin();
    let mut input = String::new();
    loop {
        input.clear();
        stdin.read_line(&mut input)?;
        let mut words = input.split_whitespace();
        let command = words.next().unwrap_or("");

        match command {
            "uci" => {
                println!("id name ChessAI");
                println!("id author Leo Minton");
                println!("uciok");
            }
            "isready" => {
                println!("readyok");
            }
            "quit" => {
                break;
            }
            _ => {
                println!("Unknown command: {}", command);
            }
        }
    }
    Ok(())
}
