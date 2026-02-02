use rand::Rng;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use std::fs::File;
use std::io::{self, BufRead, stdin};

struct Flashcard {
    front: String,
    back: String,
    correct: u32,
    incorrect: u32,
}

impl Flashcard {
    fn print_front(&self) {
        println!("Front:  {}", self.front);
    }
    fn print_back(&self) {
        println!("Back:  {}", self.back);
    }

    fn correct(&mut self) {
        self.correct += 1;
    }
    fn incorrect(&mut self) {
        self.incorrect += 1;
    }
}

fn parse_card(line: &str) -> Option<Flashcard> {
    let mut parts = line.splitn(2, ';');
    let front = parts.next().expect("Missing front").trim();
    let back = parts.next().expect("Missing back").trim();

    Some(Flashcard {
        front: front.to_string(),
        back: back.to_string(),
        correct: 0,
        incorrect: 0,
    })
}

fn load_cards(file: &File) -> io::Result<Vec<Flashcard>> {
    let mut flashcards: Vec<Flashcard> = Vec::new();
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line.expect("Line malfunction");

        if line.trim().is_empty() {
            continue;
        }

        flashcards.push(parse_card(&line).unwrap());
    }

    Ok(flashcards)
}

fn random_index(flashcards: &Vec<Flashcard>) -> usize {
    let mut rng = rand::rng();
    rng.random_range(..flashcards.len())
}

fn weight(card: &Flashcard) -> f64 {
    let total = card.correct + card.incorrect;
    if total == 0 {
        return 3.0;
    }

    (card.incorrect + 1) as f64 / (total as f64)
}

fn random_weighted_index(flashcards: &mut Vec<Flashcard>) -> Option<usize> {
    if flashcards.is_empty() {
        return None;
    }

    let weights: Vec<f64> = flashcards.iter().map(weight).collect();
    let dist = WeightedIndex::new(&weights).unwrap();
    let mut rng = rand::rng();

    Some(dist.sample(&mut rng))
}

fn learn_cards(flashcards: &mut Vec<Flashcard>) {
    loop {
        let index = random_weighted_index(flashcards).expect("Cant pick weighted random card");
        flashcards[index].print_front();
        println!("f to flip. Did you know it?");

        loop {
            let mut input = String::new();
            stdin().read_line(&mut input).expect("Failed to get input");

            match input.to_lowercase().trim() {
                "flip" | "f" => println!("Back: {}", flashcards[index].back),
                "yes" | "y" => {
                    flashcards[index].correct();
                    break;
                }
                "no" | "n" => {
                    flashcards[index].incorrect();
                    break;
                }
                "quit" | "q" => return,
                _ => println!("Command does not exist"),
            };
            println!(
                "corr: {}, incorr {}",
                flashcards[index].correct, flashcards[index].incorrect
            );
        }
    }
}

fn main() {
    let file = File::open("cards/card_1.csv").expect("File could not be found");
    let mut flashcards = load_cards(&file).expect("Could not load flashcards");

    let mut index = random_index(&flashcards);
    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to get input");

        match input.to_lowercase().trim() {
            "learn" => learn_cards(&mut flashcards),
            "show" | "s" => println!("Front: {}", flashcards[index].front),
            "next" | "n" => {
                index = random_index(&flashcards);
                println!("Front: {}", flashcards[index].front)
            }
            "flip" | "f" => println!("Back: {}", flashcards[index].back),
            "quit" | "q" => break,
            _ => println!("Command does not exist"),
        };
    }
}
