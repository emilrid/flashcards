use clap::{Parser, Subcommand};

use serde::{Deserialize, Serialize};

use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct Flashcard {
    front: String,
    back: String,
    correct: u32,
    incorrect: u32,
}

#[derive(Serialize, Deserialize)]
struct DeckState {
    current_index: usize,
    cards: Vec<Flashcard>,
}

impl DeckState {
    fn new() -> Self {
        Self {
            current_index: 0,
            cards: Vec::new(),
        }
    }

    fn add_card(&mut self, flashcard: Flashcard) {
        self.cards.push(flashcard);
    }

    fn remove_card(&mut self, index: usize) -> Result<Flashcard, io::Error> {
        if index < self.cards.len() {
            Ok(self.cards.remove(index))
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Index out of bounds",
            ))
        }
    }
}

impl Flashcard {
    fn from(front: String, back: String) -> Self {
        Self {
            front,
            back,
            correct: 0,
            incorrect: 0,
        }
    }
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    file: PathBuf,
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    Add { front: String, back: String },
    Remove { index: usize },
    List,
}

fn save_state(path: &Path, state: &DeckState) {
    let toml = toml::to_string_pretty(state).unwrap();
    fs::write(path, toml).unwrap();
}

fn load_state(path: &Path) -> DeckState {
    if let Ok(text) = fs::read_to_string(path) {
        if let Ok(state) = toml::from_str::<DeckState>(&text) {
            return state;
        }
    }

    DeckState::new()
}

fn main() {
    let args = Args::parse();
    let mut state = load_state(&args.file);

    match args.action {
        Action::Add { front, back } => {
            println!("Adding flashcard: {}, {}", front, back);
            state.add_card(Flashcard::from(front, back));
        }

        Action::Remove { index } => match state.remove_card(index - 1) {
            Ok(flashcard) => {
                println!(
                    "Removed card nbr {}: {}, {}",
                    index, flashcard.front, flashcard.back
                );
            }
            Err(error) => {
                println!("{}", error);
            }
        },

        Action::List => {
            println!("Cards in {}", &args.file.to_str().unwrap());
            for (index, card) in state.cards.iter().enumerate() {
                println!("{}: {}, {}", index + 1, card.front, card.back);
            }
        }
    }
    save_state(&args.file, &state);
}
