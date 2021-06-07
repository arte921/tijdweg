const SCHERMBREEDTE: u32 = 800;
const SCHERMHOOGTE: u32 = 800;
const ZIJMARGE: u32 = 100;

const SCROLLSNELHEID: f32 = 30.0;
const ZOOMSNELHEID: f32 = 2.0;

const TEKENSCHERMBREEDTE: u32 = SCHERMBREEDTE - ZIJMARGE;
const TEKENSCHERMHOOGTE: u32 = SCHERMHOOGTE;


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
    
    
    match (allestations.next(), allestations.last()) {
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
            positie: totaleafstand
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

    let mut tijdbegin: u16 = 420;
    // minuten per pixel
    let mut tijdsschaal: f32 = 10.0;

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread); 
        d.clear_background(Color::BLACK);


        if d.is_key_down(KeyboardKey::KEY_UP)  {
            let verschil = (SCROLLSNELHEID / tijdsschaal) as u16;
            tijdbegin = if verschil > tijdbegin {
                0
            } else {
                tijdbegin - verschil
            };
        }

        if d.is_key_down(KeyboardKey::KEY_DOWN) {
            tijdbegin += (SCROLLSNELHEID / tijdsschaal) as u16;
        }

        if d.is_key_down(KeyboardKey::KEY_P) {
            tijdsschaal *= ZOOMSNELHEID;
        }

        if d.is_key_down(KeyboardKey::KEY_O) {
            tijdsschaal /= ZOOMSNELHEID;
        }

        for station in &stationsafstanden {
            let tekenx = (station.positie * TEKENSCHERMBREEDTE as f32) as i32;

            d.draw_line(
                tekenx,
                0,
                tekenx,
                TEKENSCHERMHOOGTE as i32,
                Color::WHITE
            );

            d.draw_text(
                &station.station.to_string(),
                tekenx,
                0,
                30,
                Color::GRAY
            );
        }

        for uur in 0..=23 {

            let uurtekenhoogte = (uur * 60 - tijdbegin as i32) * tijdsschaal as i32;

            d.draw_text(
                &format!("{}:00", uur),
                TEKENSCHERMBREEDTE as i32,
                uurtekenhoogte,
                30,
                Color::RED
            );

            d.draw_line(
                0,
                uurtekenhoogte,
                TEKENSCHERMBREEDTE as i32,
                uurtekenhoogte,
                Color::LIGHTGRAY
            );

            for tiendeminuut in 1..=5 {
                let tiendeminuut_tekenhoogte = uurtekenhoogte + ((10 * tiendeminuut) as f32 * tijdsschaal) as i32;

                d.draw_line(
                    0,
                    tiendeminuut_tekenhoogte,
                    TEKENSCHERMBREEDTE as i32,
                    tiendeminuut_tekenhoogte,
                    if tiendeminuut == 3 {
                        Color::GRAY
                    } else {
                        Color::DARKGRAY
                    }
                );
            }
        }

        for rit in &zichtbaretijdwegen {
            for ritdeel in &rit.tijdwegen {
                if !wordt_weergegeven(ritdeel, &config) {
                    continue;
                }

                let ritduur = ritdeel.aankomsttijd - ritdeel.vertrektijd;

                let (beginstation, eindstation) = gemeenschappelijke_stations(&ritdeel, &config);

                let ritafstanden = bereken_station_relatieve_positie(&ritdeel.stations, &afstanden);

                let begintijd = (vind_station(&ritafstanden, beginstation.to_string()).positie * ritduur as f32) as u16 + ritdeel.vertrektijd;
                let eindtijd = (vind_station(&ritafstanden, eindstation.to_string()).positie * ritduur as f32) as u16 + ritdeel.vertrektijd;

                let begin_x_breuk = vind_station(&stationsafstanden, beginstation.to_string()).positie;
                let eind_x_breuk = vind_station(&stationsafstanden, eindstation.to_string()).positie;

                let lijn_start_coordinaat = Vector2 {
                    x: begin_x_breuk * TEKENSCHERMBREEDTE as f32,
                    y: (begintijd as f32 - tijdbegin as f32) * tijdsschaal 
                };

                let lijn_eind_coordinaat = Vector2 {
                    x: eind_x_breuk * TEKENSCHERMBREEDTE as f32,
                    y: (eindtijd as f32 - tijdbegin as f32) as f32 * tijdsschaal                    
                };
                
                d.draw_line_ex(lijn_start_coordinaat, lijn_eind_coordinaat, 1.0, Color::YELLOW);
            }
        }
    }

    Ok(())
}
