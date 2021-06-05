use raylib::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

struct Stationpositie {
    station: String,
    positie: f64
}

#[derive(Serialize, Deserialize)]
struct Config {
    weergegeven_stations: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Afstand {
    afstand: f64,
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

fn gemeenschappelijk_station(ritdeel: &Ritdeel, config: &Config, eerste: bool) -> String {
    let stations = ritdeel.stations.into_iter();
    if !eerste {
        stations.rev();
    }

    match stations.find(|ritstation| config.weergegeven_stations.iter().any(|weergegevenstation| &weergegevenstation == ritstation)) {
        None => "e".to_string(),
        Some(station) => station.to_string()
    }
}

fn vindt_afstand(afstanden: &Vec<Afstand>, stationa: String, stationb: String) -> f64 {
    match afstanden
        .iter()
        .find(|feature| (feature.van == stationa && feature.naar == stationb) || (feature.naar == stationa && feature.van == stationb))
    {
        None => 0.0,
        Some(feature) => feature.afstand
    }
}

fn tijd_in_station(ritdeel: &Ritdeel, station: String, afstanden: &Vec<Afstand>) -> f64 {
    let mut totale_afstand = 0.0;

    let mut station_afstand = 0.0; 

    for i in 1..ritdeel.stations.len() {
        totale_afstand += vindt_afstand(afstanden, ritdeel.stations[i - 1].to_string(), ritdeel.stations[i].to_string());
        
        if ritdeel.stations[i] == station {
            station_afstand = totale_afstand;
        }
    }

    (ritdeel.vertrektijd - ritdeel.aankomsttijd) as f64 * totale_afstand / station_afstand
}

fn bereken_station_relatieve_positie(stations: Vec<String>, afstanden: Vec<Afstand>) -> Vec<Stationpositie> {
    let mut resultaat: Vec<Stationpositie> = vec![Stationpositie {
        station: stations[0],
        positie: 0.0
    }];

    let mut totaleafstand = 0.0;
    
    for i in 1..stations.len() {
        let afstand = vindt_afstand(&afstanden, stations[i - 1], stations[i]);
        totaleafstand += afstand;

        resultaat.push(Stationpositie {
            station: stations[i],
            positie: afstand
        });
    }

    resultaat
        .into_iter()
        .map(|stationpositie| Stationpositie {
            station: stationpositie.station,
            positie: stationpositie.positie / totaleafstand
        })
        .collect()
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

    let stationsafstanden = bereken_station_relatieve_positie(config.weergegeven_stations, afstanden);

    let zichtbaretijdwegen: Vec<Rit> = ritjes
        .into_iter()
        .filter(|rit| {
            rit.tijdwegen.iter().any(|ritdeel| {
                wordt_weergegeven(ritdeel, &config)
            })
        })
        .collect();

    
    let (mut rl, thread) = raylib::init()
        .size(800, 800)
        .title("Tijd-weg diagram")
        .build();

    rl.set_target_fps(60);

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread); 
        d.clear_background(Color::BLACK);

        for rit in &zichtbaretijdwegen {
            for ritdeel in &rit.tijdwegen {
                if !wordt_weergegeven(ritdeel, &config) {
                    continue;
                }

                let beginstation = gemeenschappelijk_station(&ritdeel, &config, true);
                let eindstation = gemeenschappelijk_station(&ritdeel, &config, false);

                let begintijd = tijd_in_station(ritdeel, beginstation, &afstanden);
                let eindtijd = tijd_in_station(ritdeel, eindstation, &afstanden);

                let begin_x_breuk = stationsafstanden.iter().find(|station| station.station == beginstation).positie;
                let begin_y_breuk = stationsafstanden.iter().find(|station| station.station == eindstation).positie;
                
            }
        }
    }

    Ok(())
}
