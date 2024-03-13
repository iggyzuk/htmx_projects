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
    use serde::Deserialize;

    use super::{Abilities, Ability, Hero, Heroes, Talent, Talents};

    #[derive(Deserialize)]
    struct DeData {
        heroes: Vec<DeHero>,
    }

    #[derive(Deserialize)]
    struct DeHero {
        name: String,
        desc: String,
        icon: String,
        universe: String,
        abilities: Vec<DeAbility>,
        talents: Vec<DeTalent>,
    }

    #[derive(Deserialize)]
    struct DeAbility {
        name: String,
        desc: String,
    }

    #[derive(Deserialize)]
    struct DeTalent {
        name: String,
        desc: String,
        level: u8,
    }

    pub(super) fn parse() -> (Heroes, Abilities, Talents) {
        let data: DeData = match toml::from_str(include_str!("../content.toml")) {
            Ok(data) => data,
            Err(e) => {
                panic!("error deserializing TOML: {}", e);
            }
        };
        (
            Heroes(
                data.heroes
                    .iter()
                    .map(|x| {
                        Hero::new(
                            x.name.clone(),
                            x.desc.clone(),
                            x.icon.clone(),
                            x.universe.clone(),
                        )
                    })
                    .collect(),
            ),
            Abilities(
                data.heroes
                    .iter()
                    .flat_map(|h| h.abilities.iter().map(move |a| (h, a)))
                    .map(|(h, a)| Ability::new(h.name.clone(), a.name.clone(), a.desc.clone()))
                    .collect(),
            ),
            Talents(
                data.heroes
                    .iter()
                    .flat_map(|h| h.talents.iter().map(move |t| (h, t)))
                    .map(|(h, t)| {
                        Talent::new(h.name.clone(), t.name.clone(), t.desc.clone(), t.level)
                    })
                    .collect(),
            ),
        )
    }
}
