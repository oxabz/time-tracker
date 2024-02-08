use closure::closure;
use leptos::*;

use crate::{invoke::{clear_activities, hard_clear_activities}, notifications::Messages};

#[component]
pub fn Actions() -> impl IntoView{
    let message = expect_context::<Messages>();
    let (clear_dialog, set_clear_dialog) = create_signal(false);

    let export = move |_| {
        log::info!("Exporting data");
        
        spawn_local(async move {
            let res = crate::invoke::export_activities().await;

            match res {
                Ok(_) => {
                    message.success("Data exported".to_string());
                },
                Err(err) => {
                    message.error(format!("Failed to export data: {}", err));
                },
            }
        });
    };

    let open_clear_dialog = closure!(clone set_clear_dialog, |_| {
        set_clear_dialog.set(true);
    });

    let close_clear_dialog = closure!(clone set_clear_dialog, |_| {
        set_clear_dialog.set(false);
    });

    let clear = closure!(clone set_clear_dialog, |_| {
        log::info!("Clearing data");

        // Close the dialog
        set_clear_dialog.set(false);

        // Clear the data
        spawn_local(async move {
            let res = clear_activities().await;

            match res {
                Ok(_) => {
                    message.success("Data cleared".to_string());
                },
                Err(err) => {
                    message.error(format!("Failed to clear data: {}", err));
                },
            }

        });
    
    });

    let hard_clear = closure!(clone set_clear_dialog, |_| {
        log::info!("Hard clearing data");

        // Close the dialog
        set_clear_dialog.set(false);

        // Clear the data
        spawn_local(async move {
            let res = hard_clear_activities().await;

            match res {
                Ok(_) => {
                    message.success("Data cleared".to_string());
                },
                Err(err) => {
                    message.error(format!("Failed to clear data: {}", err));
                },
            }

        });

    });

    view! {
        <div class="bg-base-200 p-6 flex items-center rounded-lg justify-end gap-4" id="actions">
            <button class="btn btn-accent" on:click=export>{"Export"}</button>
            <button class="btn btn-error" on:click=open_clear_dialog>{"Clear"}</button>
            <dialog open=clear_dialog  class="modal">
                <div  class="modal-box">
                    <h3 class="font-bold text-lg">Do you want to clear ?</h3>
                    <ul>
                        <li>No -> Cancel</li>
                        <li>Mark as finished -> Clear</li>
                        <li>Irreparably delete all data-> Hard Clear (Double click)</li>
                    </ul>
                    <div class="modal-action">
                        <button class="btn btn-primary" on:click=close_clear_dialog>No</button>
                        <button class="btn btn-warning" on:click=clear>Clear</button>
                        <button class="btn btn-error" on:dblclick=hard_clear>Hard Clear</button>
                    </div>
                </div>
            </dialog>
        </div>
    }
}