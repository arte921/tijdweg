use raylib::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

#[derive(Serialize, Deserialize)]
struct Config {
    weergegeven_stations: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Afstand {
    afstand: f32,
    van: String,
    naar: String,
}

#[derive(Serialize, Deserialize)]
struct Ritdeel {
    vertrektijd: u16,
    aankomsttijd: u16,
    stations: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Tijdweg {
    ritnummer: String,
    tijdwegen: Vec<Ritdeel>,
}

fn main() -> std::io::Result<()> {
    let mut ritjes_json = String::new();
    File::open("opslag/alletijdwegen.json")?.read_to_string(&mut ritjes_json)?;
    let ritjes: Vec<Tijdweg> = serde_json::from_str(&ritjes_json)?;

    let mut afstanden_json = String::new();
    File::open("opslag/featureafstanden.json")?.read_to_string(&mut afstanden_json)?;
    let afstanden: Vec<Afstand> = serde_json::from_str(&afstanden_json)?;

    let mut config_json = String::new();
    File::open("opslag/config.json")?.read_to_string(&mut config_json)?;
    let config: Config = serde_json::from_str(&config_json)?;

    let zichtbaretijdwegen: Vec<Tijdweg> = ritjes
        .into_iter()
        .filter(|rit| {
            rit.tijdwegen.iter().any(|tijdweg| {
                tijdweg.stations.iter().any(|station| {
                    config
                        .weergegeven_stations
                        .iter()
                        .any(|weergegevenstation| weergegevenstation == station)
                })
            })
        })
        .collect();

        

    /*
    let (mut rl, thread) = raylib::init()
        .size(800, 800)
        .title("Tijd-weg diagram")
        .build();

    rl.set_target_fps(60);

    while !rl.window_should_close() {
    }*/

    Ok(())
}
