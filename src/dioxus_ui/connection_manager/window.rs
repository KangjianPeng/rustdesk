use super::super::prelude::*;
use super::backend::close_manager;
use super::panel::Panel;

#[component]
pub(crate) fn Window() -> Element {
    rsx! {
        UiStyles {}
        div { class: "flex h-screen min-h-[480px] w-full flex-col overflow-hidden bg-rd-window",
            header { class: "flex h-8 items-center border-b border-[#404040] bg-rd-surface px-[5px] select-none",
                div { class: "flex h-8 min-w-0 flex-1 items-center cursor-grab", onmousedown: move |_| {
                    #[cfg(not(target_os = "android"))]
                    crate::dioxus_ui::native::window().drag();
                }, "RustDesk" }
                if cfg!(not(target_os = "android")) {
                    Button { css_class: "h-8 w-8 bg-transparent p-0 text-[#bfc0c2] hover:bg-[#c42b1c] hover:text-white [&>button]:h-full [&>button]:w-full", title: "Close", variant: ButtonVariant::Ghost, size: ButtonSize::IconSm, onclick: move |_| {
                        #[cfg(not(target_os = "android"))]
                        spawn(close_manager(true));
                    }, "×" }
                }
            }
            Panel { standalone: true }
        }
    }
}

#[component]
fn UiStyles() -> Element {
    rsx! { document::Stylesheet { href: asset!("/assets/tailwind.css") } }
}
