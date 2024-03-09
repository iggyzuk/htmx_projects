use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub const LETTERS: &'static str = "qwertyuiopasdfghjklzxcvbnm";

pub type GameId = Uuid;

pub fn short_id(id: Uuid) -> String {
    id.to_string().chars().take(8).collect::<String>()
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Game {
    pub id: Uuid,
    pub word: String,
    pub guesses: Vec<String>,
    pub created: Option<DateTime<Utc>>,
}

impl Game {
    pub fn new(id: Uuid, word: String) -> Self {
        Self {
            id,
            word,
            guesses: vec![],
            created: Some(Utc::now()),
        }
    }

    pub fn is_complete(&self) -> bool {
        self.guesses.len() >= 6 || self.is_victory()
    }

    pub fn is_victory(&self) -> bool {
        self.guesses.contains(&self.word)
    }

    pub fn is_loss(&self) -> bool {
        self.guesses.len() >= 6 && !self.is_victory()
    }

    pub fn add_guess(&mut self, word: String) {
        self.guesses.push(word);
    }

    pub fn get_guesses(&self) -> Vec<Option<String>> {
        let mut result = vec![];
        for i in 0..6 {
            if let Some(guess) = self.guesses.get(i) {
                result.push(Some(guess.clone()));
            } else {
                result.push(None);
            }
        }
        result
    }

    pub fn get_available_letters(&self) -> HashMap<char, Letter> {
        let mut letter_map: HashMap<char, Letter> = HashMap::new();

        self.guesses
            .iter()
            .map(|guess| WordState::guess(guess, &self.word))
            .flat_map(|state| state.letters)
            .for_each(|letter| {
                if let Some(existing_letter) = letter_map.get_mut(&letter.id) {
                    // New letter state is better than the previous? e.g. Correct > WrongPlace
                    if letter.state > existing_letter.state {
                        letter_map.insert(letter.id, letter);
                    }
                } else {
                    letter_map.insert(letter.id, letter);
                }
            });

        // Add all other characters (they're all empty)
        LETTERS.chars().for_each(|ch| {
            if !letter_map.contains_key(&ch) {
                letter_map.insert(ch, Letter::new(ch, LetterState::Empty));
            }
        });

        letter_map
    }
}

pub struct WordState {
    pub letters: Vec<Letter>,
}

impl WordState {
    pub fn empty() -> Self {
        Self {
            letters: vec![
                Letter::new('-', LetterState::Empty),
                Letter::new('-', LetterState::Empty),
                Letter::new('-', LetterState::Empty),
                Letter::new('-', LetterState::Empty),
                Letter::new('-', LetterState::Empty),
            ],
        }
    }

    pub fn guess(guess: &String, word: &String) -> WordState {
        let mut duplicates = HashSet::new();
        let letters = guess
            .to_lowercase()
            .chars()
            .enumerate()
            .map(|(position, letter)| {
                if word.chars().nth(position).unwrap() == letter {
                    Letter::new(letter, LetterState::Correct)
                } else if word.contains(letter) && !duplicates.contains(&letter) {
                    duplicates.insert(letter);
                    Letter::new(letter, LetterState::WrongPlace)
                } else {
                    Letter::new(letter, LetterState::Wrong)
                }
            })
            .collect::<Vec<_>>();

        WordState { letters }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Letter {
    pub id: char,
    pub state: LetterState,
}

impl Letter {
    fn new(id: char, state: LetterState) -> Self {
        Self { id, state }
    }
}

#[derive(PartialEq, Eq, Debug, Ord)]
pub enum LetterState {
    Correct,
    WrongPlace,
    Wrong,
    Empty,
}

impl PartialOrd for LetterState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Note: compare in reverse so that we can do: Correct(0) > Wrong(1)
        Some(other.cmp(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guess_word_into_state() {
        let state = WordState::guess(&"smell".to_string(), &"state".to_string());
        assert_eq!(state.letters[0].state, LetterState::Correct);
        assert_eq!(state.letters[1].state, LetterState::Wrong);
        assert_eq!(state.letters[2].state, LetterState::WrongPlace);
        assert_eq!(state.letters[3].state, LetterState::Wrong);
        assert_eq!(state.letters[4].state, LetterState::Wrong);
    }

    #[test]
    fn guess_with_double_letter_into_word_state() {
        let state = WordState::guess(&"smell".to_string(), &"slate".to_string());
        assert_eq!(state.letters[0].state, LetterState::Correct);
        assert_eq!(state.letters[1].state, LetterState::Wrong);
        assert_eq!(state.letters[2].state, LetterState::WrongPlace);
        assert_eq!(state.letters[3].state, LetterState::WrongPlace); // First 'l' was here
        assert_eq!(state.letters[4].state, LetterState::Wrong); // Duplicate 'l' is considered wrong
    }

    #[test]
    fn quick_victory() {
        let mut game = Game::new(Uuid::new_v4(), "final".to_string());
        game.add_guess("guess".to_string());
        assert_eq!(game.is_complete(), false);
        game.add_guess("final".to_string());
        assert_eq!(game.is_complete(), true);
        assert_eq!(game.is_victory(), true);
        assert_eq!(game.is_loss(), false);
    }

    #[test]
    fn victory() {
        let mut game = Game::new(Uuid::new_v4(), "final".to_string());
        game.add_guess("crane".to_string());
        assert_eq!(game.is_complete(), false);
        game.add_guess("pilot".to_string());
        assert_eq!(game.is_complete(), false);
        game.add_guess("husky".to_string());
        assert_eq!(game.is_complete(), false);
        game.add_guess("badge".to_string());
        assert_eq!(game.is_complete(), false);
        game.add_guess("epoxy".to_string());
        assert_eq!(game.is_complete(), false);
        game.add_guess("final".to_string());
        assert_eq!(game.is_complete(), true);
        assert_eq!(game.is_victory(), true);
        assert_eq!(game.is_loss(), false);
    }

    #[test]
    fn loss() {
        let mut game = Game::new(Uuid::new_v4(), "final".to_string());
        game.add_guess("crane".to_string());
        assert_eq!(game.is_complete(), false);
        game.add_guess("pilot".to_string());
        assert_eq!(game.is_complete(), false);
        game.add_guess("husky".to_string());
        assert_eq!(game.is_complete(), false);
        game.add_guess("badge".to_string());
        assert_eq!(game.is_complete(), false);
        game.add_guess("epoxy".to_string());
        assert_eq!(game.is_complete(), false);
        game.add_guess("wrong".to_string());
        assert_eq!(game.is_complete(), true);
        assert_eq!(game.is_victory(), false);
        assert_eq!(game.is_loss(), true);
    }
}
