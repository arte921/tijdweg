use raylib::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;

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
struct Rit {
    ritnummer: String,
    tijdwegen: Vec<Ritdeel>,
}

fn wordt_weergegeven(ritdeel: &Ritdeel, config: &Config) -> bool {
    ritdeel.stations.iter().any(|station| {
        config
            .weergegeven_stations
            .iter()
            .any(|weergegevenstation| weergegevenstation == station)
    })
}

fn tijd_in_station(ritdeel: &Ritdeel, station: String, afstanden: &Vec<Afstand>) -> f32 {
    let mut totale_afstand = 0.0;

    let mut station_afstand = 0.0; 

    for i in 1..ritdeel.stations.len() {
        totale_afstand += match afstanden
            .iter()
            .find(|feature| 
                (feature.van == ritdeel.stations[i - 1] && feature.naar == ritdeel.stations[i]) ||
                (feature.naar == ritdeel.stations[i - 1] && feature.van == ritdeel.stations[i])
            )
            {
                None => 0.0,
                Some(feature) => feature.afstand
            };
        
        if ritdeel.stations[i] == station {
            station_afstand = totale_afstand;
        }
    }

    (ritdeel.vertrektijd - ritdeel.aankomsttijd) as f32 * totale_afstand / station_afstand
}

fn main() -> std::io::Result<()> {
    let mut ritjes_json = String::new();
    File::open("opslag/alletijdwegen.json")?.read_to_string(&mut ritjes_json)?;
    let ritjes: Vec<Rit> = serde_json::from_str(&ritjes_json)?;

    let mut afstanden_json = String::new();
    File::open("opslag/featureafstanden.json")?.read_to_string(&mut afstanden_json)?;
    let afstanden: Vec<Afstand> = serde_json::from_str(&afstanden_json)?;

    let mut config_json = String::new();
    File::open("opslag/config.json")?.read_to_string(&mut config_json)?;
    let config: Config = serde_json::from_str(&config_json)?;

    let zichtbaretijdwegen: Vec<Rit> = ritjes
        .into_iter()
        .filter(|rit| {
            rit.tijdwegen.iter().any(|ritdeel| {
                wordt_weergegeven(ritdeel, &config)
            })
        })
        .collect();

    
    let (mut rl, _) = raylib::init()
        .size(800, 800)
        .title("Tijd-weg diagram")
        .build();

    rl.set_target_fps(60);

    while !rl.window_should_close() {
        for rit in &zichtbaretijdwegen {
            for ritdeel in &rit.tijdwegen {
                if !wordt_weergegeven(ritdeel, &config) {
                    continue;
                }


            }
        }
    }

    Ok(())
}
