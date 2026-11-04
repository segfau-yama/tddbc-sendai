use super::{
    controller::{Controller, DomEvent, InputPhase},
    models::GuiField,
    services::GuiMolkky,
    view,
};
use crate::core::{Molkky, Name, SkittleState, Team};
use dioxus::prelude::*;

#[cfg(test)]
#[path = "../../tests/gui_2d/app.rs"]
mod tests;

fn render(controller: &Controller) -> Element {
    let teams = controller.teams();
    let field = controller.field();
    let stick = controller.stick();
    let finished = controller.is_finished();
    let input_phase = controller.phase();
    let rules = controller.rules();
    let (phase, status) = if finished {
        ("finished", "試合終了")
    } else {
        match input_phase {
            InputPhase::Ready => ("ready", "左ボタンを押して照準・蓄力を開始します。"),
            InputPhase::Charging => ("charging", "マウスで狙い、左ボタンを離すと投擲します。"),
            InputPhase::Simulating => ("simulating", "投擲中…"),
        }
    };
    let (position, angle) = if input_phase == InputPhase::Simulating {
        rules.pose(stick.stick().id()).unwrap()
    } else {
        (GuiField::THROW_POSITION, 0.0)
    };
    let fallen = rules.hit(stick, field.skittles());
    let targets = if input_phase == InputPhase::Charging {
        rules.aim(stick, field.skittles())
    } else {
        Vec::new()
    };
    let skittles = field.skittles().iter().map(|skittle| {
        let pos = rules.pose(skittle.id()).unwrap().0;
        let skittle = if (input_phase == InputPhase::Simulating || finished)
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
        {view::aim(GuiField::THROW_POSITION, stick.target())}
        {view::stick(stick.stick().id(), position, angle, GuiMolkky::STICK_LENGTH, 2.0 * GuiMolkky::STICK_RADIUS)}
    };
    rsx! {
        style { {include_str!("view/style.css")} }
        h1 { "モルック" }
        p { role: "status", "{status}" }
        div { class: "game-layout",
            div { id: "molkky-input", "data-phase": phase,
                {view::field(GuiField::BOUNDS, scene)}
                {view::speedometer(stick.speed(), Controller::MAX_SPEED)}
            }
            div { class: "scores",
                {view::scores(teams)}
                if finished { {view::result(rules.judge_game(teams))} }
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
