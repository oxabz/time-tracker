use std::time::Duration;
use web_time::SystemTime;

use leptos::*;

use crate::components::pure_html::icons;

#[derive(Clone, Debug)]
pub enum Message {
    Success(String),
    Error(String),
}


const MESSAGE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Copy, Clone, Debug, Default)]
pub struct Messages {
    messages: RwSignal<Vec<(Message, SystemTime)>>,
}

impl Messages {
    pub fn success(&self, message: String) {
        self.messages.update(|messages| {
            messages.push((Message::Success(message), SystemTime::now()));
        });
    }

    pub fn error(&self, message: String) {
        self.messages.update(|messages| {
            messages.push((Message::Error(message), SystemTime::now()));
        });
    }

    pub fn remove_old_messages(&self) {
        self.messages.update(|messages| {
            let now = SystemTime::now();
            messages.retain(|(_, time)| now.duration_since(*time).unwrap_or_default() < MESSAGE_TIMEOUT);
        });
    }

    pub fn get_messages(&self) -> Vec<(Message, SystemTime)> {
        self.messages.get().clone()
    }
}



#[component]
/// A component that :
/// - Provides a context for the Messages
/// - Removes old messages every seconds
/// - Renders the messages
pub fn Notifications() -> impl IntoView {
    provide_context(Messages::default());

    let messages = expect_context::<Messages>();

    // Every seconds, remove old messages
    set_interval(
        {
            let messages = messages.clone();
            move || {
                messages.remove_old_messages();
            }
        }, 
        Duration::from_secs(1)
    );

    // Render the messages
    let messages_view = move ||{
        let messages = messages.get_messages();
        messages.into_iter().map(|(message, _)|{
            match message {
                Message::Success(message) => {
                    view!{
                        <div role="alert" class="alert alert-success">
                            {icons::success()}
                        <span>{message}</span>
                        </div>
                    }
                },
                Message::Error(message) => {
                    view!{
                        <div role="alert" class="alert alert-error">
                            {icons::error()}
                        <span>{message}</span>
                        </div>
                    }
                },
                
            }
        }).collect_view()
    };


    view! {
        <div class="fixed bottom-4 right-4">
            {messages_view}
        </div>
    }
}