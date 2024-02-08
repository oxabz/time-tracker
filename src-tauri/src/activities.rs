use std::{collections::HashMap, time::SystemTime};

use rusqlite::{params, Connection};


/**
A struct serving as an interface to the activities table in the database.

# Table Structure
- Activities
    - id - The unique identifier for the activity
    - name - The name of the activity it identifies the activity
    - start_time - The time the activity started in seconds since the epoch
    - end_time - The time the activity ended in seconds since the epoch if it has ended
- Clears
    - id - The unique identifier for the clear
    - time - The time the clear was performed in seconds since the epoch
    */
pub struct Activities(Connection);

impl Activities {
    /**
    Create a new instance of the Activities struct.

    # Arguments
    conn - A Connection to the database
     */
    pub fn new(conn: Connection) -> Self {
        Self(conn)
    }

    /**
    Initialize the database with the activities and clears tables.

     */
    pub fn init_db(&self) -> Result<(), rusqlite::Error> {
        // Setup the activities table
        self.0
            .execute(
                "
                CREATE TABLE IF NOT EXISTS activities (
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    start_time INTEGER NOT NULL,
                    end_time INTEGER
                )",
                [],
            )?;
        // Setup the clears table
        self.0
            .execute(
                "
                CREATE TABLE IF NOT EXISTS clears (
                    id INTEGER PRIMARY KEY,
                    time INTEGER NOT NULL
                )
                ",
                [],
            )?;
        // Add the first clear at UNIX EPOCH to make sure all the activities are counted
        self.0.execute("INSERT INTO clears(id, time) VALUES (1, 0) ON CONFLICT DO NOTHING;", [])?;
        Ok(())
    }

    /**
    Start an activity with the given name.

    # Arguments
    name - The name of the activity
    offset - The offset in seconds from the current time

    # Returns
    A Result with the success or error of the operation
     */
    pub fn start_activity(&self, name: &str, offset: i64) -> Result<(), rusqlite::Error> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut start_time = now as i64 + offset;
        
        if let Some((_, current_start_time)) = self.currrent_activity()? {
            if start_time < current_start_time as i64 {
                start_time = current_start_time as i64;
            }
        }

        self.stop_activity(offset)?;

        self.0.execute(
            "INSERT INTO activities (name, start_time) VALUES (?, ?)",
            params![name, start_time],
        )?;
        Ok(())
    }

    /**
    Stop the current activity with the given offset.

    # Arguments
    offset - The offset in seconds from the current time

    # Returns
    A Result with the success or error of the operation
     */
    pub fn stop_activity(&self, offset: i64) -> Result<(), rusqlite::Error> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut end_time = now as i64 + offset;
        
        if let Some((_, start_time)) = self.currrent_activity()? {
            if end_time < start_time as i64 {
                end_time = start_time as i64;
            }
        }

        self.0.execute(
            "UPDATE activities SET end_time = ? WHERE end_time IS NULL",
            params![end_time],
        )?;
        Ok(())
    }

    /**
    Return the name of the current activity if there is one.

    # Returns
    The name of the current activity if there is one
     */
    pub fn currrent_activity(&self) -> Result<Option<(String, u64)>, rusqlite::Error> {
        let mut stmt = self.0.prepare(
            "SELECT name, start_time FROM activities WHERE end_time IS NULL ORDER BY start_time DESC LIMIT 1",
        )?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let name: String = row.get(0)?;
            let start_time: u64 = row.get(1)?;
            Ok(Some((name, start_time)))
        } else {
            Ok(None)
        }
    }

    /**
    List all the activities names. Even if they were cleared.
    
    # Returns
    A list of all the activities names
     */
    pub fn list_activities(&self) -> Result<Vec<String>, rusqlite::Error> {
        let mut stmt = self.0.prepare("SELECT DISTINCT name FROM activities")?;

        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut activities = Vec::new();

        for activity in rows {
            activities.push(activity?);
        }

        Ok(activities)
    }

    /**
    Return the total time of each activity.

    # Returns
    A HashMap with the name of the activity as the key and the total time in seconds as the value
     */
    pub fn activities_times(&self) -> Result<HashMap<String, u64>, rusqlite::Error> {
        let mut stmt = self.0.prepare(
            "SELECT name, start_time, end_time FROM activities WHERE start_time >= (SELECT time FROM clears ORDER BY time DESC LIMIT 1)",
        )?;
        
        let times = stmt.query_map([], |row| {
            let name: String = row.get(0)?;
            let start_time: u64 = row.get(1)?;
            let end_time: Option<u64> = row.get(2)?;
            Ok((name, start_time, end_time))
        })?;

        let mut activities = HashMap::new();

        for time in times {
            let (name, start_time, end_time) = time?;
            let duration = match end_time {
                Some(end_time) => end_time - start_time,
                None => SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    - start_time,
            };
            *activities.entry(name).or_insert(0) += duration;
        }

        Ok(activities)
        
    }

    /**
    Mark the current time as the last time the database was cleared. All the activities before this time are ignored when counting time 
    but they are still in the database and they contribute to the list of activity.

    # Returns
    A Result with the success or error of the operation
     */
    pub fn clear_activities(&self) -> Result<(), rusqlite::Error> {
        // Before clearing the activities, we need to stop the current activity
        self.stop_activity(0)?;
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.0.execute("INSERT INTO clears (time) VALUES (?)", params![now])?;
        Ok(())
    }

    /**
     * Remove all activities and clears from the database
     */
    pub fn hard_clear_activities(&self) -> Result<(), rusqlite::Error> {
        self.0.execute("DELETE FROM activities", [])?;
        self.0.execute("DELETE FROM clears", [])?;
        Ok(())
    }

    /// Get the activities for today
    pub fn todays_activities(&self) -> Result<Vec<(String, u64, Option<u64>)>, rusqlite::Error>{
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let today = now - now % 86400;
        let mut stmt = self.0.prepare(
            "SELECT name, start_time, end_time FROM activities WHERE start_time >= ?",
        )?;
        let times = stmt.query_map(params![today], |row| {
            let name: String = row.get(0)?;
            let start_time: u64 = row.get(1)?;
            let end_time: Option<u64> = row.get(2)?;
            Ok((name, start_time, end_time))
        })?;

        let mut activities = Vec::new();
        for time in times {
            activities.push(time?);
        }

        Ok(activities)
    }
}
