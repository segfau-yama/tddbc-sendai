use super::{
    controllers::{Controller, DomEvent, render},
    services::teams::from_names,
};
use dioxus::prelude::*;

#[component]
pub fn App() -> Element {
    let mut state = use_signal(|| Controller::new(from_names(&["赤チーム", "青チーム"])));
    let mut bridge = use_signal(|| None::<document::Eval>);
    let mut error = use_signal(|| false);
    use_drop(move || {
        if let Some(eval) = *bridge.peek() {
            let _ = eval.send(());
        }
    });
    let content = render(&state.read());
    rsx! {
        main {
            onmounted: move |_| {
                let mut eval = document::eval(include_str!("app.js"));
                bridge.set(Some(eval));
                spawn(async move {
                    while let Ok(event) = eval.recv::<DomEvent>().await { state.write().handle(event); }
                    state.write().handle(DomEvent::Blur);
                    error.set(true);
                });
            },
            document::Title { "モルック" }
            {content}
            if error() { p { role: "alert", "入力の接続が停止しました。画面を再読み込みしてください。" } }
        }
    }
}
