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
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
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
    cards: Vec<Flashcard>,
}

impl DeckState {
    fn new() -> Self {
        Self { cards: Vec::new() }
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

    fn get_card(&self, index: usize) -> &Flashcard {
        &self.cards[index]
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
            if state.cards.len() <= 0 {
                println!("Deck is empty");
            } else {
                ratatui::run(|terminal| FlipApp::new(&mut state, order).run(terminal))?;
            }
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

struct FlipApp {
    should_exit: bool,
    deck: Vec<Flashcard>,
    order: Order,
    show_side: Side,
    index: usize,
}

impl FlipApp {
    fn new(deck_state: &DeckState, order: Order) -> Self {
        Self {
            should_exit: false,
            deck: deck_state.cards.clone(),
            index: 0,
            order,
            show_side: Side::Front,
        }
    }
    
    fn flip_card(&mut self) {
        self.show_side = self.show_side.toggle();
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| self.render(frame))?;

            if let Some(key) = event::read()?.as_key_press_event() {
                match key.code {
                    KeyCode::Char('q') => self.should_exit = true,
                    KeyCode::Char('f') => self.flip_card(),
                    KeyCode::Char('n') => {
                        if self.index < self.deck.len() - 1 {
                            self.index += 1;
                            self.flip_card();
                        }
                    }
                    KeyCode::Char('b') => {
                        if self.index > 0 {
                            self.index -= 1;
                            self.flip_card();
                        }
                    }
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
                Constraint::Length(3),  // progress
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
            .split(vertical[2]);

        let card = Block::default()
            .title("Flashcard")
            .title_bottom("q[uit], f[lip], n[ext], b[ack]")
            .borders(Borders::ALL);

        let paragraph = Paragraph::new(
            {
                let card = &self.deck[self.index];

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

        let progress_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(40), // flashcard horizontal
                Constraint::Min(0),
            ])
            .split(vertical[1]);

        let progress = (self.index + 1) as f64 / self.deck.len() as f64;
        let gauge = Gauge::default()
            .block(Block::default().title("Progress").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Green))
            .ratio(progress)
            .label(format!(
                "{}/{}",
                self.index + 1,
                self.deck.len()
            ));

        frame.render_widget(paragraph, horizontal[1]);
        frame.render_widget(gauge, progress_horizontal[1]);
    }
}
