use leptos::*;
use web_time::{Duration, SystemTime};

use crate::{invoke::todays_activities, notifications::Messages};

/// The start hour of the timeline
const START_HOUR: u32 = 8;
/// The end hour of the timeline (the timeline finishes at END_HOUR exaclty, it does not include it)
const END_HOUR: u32 = 19;


pub fn hour_mark(hour: u32) -> impl IntoView{
    let left = ((hour - START_HOUR) as f64 / (END_HOUR - START_HOUR) as f64 * 100.0) as u32;
    view! {
        <div class="absolute top-0 bg-base-content h-12" style=format!("left: {left}%; width: 1px;", left=left)></div>
    }
}

pub fn hour_label(hour: u32) -> impl IntoView{
    let left = ((hour - START_HOUR) as f64 / (END_HOUR - START_HOUR) as f64 * 100.0) as u32;
    let text = format!("{:02}:00", hour);
    view! {
        <p class="absolute top-0 text-sm text-center" style=format!("left: calc({left}% - 1.25rem); width: 2.5rem;", left=left)>
            {text}
        </p>
    }
}

pub fn render_activity((activity, start, end):(String, u64, Option<u64>)) -> impl IntoView{
    let end = match end {
        Some(end) => end,
        None => {
            let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
            now
        }
    };

    let day_start = start - start % 86400;
    let timeline_start = day_start + START_HOUR as u64 * 3600;
    let timeline_end = day_start + END_HOUR as u64 * 3600;
    let timeline_duration = timeline_end - timeline_start;

    let left = (start - timeline_start as u64) as f32 / timeline_duration as f32 * 100.0;
    let width = (end - start) as f32 / timeline_duration as f32 * 100.0;

    let mut hue: u32 = 0;
    for byte in activity.as_bytes() {
        let inc = *byte as u32 * 360 / 255;
        hue += inc;
    }
    hue %= 360;

    let style = format!("left: {left:.2}%; width: {width:.2}%; background-color: hsl({hue}, 70%, 70%);", left=left, width=width, hue=hue);

    view! {
        <div title={activity.clone()} class="absolute top-0 h-12 p-2 radius rounded-md text-primary-content" style=style>
            <p class="text-xs truncate">{activity}</p>
        </div>
    }
}

fn now_line() -> impl IntoView{
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    let day_start = now - now % 86400;
    let timeline_start = day_start + START_HOUR as u64 * 3600;
    let timeline_end = day_start + END_HOUR as u64 * 3600;
    let timeline_duration = timeline_end - timeline_start;

    let left = (now - timeline_start as u64) as f32 / timeline_duration as f32 * 100.0;

    view! {
        <div class="absolute top-0 bg-primary h-12" style=format!("left: {left:2}%; width: 3px;", left=left)></div>
    }
}

#[component]
pub fn Timeline() -> impl IntoView{
    let (activities, set_activities) = create_signal(Vec::new());
    let messages = expect_context::<Messages>();

    let update_activities = move || {
        let set_activities = set_activities.clone();
        async move {
            let activities = todays_activities().await;
            match activities {
                Ok(activities) => {
                    set_activities.set(activities);
                },
                Err(_) => {
                    messages.error("Failed to fetch activities".to_string());
                }
            }
        }
    };

    spawn_local(update_activities());

    set_interval(move ||{
        spawn_local(update_activities());
    }, Duration::from_secs(10));
    

    view! {
        <div class="w-full flex flex-col px-5">
            <div class="w-full h-12 relative overflow-hidden">
                // Timeline lines
                {(START_HOUR..=END_HOUR).into_iter().map(hour_mark).collect_view()}
                // Activities & now line
                {
                    move || {
                        let activities = activities.get();
                        view! {
                            <>
                                {activities.into_iter().map(render_activity).collect_view()}
                                {now_line()} // Now line just tags along for the update
                            </>
                        }
                    }
                }
            </div>
            // Hours labels
            <p class="w-full relative text-sm">
                // Non breaking space for the hours labels to align with the lines
                {"\u{00A0}"}
                // Hours labels
                {(START_HOUR..=END_HOUR).into_iter().map(hour_label).collect_view()}
            </p>
        </div>
    }
}