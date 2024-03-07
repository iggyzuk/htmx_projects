use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::constants::LETTERS;

pub type GameId = Uuid;

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

    pub fn get_available_letters(&self) -> Vec<char> {
        let mut letters = LETTERS.chars().collect::<Vec<_>>();

        let used_chars = self.guesses.iter().flat_map(|x| x.chars());

        for used_char in used_chars {
            if letters.contains(&used_char) {
                letters.retain(|&c| c != used_char);
            }
        }

        letters
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
