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

pub enum LetterState {
    Correct,
    WrongPlace,
    Wrong,
    Empty,
}

pub fn process_word(word: String) -> WordState {
    let word = word.to_lowercase();
    let letters = word
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