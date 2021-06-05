const SCHERMBREEDTE: u32 = 800;
const SCHERMHOOGTE: u32 = 800;

const TIJDBEGIN: u16 = 7 * 60;
const TIJDSCHAAL: f32 = 0.1;

use raylib::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

struct Stationpositie {
    station: String,
    positie: f32
}

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

fn gemeenschappelijke_stations(ritdeel: &Ritdeel, config: &Config) -> (String, String) {
    let mut allestations = ritdeel
        .stations
        .iter()
        .filter(|ritstation| config
            .weergegeven_stations
            .iter()
            .any(|weergegevenstation| weergegevenstation == *ritstation)
        );

    match (allestations.next(), allestations.next()) {
        (Some(stationa), Some(stationb)) => (stationa.to_string(), stationb.to_string()),
        (Some(stationa), None) => (stationa.to_string(), stationa.to_string()),
        _ => panic!()
    }
}

fn vindt_afstand(afstanden: &Vec<Afstand>, stationa: String, stationb: String) -> f32 {
    match afstanden
        .iter()
        .find(|feature| (feature.van == stationa && feature.naar == stationb) || (feature.naar == stationa && feature.van == stationb))
    {
        None => panic!(),
        Some(feature) => feature.afstand
    }
}

fn tijd_in_station(ritdeel: &Ritdeel, station: String, afstanden: &Vec<Afstand>) -> u16 {
    let mut totale_afstand = 0.0;

    let mut station_afstand = 0.0; 

    for i in 1..ritdeel.stations.len() {
        totale_afstand += vindt_afstand(afstanden, ritdeel.stations[i - 1].to_string(), ritdeel.stations[i].to_string());
        
        if ritdeel.stations[i] == station {
            station_afstand = totale_afstand;
        }
    }

    ((ritdeel.aankomsttijd - ritdeel.vertrektijd) as f32 * totale_afstand / station_afstand) as u16
}

fn vind_station(stationsposities: &Vec<Stationpositie>, stationscode: String) -> &Stationpositie {
    match stationsposities.into_iter().find(|station| station.station == stationscode) {
        None => panic!(),
        Some(station) => station
    }
}

fn bereken_station_relatieve_positie(stations: &Vec<String>, afstanden: &Vec<Afstand>) -> Vec<Stationpositie> {
    let mut resultaat: Vec<Stationpositie> = vec![Stationpositie {
        station: stations[0].to_string(),
        positie: 0.0
    }];

    let mut totaleafstand = 0.0;
    
    for i in 1..stations.len() {
        let afstand = vindt_afstand(&afstanden, stations[i - 1].to_string(), stations[i].to_string());
        totaleafstand += afstand;

        resultaat.push(Stationpositie {
            station: stations[i].to_string(),
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

    let stationsafstanden = bereken_station_relatieve_positie(&config.weergegeven_stations, &afstanden);

    let zichtbaretijdwegen: Vec<Rit> = ritjes
        .into_iter()
        .filter(|rit| {
            rit.tijdwegen.iter().any(|ritdeel| {
                wordt_weergegeven(ritdeel, &config)
            })
        })
        .collect();

    
    let (mut rl, thread) = raylib::init()
        .size(SCHERMBREEDTE as i32, SCHERMHOOGTE as i32)
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

                let (beginstation, eindstation) = gemeenschappelijke_stations(&ritdeel, &config);

                let begintijd = tijd_in_station(ritdeel, beginstation.to_string(), &afstanden);
                let eindtijd = tijd_in_station(ritdeel, eindstation.to_string(), &afstanden);

                let begin_x_breuk = vind_station(&stationsafstanden, beginstation.to_string()).positie;
                let eind_x_breuk = vind_station(&stationsafstanden, eindstation.to_string()).positie;

                let lijn_start_coordinaat = Vector2 {
                    x: begin_x_breuk * SCHERMBREEDTE as f32,
                    y: (begintijd as f32 - TIJDBEGIN as f32) * TIJDSCHAAL                    
                };

                let lijn_eind_coordinaat = Vector2 {
                    x: eind_x_breuk * SCHERMBREEDTE as f32,
                    y: (eindtijd as f32 - TIJDBEGIN as f32) as f32 * TIJDSCHAAL                    
                };
                
                d.draw_line_ex(lijn_start_coordinaat, lijn_eind_coordinaat, 1.0, Color::YELLOW);
            }
        }
    }

    Ok(())
}
