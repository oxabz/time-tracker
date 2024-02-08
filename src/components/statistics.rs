use leptos::*;
use web_time::Duration;

use crate::invoke::get_activities_time;


#[component]
fn ProportionBar(
    activity: String,
    time: String,
    proportion: f64,
) -> impl IntoView {
    let style = format!("width: {percent}%; background-color:color-mix(in srgb, #CC8899 {percent}%, #00AAFF); color:#111111;", percent = proportion * 100.0);
    view! {
        <li class="w-full p-2">
            <div class="relative w-full h-full">
                <p>{"\u{00A0}"}</p> // NBSP to make the div have a height
                <p class="absolute w-full top-0 left-0 p-2">{time.clone()}{" ● "}{activity.clone()}</p>
                <p class="absolute top-0 left-0 overflow-hidden rounded text-nowrap p-2" style=style>{time}{" ● "}{activity}</p>
            </div>
        </li> 
    }
}

#[component]
pub fn Statistics() -> impl IntoView {
    let (activities, set_activities) = create_signal(Vec::new());

    let update_statistics = move || {
        let set_activities = set_activities.clone();
        async move {
            let activities = get_activities_time().await;
            let mut activities = match activities {
                Ok(activities) => activities,
                Err(_) => return,
            };

            activities.sort_by(|a, b| b.1.cmp(&a.1));

            set_activities.set(activities);
        }
    };

    spawn_local(update_statistics());

    
    set_interval(move ||{
        spawn_local(update_statistics());
    }, Duration::from_secs(1));
    
    let render_activities = move || {
        let activities = activities.get();

        let max_time: u64 = *activities.iter().map(|(_, time)| time).max().unwrap_or(&1);
        
        
        activities.into_iter().map(|(activity, time) : (String, u64)| {
            let proportion = time as f64 / max_time as f64;


            let hours = time / 3600;
            let minutes = (time % 3600) / 60;

            view! {
                <ProportionBar activity=activity time=format!("{hours:2}h{minutes:2}") proportion=proportion/>
            }
        }).collect_view()
    };
    
    view! {
        <ul class="w-full h-full bg-base-200 rounded-lg flex flex-col p-4 gap-2 overflow-y-scroll">
            {render_activities}
        </ul>
    }
}