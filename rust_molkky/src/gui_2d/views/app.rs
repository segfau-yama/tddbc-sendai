use super::{
    stick::{Phase, StickInput, phase},
    *,
};
use crate::gui_2d::{
    models::GuiField,
    services::{Game, field::create_field, teams::from_names},
};
use dioxus::prelude::*;

#[cfg(test)]
#[path = "../../../tests/gui_2d/views/app.rs"]
mod tests;

pub(super) fn render(game: Signal<Game>) -> Element {
    let input_phase = phase(&game.read());
    let shared_game = game;
    let game = game.read();
    let stick = game.stick();
    let finished = game.is_finished();
    let (phase, status) = if finished {
        ("finished", "試合終了")
    } else {
        match input_phase {
            Phase::Ready => ("ready", "左ボタンを押して照準・蓄力を開始します。"),
            Phase::Charging => ("charging", "マウスで狙い、左ボタンを離すと投擲します。"),
            Phase::Simulating => ("simulating", "投擲中…"),
        }
    };
    let aiming = input_phase == Phase::Charging;
    let skittles = game.skittles().iter().map(|skittle| {
        let id = skittle.as_ref().id();
        let class = if aiming && skittle.aimed() {
            "aimed"
        } else {
            ""
        };
        rsx! { g { key: "{id}", class, {super::skittle(skittle)} } }
    });
    let scene = rsx! {
        {skittles}
        Stick { game: shared_game }
    };
    rsx! {
        style { {include_str!("style.css")} }
        h1 { "モルック" }
        p { role: "status", "{status}" }
        div { class: "game-layout",
            div { id: "molkky-input", "data-phase": phase,
                {field(GuiField::BOUNDS, scene)}
                {speedometer(stick.speed(), StickInput::MAX_SPEED)}
            }
            div { class: "scores",
                {scores(game.teams())}
                if finished { {result(game.winner())} }
            }
        }
    }
}

#[component]
pub fn App() -> Element {
    let game = use_signal(|| {
        let (field, stick, skittles) = create_field();
        Game::new(
            from_names(&["赤チーム", "青チーム"]),
            field,
            stick,
            skittles,
        )
    });
    rsx! {
        main {
            document::Title { "モルック" }
            {render(game)}
        }
    }
}
