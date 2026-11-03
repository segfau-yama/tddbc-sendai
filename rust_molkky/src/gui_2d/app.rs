use super::{
    controller::{Controller, DomEvent},
    models::{GamePhase, GuiField},
    services::GuiMolkky,
    view,
};
use crate::core::{Molkky, Name, SkittleState, Team};
use dioxus::prelude::*;

#[cfg(test)]
#[path = "../../tests/gui_2d/app.rs"]
mod tests;

fn render(controller: &Controller) -> Element {
    let game = controller.game();
    let rules = controller.rules();
    let phase = match game.phase {
        GamePhase::Ready => "ready",
        GamePhase::Charging => "charging",
        GamePhase::Simulating => "simulating",
        GamePhase::Finished => "finished",
    };
    let status = match game.phase {
        GamePhase::Ready => "左ボタンを押して照準・蓄力を開始します。",
        GamePhase::Charging => "マウスで狙い、左ボタンを離すと投擲します。",
        GamePhase::Simulating => "投擲中…",
        GamePhase::Finished => "試合終了",
    };
    let (position, angle) = if game.phase == GamePhase::Simulating {
        rules.pose(game.stick.stick().id()).unwrap()
    } else {
        (GuiField::THROW_POSITION, 0.0)
    };
    let fallen = rules.hit(&game.stick, game.field.skittles());
    let targets = if game.phase == GamePhase::Charging {
        rules.aim(&game.stick, game.field.skittles())
    } else {
        Vec::new()
    };
    let skittles = game.field.skittles().iter().map(|skittle| {
        let pos = rules.pose(skittle.id()).unwrap().0;
        let skittle = if matches!(game.phase, GamePhase::Simulating | GamePhase::Finished)
            && fallen
                .iter()
                .any(|fallen_skittle| fallen_skittle.id() == skittle.id())
        {
            skittle.with_state(SkittleState::Fall)
        } else {
            *skittle
        };
        let id = skittle.id();
        let class = if targets.iter().any(|target| target.id() == id) {
            "aimed"
        } else {
            ""
        };
        rsx! { g { key: "{id}", class, {view::skittle(&skittle, pos, GuiMolkky::SKITTLE_RADIUS)} } }
    });
    let scene = rsx! {
        {skittles}
        {view::aim(GuiField::THROW_POSITION, game.stick.target())}
        {view::stick(game.stick.stick().id(), position, angle, GuiMolkky::STICK_LENGTH, 2.0 * GuiMolkky::STICK_RADIUS)}
    };
    rsx! {
        style { {include_str!("view/style.css")} }
        h1 { "モルック" }
        p { role: "status", "{status}" }
        div { class: "game-layout",
            div { id: "molkky-input", "data-phase": phase,
                {view::field(GuiField::BOUNDS, scene)}
                {view::speedometer(game.stick.speed(), Controller::MAX_SPEED)}
            }
            div { class: "scores",
                {view::scores(&game.teams)}
                if game.phase == GamePhase::Finished { {view::result(rules.judge_game(&game.teams))} }
            }
        }
    }
}

#[component]
pub fn App() -> Element {
    let mut controller = use_signal(|| {
        Controller::new(vec![
            Team::new(Name::new("赤チーム")),
            Team::new(Name::new("青チーム")),
        ])
    });
    let mut bridge = use_signal(|| None::<document::Eval>);
    let mut error = use_signal(|| false);
    use_drop(move || {
        if let Some(eval) = *bridge.peek() {
            let _ = eval.send(());
        }
    });
    let content = render(&controller.read());
    rsx! {
        main {
            onmounted: move |_| {
                let mut eval = document::eval(include_str!("controller.js"));
                bridge.set(Some(eval));
                spawn(async move {
                    while let Ok(event) = eval.recv::<DomEvent>().await { controller.write().handle(event); }
                    controller.write().handle(DomEvent::Blur);
                    error.set(true);
                });
            },
            document::Title { "モルック" }
            {content}
            if error() { p { role: "alert", "入力の接続が停止しました。画面を再読み込みしてください。" } }
        }
    }
}
