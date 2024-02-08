// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{io::{BufWriter, Write}, sync::Mutex};

use activities::Activities;
use log::info;
use tauri::{api::dialog::FileDialogBuilder, State};

/// Takes a Result. If it's an error, it sends it to the channel. If it's Ok, continues.
macro_rules! channel_try {
    ($tx:ident, $expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                $tx.send(Err(e.to_string())).unwrap();
                return;
            }
        }
    };
}

mod activities;

#[tauri::command]
/// Start an activity with an offset. If an activity is already running, it will be stopped with the same offset.
/// 
/// # Arguments
/// activity - The name of the activity
/// offset - The offset for the start of the activity in seconds from now 
///     Ex : 0 the activity starts now, 60 for 1 minute from now, -60 for 1 minute ago
fn start_activity(db: State<Mutex<Activities>>, activity: &str, offset: i64) -> Result<(), String> {
    info!("Starting activity with name: {}", activity);
    let activities = db.lock().unwrap();

    activities.start_activity(activity, offset).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
/// Stop the current activity with an offset.
/// 
/// # Arguments
/// offset - The offset for the stop of the activity in seconds from now
///     Ex : 0 the activity stops now, 60 for 1 minute from now, -60 for 1 minute ago
fn stop_activity(db: State<Mutex<Activities>>, offset: i64) -> Result<(), String> {
    let activities = db.lock().unwrap();

    activities.stop_activity(offset).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
/// Get the current activity
/// 
/// # Returns
/// The name of the current activity if there is one
fn get_current_activity(db: State<Mutex<Activities>>) -> Result<String, String> {
    let activities = db.lock().unwrap();

    activities.currrent_activity()
        .map(|activity| activity.map(|(x,_)|x).unwrap_or(String::new()))
        .map_err(|e| e.to_string())
}

#[tauri::command]
/// Get a list of all activities with their cumulative time
/// 
/// # Returns
/// A list of activities with their cumulative time
///     Ex : [("Foo", 3600), ("Bar", 1800), ("Baz", 720)]
///    The time is in seconds
fn get_activities_times(db: State<Mutex<Activities>>) -> Result<Vec<(String, u64)>, String> {
    let activities = db.lock().unwrap();

    activities.activities_times().map_err(|e| e.to_string())
        .map(|activities| activities.into_iter().collect())
}

#[tauri::command]
/// Get a list of all activities
/// 
/// # Returns
/// A list of activities
///    Ex : ["Foo", "Bar", "Baz"]
fn list_activities(db: State<Mutex<Activities>>) -> Result<Vec<String>, String> {
    let activities = db.lock().unwrap();

    activities.list_activities().map_err(|e| e.to_string())
}

#[tauri::command]
/// Mark the current time as the last time the database was cleared
/// Does not clear the data, only marks the time
fn clear_activities(db: State<Mutex<Activities>>) -> Result<(), String> {
    let activities = db.lock().unwrap();

    activities.clear_activities().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
/// Delete all the activities from the database
/// Warning: Unrecoverable!
fn hard_clear_activities(db: State<Mutex<Activities>>) -> Result<(), String> {
    let activities = db.lock().unwrap();

    activities.hard_clear_activities().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
/// Get all the activities for today
///
/// # Returns
/// A list of activities with their start time and end time
///     Ex : [("Foo", 3600, Some(7200)), ("Bar", 1800, None), ("Baz", 720, Some(3600))]
///     The time is in seconds
fn todays_activities(db: State<Mutex<Activities>>) -> Result<Vec<(String, u64, Option<u64>)>, String> {
    let activities = db.lock().unwrap();

    activities.todays_activities().map_err(|e| e.to_string())
}

#[tauri::command(async)]
/// Export activity time to a CSV file
/// 
/// Same as get_activities_times but exports to a CSV file
fn export_activities(db: State<'_, Mutex<Activities>>) -> Result<(), String> {
    let activities = db.lock().unwrap();

    let activities_times = activities.activities_times().map_err(|e| e.to_string())?;
    
    // Unlock the mutex once we have the data to avoid blocking while the user pick a file
    drop(activities);

    let default_path = directories::UserDirs::new().unwrap().document_dir().unwrap().to_owned();

    let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();

    FileDialogBuilder::new()
        .set_directory(default_path)
        .set_title("Save activities to")
        .save_file(move |path|{
            let Some(path) = path else{
                tx.send(Err("No file selected".to_string())).unwrap();
                return;
            };
            let file = std::fs::File::create(path).map_err(|e| e.to_string());
            let file = channel_try!(tx, file);

            let mut wtr = BufWriter::new(file);
            channel_try!(tx, writeln!(wtr, "Activity,Time"));
            for (activity, time) in activities_times {
                let hours = time / 3600;
                let minutes = (time % 3600) / 60;
                channel_try!(tx, writeln!(wtr, "{},{}h{}m", activity, hours, minutes));
            }

            channel_try!(tx, wtr.flush());

            tx.send(Ok(())).unwrap();
        });
    
    rx.recv().unwrap()
}


fn main() {
    simple_logger::init().unwrap();
    let mut data = directories::ProjectDirs::from("fr", "cideco", "activity-tracker").unwrap().data_local_dir().to_owned();
    if !data.exists(){
        std::fs::create_dir_all(&data).unwrap();
    }
    data.push("activity-tracker.db");

    let conn = rusqlite::Connection::open(data).unwrap();
    let activities = activities::Activities::new(conn);
    activities.init_db().expect("Error initiating database");
    let activities = Mutex::from(activities);

    tauri::Builder::default()
        .manage(activities)
        .invoke_handler(tauri::generate_handler![
            start_activity, 
            stop_activity, 
            get_current_activity, 
            get_activities_times,
            list_activities,
            clear_activities,
            hard_clear_activities,
            todays_activities,
            export_activities
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
