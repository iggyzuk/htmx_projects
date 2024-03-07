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

#[derive(PartialEq, Eq)]
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
        let letters = guess
            .to_lowercase()
            .chars()
            .enumerate()
            .map(|(position, letter)| {
                if word.chars().nth(position).unwrap() == letter {
                    Letter::new(letter, position as u8, LetterState::Correct)
                } else if word.contains(letter) {
                    Letter::new(letter, position as u8, LetterState::WrongPlace)
                } else {
                    Letter::new(letter, position as u8, LetterState::Wrong)
                }
            })
            .collect::<Vec<_>>();

        WordState { letters }
    }
}
