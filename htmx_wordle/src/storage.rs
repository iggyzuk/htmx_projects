use std::path::Path;

use serde::{Deserialize, Serialize};
use tokio::{
    fs::{create_dir_all, File},
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::{constants::SAVE_DATA_PATH, game::Game};

#[derive(Deserialize, Serialize, Debug)]
pub struct SaveData {
    pub games: Vec<Game>,
}

pub async fn save(games: Vec<Game>) -> Result<(), String> {
    let save_data = SaveData { games };

    // Serialize the data to a JSON string
    let json_string = match serde_json::to_string(&save_data) {
        Ok(json) => json,
        Err(e) => {
            return Err(format!("Error serializing to JSON: {}", e));
        }
    };

    // Path to write the JSON file
    let file_path = Path::new(SAVE_DATA_PATH);

    // Create directories recursively if they don't exist
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            if let Err(e) = create_dir_all(parent).await {
                return Err(format!("Error creating directories: {}", e));
            }
        }
    }

    // Open the file in write mode
    let mut file = match File::create(&file_path).await {
        Ok(file) => file,
        Err(e) => {
            return Err(format!("Error creating file: {}", e));
        }
    };

    // Write the JSON string to the file
    if let Err(e) = file.write_all(json_string.as_bytes()).await {
        return Err(format!("Error writing to file: {}", e));
    }

    println!("{:?}: {}", file_path, json_string);

    Ok(())
}

pub async fn load() -> Result<SaveData, String> {
    // Path to your JSON file
    let file_path = Path::new(SAVE_DATA_PATH);

    // Open the file in read mode
    let mut file = match File::open(&file_path).await {
        Ok(file) => file,
        Err(e) => {
            return Err(format!("Error opening file: {}", e));
        }
    };

    // Read the contents of the file into a String
    let mut contents = String::new();
    if let Err(e) = file.read_to_string(&mut contents).await {
        return Err(format!("Error reading file: {}", e));
    }

    // Deserialize the JSON string into a SaveData struct
    let save_data: SaveData = match serde_json::from_str(&contents) {
        Ok(data) => data,
        Err(e) => {
            return Err(format!("Error deserializing JSON: {}", e));
        }
    };

    println!("{:?}: {:#?}", file_path, save_data);

    Ok(save_data)
}
