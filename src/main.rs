use std::{
    fs, io,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand, ValueEnum};

use serde::{Deserialize, Serialize};

use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Alignment, Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph, Wrap},
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

    fn current_card(&self) -> &Flashcard {
        &self.cards[self.current_index]
    }

    fn increment_index(&mut self) {
        if self.current_index < self.cards.len() - 1 {
            self.current_index += 1;
        }
    }

    fn decrement_index(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
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
    Add {
        front: String,
        back: String,
    },
    Remove {
        index: usize,
    },
    List,
    Flip {
        #[arg(default_value = "sequential")]
        order: Order,
    },
}

#[derive(ValueEnum, Debug, Clone)]
enum Order {
    Sequential,
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

fn main() -> io::Result<()> {
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

        Action::Flip { order } => {
            ratatui::run(|terminal| FlipApp::new(&mut state, order).run(terminal))?;
        }
    }
    save_state(&args.file, &state);

    Ok(())
}

#[derive(Clone)]
enum Side {
    Front,
    Back,
}

impl Side {
    fn toggle(&self) -> Self {
        match self {
            Side::Front => Side::Back,
            Side::Back => Side::Front,
        }
    }
}

struct FlipApp<'a> {
    should_exit: bool,
    deck_state: &'a mut DeckState,
    order: Order,
    show_side: Side,
    index: usize,
}

impl<'a> FlipApp<'a> {
    fn new(deck_state: &'a mut DeckState, order: Order) -> Self {
        Self {
            should_exit: false,
            deck_state,
            order,
            show_side: Side::Front,
            index: 0,
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| self.render(frame))?;

            if let Some(key) = event::read()?.as_key_press_event() {
                match key.code {
                    KeyCode::Char('q') => self.should_exit = true,
                    KeyCode::Char('f') => self.show_side = self.show_side.toggle(),
                    KeyCode::Char('n') => self.deck_state.increment_index(),
                    KeyCode::Char('b') => self.deck_state.decrement_index(),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn render(&self, frame: &mut ratatui::Frame) {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(10), // flashcard vertical
                Constraint::Min(0),
            ])
            .split(frame.area());

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(40), // flashcard horizontal
                Constraint::Min(0),
            ])
            .split(vertical[1]);

        let card = Block::default()
            .title("Flashcard")
            .title_bottom("q[uit], f[lip], n[ext], b[ack]")
            .borders(Borders::ALL);

        let paragraph = Paragraph::new(
            {
                let card = self.deck_state.current_card();

                match self.show_side {
                    Side::Front => &card.front,
                    Side::Back => &card.back,
                }
            }
            .clone(),
        )
        .alignment(Alignment::Center)
        .block(card)
        .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, horizontal[1]);
    }
}
