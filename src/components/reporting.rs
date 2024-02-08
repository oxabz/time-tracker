use closure::closure;
use leptos::{html::Input, *};
use log::info;
use web_sys::{KeyboardEvent, MouseEvent, SubmitEvent};

use crate::{invoke::{get_current_activity, list_activities, start_activity, stop_activity}, notifications::Messages};

pub fn offset_string(offset: i64) -> String {
    if offset == 0 {
        return "now".to_string();
    }

    let sign = offset > 0;
    let offset = offset.abs();

    let hours = offset / 3600;
    let minutes = (offset % 3600) / 60;

    let res = match (hours, minutes) {
        (0, 0) => "".to_string(),
        (0, minutes) => format!("{}m", minutes),
        (hours, 0) => format!("{}h", hours),
        (hours, minutes) => format!("{}h and {}m", hours, minutes),
    };

    if sign {
        format!("in {}", res)
    } else {
        format!("{} ago", res)
    }
}

#[component]
pub fn OffsetModal<ModalCb: Fn(bool) + Clone + 'static, OffsetCb: Fn(i64) + 'static>(open: ReadSignal<bool>, modal_cb: ModalCb, offset_cb: OffsetCb) -> impl IntoView{
    let (offset, set_offset) = create_signal(0i64);
    let element_ref = create_node_ref::<Input>();

    let close = move || {
        modal_cb(false);
    };

    create_effect({
        let element_ref = element_ref.clone();
        move |_| {
            if !open.get() {return;}
            info!("Focusing input");
            
            let res = element_ref.get().unwrap().focus();

            set_offset.set(0);

            if let Err(e) = res {
                log::error!("Failed to focus input: {:?}", e);
            }
        }
    });

    let handle_key = closure!(clone close, |ev: KeyboardEvent| {
        let shift = if ev.shift_key() { 10 * 60 } else { 30 * 60 };
        match ev.key().as_str() {
            "ArrowLeft" => {
                set_offset.update(|offset| *offset -= shift);
            },
            "ArrowRight" => {
                set_offset.update(|offset| *offset += shift);
            },
            "Escape" => {
                close();
            },
            "Enter" => {
                offset_cb(offset.get_untracked());
                close();
            },
            _ => {},
        }
    });

    view! {
        <dialog open=open class="modal" on:keydown=handle_key>
            <div  class="modal-box flex flex-col items-center" autofocus>
                <h3 class="text-lg w-full">Offset : </h3>
                <div class="h-24 w-64 flex justify-between items-center">
                    <kbd class="kbd h-fit">{"◀"}</kbd>
                    <input _ref=element_ref type="button" class="text-xl h-fit text-center w-48" value={move||{
                        let offset = offset.get();
                        offset_string(offset)
                    }} />
                    <kbd class="kbd h-fit">{"▶"}</kbd>
                </div>
                <div class="modal-action w-full flex justify-end">
                    <button on:click=move|_|close()>Cancel</button>
                </div>
            </div>
        </dialog>
    }
}

#[component]
pub fn Reporting()-> impl IntoView{
    let (activity_name, set_activity_name) = create_signal(String::new());
    let (activities, set_activities) = create_signal(Vec::new());
    let (offset_modal_open, set_offset_modal_open) = create_signal(false);
    // The action to perform when the form is submitted true for start, false for stop
    let (action, set_action) = create_signal(true);

    let messages = expect_context::<Messages>();


    let offset_submit = closure!(
        clone action,
        |offset| {
            let action = action.get_untracked();
            info!("Offset: {}, Action :{}", offset, action);


            if action {
                let activity = activity_name.get_untracked();
                spawn_local(async move {
                    let res = start_activity(&activity, offset).await;
                    match res {
                        Ok(_) => {
                            messages.success(format!("Started activity: {}", activity));
                        },
                        Err(_) => {
                            messages.error(format!("Failed to start activity"));
                        }
                    }
                });
            } else {
                let activity = activity_name.get_untracked();
                spawn_local(async move {
                    let res = stop_activity(offset).await;
                    match res {
                        Ok(_) => {
                            messages.success(format!("Stopped activity: {}", activity));
                        },
                        Err(_) => {
                            messages.error(format!("Failed to stop activity"));
                        }
                    }
                });
            }
        }
    );

    // Update the activity name when the input value changes
    let update_value = move |event| {
        set_activity_name.set(event_target_value(&event));
    };

    // Handle the form submission
    // Shows the offset modal when the form is submitted
    let start_activity = closure!(clone set_action, clone set_offset_modal_open, |ev:SubmitEvent|{
        info!("Starting activity");
        set_action.set(true);
        set_offset_modal_open.set(true);
        ev.prevent_default();
    });

    let stop_activity = closure!(clone set_action, clone set_offset_modal_open, |ev: MouseEvent|{
        info!("Stopping activity");
        set_action.set(false);
        set_offset_modal_open.set(true);
        ev.prevent_default();
    });


    spawn_local({
        let set_activity_name = set_activity_name.clone();
        async move {
            let activity = get_current_activity().await;
            set_activity_name.set(activity);
        }
    });

    spawn_local(
        async move {
            let activities = list_activities().await;
            let activities = match activities {
                Ok(activities) => activities,
                Err(_) => return,
            };
            set_activities.set(activities);
        }
    );


    view! {
        <>
        <form class="bg-base-200 p-6 flex items-center rounded-lg gap-4" id="reporting" on:submit=start_activity>
            <input id="activity-input" list="known-activity" class="input w-full" type="text" placeholder="Activity" on:change=update_value value=activity_name/>
            <datalist id="known-activity">
                {move ||activities.get().into_iter().map(|activity| view!{<option value=activity/>}).collect_view()}
            </datalist>
            <input type="submit" class="btn btn-primary" value="Start!" />
            <button class="btn btn-error" on:click=stop_activity>{"Stop!"}</button>
        </form>
        <OffsetModal 
            open=offset_modal_open
            modal_cb={move|v|{
                
                set_offset_modal_open.set(v);
            }}
            offset_cb=offset_submit
            />
        </>
    }

}