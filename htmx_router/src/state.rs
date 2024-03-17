use std::sync::Arc;

use tokio::sync::RwLock;

fn into_id(id: &String) -> String {
    // Replace spaces with dashes and remove strange characters
    id.to_lowercase()
        .chars()
        .filter_map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                Some(c)
            } else if c == ' ' {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
}

pub(crate) struct AppState {
    pub(crate) heroes: RwLock<Heroes>,
    pub(crate) abilities: RwLock<Abilities>,
    pub(crate) talents: RwLock<Talents>,
}

impl AppState {
    pub(crate) fn new() -> Arc<Self> {
        let (heroes, abilities, talents) = deserialize::parse();

        Arc::new(AppState {
            heroes: RwLock::new(heroes),
            abilities: RwLock::new(abilities),
            talents: RwLock::new(talents),
        })
    }
}

pub(crate) struct Heroes(pub(crate) Vec<Hero>);

impl Heroes {
    pub(crate) fn read<T: AsRef<str>>(&self, id: T) -> Option<&Hero> {
        self.0.iter().find(|t| t.id == id.as_ref())
    }
}

pub(crate) struct Hero {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) desc: String,
    pub(crate) icon: String,
    pub(crate) universe: String,
}

impl Hero {
    fn new(name: String, desc: String, icon: String, universe: String) -> Self {
        Self {
            id: into_id(&name),
            name,
            desc,
            icon,
            universe,
        }
    }
}

pub(crate) struct Abilities(pub(crate) Vec<Ability>);

impl Abilities {
    pub(crate) fn read<T: AsRef<str>>(&self, id: T) -> Option<&Ability> {
        self.0.iter().find(|t| t.id == id.as_ref())
    }
    pub(crate) fn for_hero(&self, id: &String) -> Vec<&Ability> {
        self.0
            .iter()
            .filter(|a| a.hero_id == *id)
            .collect::<Vec<_>>()
    }
}

pub(crate) struct Ability {
    pub(crate) id: String,
    pub(crate) hero_id: String,
    pub(crate) name: String,
    pub(crate) desc: String,
}

impl Ability {
    fn new(hero_id: String, name: String, desc: String) -> Self {
        Self {
            id: into_id(&name),
            hero_id: into_id(&hero_id),
            name,
            desc,
        }
    }
}

pub(crate) struct Talents(pub(crate) Vec<Talent>);

impl Talents {
    pub(crate) fn read<T: AsRef<str>>(&self, id: T) -> Option<&Talent> {
        self.0.iter().find(|t| t.id == id.as_ref())
    }

    pub(crate) fn for_hero(&self, id: &String) -> Vec<&Talent> {
        self.0
            .iter()
            .filter(|a| a.hero_id == *id)
            .collect::<Vec<_>>()
    }
}

pub(crate) struct Talent {
    pub(crate) id: String,
    pub(crate) hero_id: String,
    pub(crate) name: String,
    pub(crate) desc: String,
    pub(crate) level: u8,
}

impl Talent {
    fn new(hero_id: String, name: String, desc: String, level: u8) -> Self {
        Self {
            id: into_id(&name),
            hero_id: into_id(&hero_id),
            name,
            desc,
            level,
        }
    }
}

mod deserialize {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    struct Data {
        heroes: Vec<Hero>,
    }

    #[derive(Deserialize, Serialize)]
    struct Hero {
        name: String,
        desc: String,
        icon: String,
        universe: String,
        abilities: Vec<Ability>,
        talents: Vec<Talent>,
    }

    #[derive(Deserialize, Serialize)]
    struct Ability {
        name: String,
        desc: String,
    }

    #[derive(Deserialize, Serialize)]
    struct Talent {
        name: String,
        desc: String,
        level: u8,
    }

    pub(super) fn parse() -> (super::Heroes, super::Abilities, super::Talents) {
        let data: Data = match ron::from_str(include_str!("../content.ron")) {
            Ok(data) => data,
            Err(e) => {
                panic!("error deserializing RON: {}", e);
            }
        };

        // let x = ron::to_string(&data).unwrap();
        // println!("{x}");

        (
            super::Heroes(
                data.heroes
                    .iter()
                    .map(|x| {
                        super::Hero::new(
                            x.name.clone(),
                            x.desc.clone(),
                            x.icon.clone(),
                            x.universe.clone(),
                        )
                    })
                    .collect(),
            ),
            super::Abilities(
                data.heroes
                    .iter()
                    .flat_map(|h| h.abilities.iter().map(move |a| (h, a)))
                    .map(|(h, a)| {
                        super::Ability::new(h.name.clone(), a.name.clone(), a.desc.clone())
                    })
                    .collect(),
            ),
            super::Talents(
                data.heroes
                    .iter()
                    .flat_map(|h| h.talents.iter().map(move |t| (h, t)))
                    .map(|(h, t)| {
                        super::Talent::new(h.name.clone(), t.name.clone(), t.desc.clone(), t.level)
                    })
                    .collect(),
            ),
        )
    }
}
