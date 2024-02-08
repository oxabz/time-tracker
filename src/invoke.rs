use log::{debug, error};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;


#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

pub async fn get_current_activity() -> String {
    let res = invoke("get_current_activity", to_value(&()).expect("Serde should deserialize ()")).await;

    match res {
        Ok(val) => val.as_string().unwrap(),
        Err(e) => {
            debug!("get_current_activity error: {:?}", e);
            "".to_string()
        }
    }
}

#[derive(serde::Serialize)]
struct StartActivityArgs {
    activity: String,
    offset: i64,
}

pub async fn start_activity(activity: &str, offset: i64) -> Result<(), ()> {
    let args = StartActivityArgs {
        activity: activity.to_string(),
        offset,
    };
    let res = invoke("start_activity", to_value(&args).expect("Serde should deserialize (&str, i64)")).await;

    if let Err(err) = res {
        error!("start_activity error: {:?}", err);
        return Err(());
    }

    Ok(())
}

#[derive(serde::Serialize)]
struct StopActivityArgs {
    offset: i64,
}

pub async fn stop_activity(offset: i64) -> Result<(), ()>  {
    let args = StopActivityArgs {
        offset,
    };
    let res = invoke("stop_activity", to_value(&args).expect("Serde should deserialize i64")).await;

    if let Err(err) = res {
        error!("stop_activity error: {:?}", err);
        return Err(());
    }

    Ok(())
}

pub async fn get_activities_time() -> Result<Vec<(String, u64)>, ()> {
    let res = invoke("get_activities_times", to_value(&()).expect("Serde should deserialize ()")).await;

    match res {
        Ok(val) => serde_wasm_bindgen::from_value(val).map_err(|e| {
            error!("get_activities_time error: {:?}", e);
            ()
        }),
        Err(e) => {
            error!("get_activities_time error: {:?}", e);
            Err(())
        }
    }
}

pub async fn list_activities() -> Result<Vec<String>, ()> {
    let res = invoke("list_activities", to_value(&()).expect("Serde should deserialize ()")).await;

    match res {
        Ok(val) => serde_wasm_bindgen::from_value(val).map_err(|e| {
            error!("list_activities error: {:?}", e);
            ()
        }),
        Err(e) => {
            error!("list_activities error: {:?}", e);
            Err(())
        }
    }
}

pub async fn clear_activities() -> Result<(), String> {
    let res = invoke("clear_activities", to_value(&()).expect("Serde should deserialize ()")).await;

    if let Err(err) = res {
        error!("clear_activities error: {:?}", err);

        if err.is_string() {
            return Err(err.as_string().unwrap());
        } else {
            return Err(format!("{:?}", err));
        }
    }

    Ok(())
}

pub async fn hard_clear_activities() -> Result<(), String> {
    let res = invoke("hard_clear_activities", to_value(&()).expect("Serde should deserialize ()")).await;

    if let Err(err) = res {
        error!("hard_clear_activities error: {:?}", err);

        if err.is_string() {
            return Err(err.as_string().unwrap());
        } else {
            return Err(format!("{:?}", err));
        }
    }

    Ok(())
}

pub async fn todays_activities() -> Result<Vec<(String, u64, Option<u64>)>, ()> {
    let res = invoke("todays_activities", to_value(&()).expect("Serde should serialize ()")).await;

    match res {
        Ok(val) => serde_wasm_bindgen::from_value(val).map_err(|e| {
            error!("todays_activities error: {:?}", e);
            ()
        }),
        Err(e) => {
            error!("todays_activities error: {:?}", e);
            Err(())
        }
    }
}

pub async fn export_activities()-> Result<(), String>{
    let res = invoke("export_activities", to_value(&()).expect("Serde should serialize ()")).await;

    if let Err(err) = res {
        error!("export_activities error: {:?}", err);
        return Err(format!("{:?}", err));
    }

    Ok(())
}
