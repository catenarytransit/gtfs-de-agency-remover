use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs::{self, File};

#[derive(Serialize, Deserialize)]
struct RawAgency {
    agency_id: String,
    agency_name: String,
    agency_url: String,
    agency_timezone: String,
    agency_lang: Option<String>,
    agency_phone: Option<String>,
    agency_fare_url: Option<String>,
    agency_email: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct RawRoute {
    pub route_id: String,
    pub route_short_name: Option<String>,
    pub route_long_name: Option<String>,
    pub route_desc: Option<String>,
    pub route_route_type: u8,
    pub route_url: Option<String>,
    pub agency_id: Option<String>,
    pub route_sort_order: Option<u32>,
    pub route_color: Option<String>,
    pub route_text_color: Option<String>,
    pub continuous_pickup: Option<u8>,
    pub continuous_drop_off: Option<u8>,
}

#[derive(Serialize, Deserialize)]
struct RawTrip {
    pub trip_id: String,
    pub service_id: String,
    pub route_id: String,
    pub shape_id: Option<String>,
    pub trip_headsign: Option<String>,
    pub trip_short_name: Option<String>,
    pub direction_id: Option<u8>,
    pub block_id: Option<String>,
    pub wheelchair_accessible: Option<u8>,
    pub bikes_allowed: Option<u8>,
}

#[derive(Serialize, Deserialize)]
struct RawStopTime {
    pub trip_id: String,
    pub arrival_time: Option<String>,
    pub departure_time: Option<String>,
    pub stop_id: String,
    pub stop_sequence: u32,
    pub stop_headsign: Option<String>,
    pub pickup_type: Option<u8>,
    pub drop_off_type: Option<u8>,
}

use std::env;

fn main() {
    //get folder path from argument

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Please provide a folder path as a command-line argument.");
        return;
    }

    let folder_path = &args[1];

    //read agency.txt as csv
    let agency_file_path = format!("{}/agency.txt", folder_path);
    let agency_file =
        File::open(&agency_file_path).expect(&format!("Unable to open file: {}", agency_file_path));

    let mut rdr = csv::Reader::from_reader(agency_file);

    let mut agencies: Vec<RawAgency> = Vec::new();

    for result in rdr.deserialize() {
        if let Ok(record) = result {
            let record: RawAgency = record;
            agencies.push(record);
        }
    }

    let banned_agencies = [
        "SNCF",
        "SNCB",
        "FlixBus-de",
        "FlixTrain-de",
        "SBB",
        "U-Bahn München",
        "Österreichische Bundesbahnen",
    ];

    //let route ids

    let mut route_ids_to_remove: BTreeSet<String> = BTreeSet::new();

    // read routes file

    let routes_file_path = format!("{}/routes.txt", folder_path);
    let routes_file =
        File::open(&routes_file_path).expect(&format!("Unable to open file: {}", routes_file_path));
    let mut rdr = csv::Reader::from_reader(routes_file);

    let mut routes = Vec::new();

    for result in rdr.deserialize() {
        if let Ok(record) = result {
            let record: RawRoute = record;

            //check if agency is in banned agencies
            if banned_agencies.contains(
                &record
                    .agency_id
                    .as_ref()
                    .unwrap_or(&"".to_string())
                    .as_str(),
            ) {
                route_ids_to_remove.insert(record.route_id.clone());
            } else {
                routes.push(record);
            }
        }
    }

    println!("Fixing trips");

    let mut trip_ids_to_remove = BTreeSet::new();

    // read trips file

    let trips_file_path = format!("{}/trips.txt", folder_path);

    let trips_file =
        File::open(&trips_file_path).expect(&format!("Unable to open file: {}", trips_file_path));

    let mut rdr = csv::Reader::from_reader(trips_file);

    let mut trips = Vec::new();

    for result in rdr.deserialize() {
        if let Ok(record) = result {
            let record: RawTrip = record;

            //check if route is in banned routes
            if route_ids_to_remove.contains(&record.route_id) {
                trip_ids_to_remove.insert(record.trip_id.clone());
            } else {
                trips.push(record);
            }
        }
    }

    //write trips back to file

    let trips_new_path = format!("{}/trips_cleaned.txt", folder_path);

    let mut writer = csv::Writer::from_path(&trips_new_path)
        .expect("Unable to create file");

    for trip in trips {
        writer.serialize(trip).expect("Unable to write to file");
    }

    writer.flush().expect("Unable to flush file");

    fs::rename(trips_new_path, trips_file_path).unwrap();

    println!("Fixing stop_times");

    // read stop_times file and at the same time, write to new file

    let stop_times_file_path = format!("{}/stop_times.txt", folder_path);

    let stop_times_file = File::open(&stop_times_file_path)
        .expect(&format!("Unable to open file: {}", stop_times_file_path));

    let mut rdr = csv::Reader::from_reader(stop_times_file);

    let mut writer = csv::Writer::from_path(format!("{}/stop_times_cleaned.txt", folder_path))
        .expect("Unable to create file");

    for result in rdr.deserialize() {
        if let Ok(record) = result {
            let record: RawStopTime = record;

            //check if trip is in banned trips
            if trip_ids_to_remove.contains(&record.trip_id) {
                continue;
            } else {
                writer.serialize(record).expect("Unable to write to file");
            }
        } else {
            eprintln!("Error reading record");
        }
    }

    writer.flush().expect("Unable to flush file");

    fs::rename(format!("{}/stop_times_cleaned.txt", folder_path), stop_times_file_path)
        .unwrap();
}
