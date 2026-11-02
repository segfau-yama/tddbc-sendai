use dioxus::prelude::*;

/// boundsはworldの[x, y, 幅, 高さ]。childrenもworld座標で受け取る。
pub fn field(bounds: [f32; 4], children: Element) -> Element {
    let [x, y, width, height] = bounds;
    let view_box = format!("{x} {} {width} {height}", -(y + height));
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            view_box,
            role: "img",
            "aria-label": "モルックのフィールド",
            style: "display:block;width:100%;background:#e6efdc;border:1px solid #61734e",
            g { transform: "scale(1 -1)", {children} }
        }
    }
}
