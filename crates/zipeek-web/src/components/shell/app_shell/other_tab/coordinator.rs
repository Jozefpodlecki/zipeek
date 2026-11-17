use std::{cell::RefCell, rc::Rc};

use gloo::timers::callback::Interval;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{BroadcastChannel, MessageEvent};
use yew::UseStateHandle;

use super::{identity::TabId, protocol::TabMessage, registry::PeerRegistry};

const CHANNEL: &str = "zipeek-active-tabs";
const HEARTBEAT_MS: u32 = 2_000;

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

fn broadcast(channel: &BroadcastChannel, message: TabMessage) -> Result<(), JsValue> {
    channel.post_message(&JsValue::from_str(&message.encode()))
}

pub(super) struct TabCoordinator {
    channel: BroadcastChannel,
    id: TabId,
    _listener: Closure<dyn FnMut(MessageEvent)>,
    _heartbeat: Interval,
}

impl TabCoordinator {
    pub(super) fn start(other_open: UseStateHandle<bool>) -> Result<Self, JsValue> {
        let channel = BroadcastChannel::new(CHANNEL)?;
        let id = TabId::new();
        let peers = Rc::new(RefCell::new(PeerRegistry::default()));
        let listener_peers = Rc::clone(&peers);
        let listener_channel = channel.clone();
        let listener_state = other_open.clone();
        let listener = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
            let Some(message) = event
                .data()
                .as_string()
                .and_then(|value| TabMessage::decode(&value))
            else {
                return;
            };
            if message.sender() == id {
                return;
            }
            let mut peers = listener_peers.borrow_mut();
            let now = now_ms();
            peers.expire(now);
            peers.observe(message, now);
            listener_state.set(peers.has_peers());
            if matches!(message, TabMessage::Hello(_)) {
                let _ = broadcast(&listener_channel, TabMessage::Alive(id));
            }
        });
        channel.set_onmessage(Some(listener.as_ref().unchecked_ref()));

        let heartbeat_channel = channel.clone();
        let heartbeat_peers = peers;
        let heartbeat_state = other_open;
        let heartbeat = Interval::new(HEARTBEAT_MS, move || {
            let mut peers = heartbeat_peers.borrow_mut();
            peers.expire(now_ms());
            heartbeat_state.set(peers.has_peers());
            let _ = broadcast(&heartbeat_channel, TabMessage::Alive(id));
        });
        let _ = broadcast(&channel, TabMessage::Hello(id));
        Ok(Self {
            channel,
            id,
            _listener: listener,
            _heartbeat: heartbeat,
        })
    }
}

impl Drop for TabCoordinator {
    fn drop(&mut self) {
        self.channel.set_onmessage(None);
        let _ = broadcast(&self.channel, TabMessage::Goodbye(self.id));
        self.channel.close();
    }
}
