use std::collections::HashSet;

pub struct WordState {
    pub letters: Vec<Letter>,
}

pub struct Letter {
    pub id: char,
    pub position: u8,
    pub state: LetterState,
}

impl Letter {
    fn new(id: char, position: u8, state: LetterState) -> Self {
        Self {
            id,
            position,
            state,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum LetterState {
    Correct,
    WrongPlace,
    Wrong,
    Empty,
}

impl WordState {
    pub fn empty() -> Self {
        Self {
            letters: vec![
                Letter::new('-', 0, LetterState::Empty),
                Letter::new('-', 1, LetterState::Empty),
                Letter::new('-', 2, LetterState::Empty),
                Letter::new('-', 3, LetterState::Empty),
                Letter::new('-', 4, LetterState::Empty),
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
                    Letter::new(letter, position as u8, LetterState::Correct)
                } else if word.contains(letter) && !duplicates.contains(&letter) {
                    duplicates.insert(letter);
                    Letter::new(letter, position as u8, LetterState::WrongPlace)
                } else {
                    Letter::new(letter, position as u8, LetterState::Wrong)
                }
            })
            .collect::<Vec<_>>();

        WordState { letters }
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
}
