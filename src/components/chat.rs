use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        html! {
            <div class="flex w-screen h-screen bg-gradient-to-br from-blue-50 to-purple-100">
                // Sidebar: User List
                <aside class="flex-none w-64 h-full bg-white shadow-lg border-r border-gray-200 flex flex-col">
                    <div class="text-2xl font-bold p-6 border-b border-gray-200 text-blue-700 flex items-center gap-2">
                        <svg class="w-6 h-6 text-blue-500" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M17 20h5v-2a4 4 0 00-3-3.87M9 20H4v-2a4 4 0 013-3.87m9-4a4 4 0 11-8 0 4 4 0 018 0zm6 4v2a2 2 0 01-2 2h-4a2 2 0 01-2-2v-2a2 2 0 012-2h4a2 2 0 012 2z"></path>
                        </svg>
                        {"Users"}
                    </div>
                    <div class="overflow-y-auto flex-1 p-4 space-y-3">
                        {
                            self.users.iter().map(|u| {
                                html!{
                                    <div class="flex items-center bg-blue-50 hover:bg-blue-100 rounded-lg p-3 shadow-sm transition">
                                        <img class="w-12 h-12 rounded-full border-2 border-blue-200" src={u.avatar.clone()} alt="avatar"/>
                                        <div class="ml-4">
                                            <div class="font-semibold text-blue-800">{u.name.clone()}</div>
                                            <div class="text-xs text-gray-400">{"Hi there!"}</div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                </aside>
                // Main Chat Area
                <main class="flex-1 flex flex-col h-full">
                    // Header
                    <header class="w-full h-16 bg-white border-b border-gray-200 flex items-center px-8 shadow-sm">
                        <div class="text-2xl font-bold text-purple-700 flex items-center gap-2">
                            <span>{"💬 Chat!"}</span>
                        </div>
                    </header>
                    // Messages
                    <section class="flex-1 overflow-y-auto px-8 py-6 space-y-4 bg-gradient-to-br from-white to-purple-50">
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from);
                                let avatar = user.map(|u| u.avatar.clone()).unwrap_or_default();
                                html!{
                                    <div class="flex items-end max-w-xl bg-white/80 shadow rounded-2xl p-4 gap-4">
                                        <img class="w-10 h-10 rounded-full border border-purple-200" src={avatar} alt="avatar"/>
                                        <div>
                                            <div class="font-semibold text-purple-700 text-sm">{m.from.clone()}</div>
                                            <div class="mt-1 text-gray-700 text-base break-words">
                                                {
                                                    if m.message.ends_with(".gif") {
                                                        html! { <img class="mt-2 rounded-lg max-h-40" src={m.message.clone()}/> }
                                                    } else {
                                                        html! { <span>{m.message.clone()}</span> }
                                                    }
                                                }
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </section>
                    // Input
                    <footer class="w-full h-20 bg-white border-t border-gray-200 flex items-center px-8">
                        <input
                            ref={self.chat_input.clone()}
                            type="text"
                            placeholder="Type your message..."
                            class="flex-1 py-3 px-5 bg-blue-50 rounded-full border border-blue-200 focus:outline-none focus:ring-2 focus:ring-blue-400 text-gray-700 shadow-sm transition"
                            name="message"
                            required=true
                            onkeypress={ctx.link().batch_callback(|e: KeyboardEvent| {
                                if e.key() == "Enter" { Some(Msg::SubmitMessage) } else { None }
                            })}
                        />
                        <button
                            onclick={submit}
                            class="ml-4 p-3 bg-gradient-to-br from-blue-500 to-purple-500 hover:from-blue-600 hover:to-purple-600 text-white rounded-full shadow-lg flex items-center justify-center transition"
                            title="Send"
                        >
                            <svg fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24" class="w-6 h-6">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M3 10l9-6 9 6-9 6-9-6zm0 0v6a9 9 0 009 9 9 9 0 009-9v-6"></path>
                            </svg>
                        </button>
                    </footer>
                </main>
            </div>
        }
    }
}