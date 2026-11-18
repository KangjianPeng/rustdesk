use super::super::prelude::*;
use super::backend::close_manager;
use super::panel::Panel;

#[component]
pub(crate) fn Window() -> Element {
    rsx! {
        div { class: "flex h-screen min-h-[480px] w-full flex-col overflow-hidden bg-background",
            header { class: "flex h-8 items-center border-b border-border bg-card px-[5px] select-none",
                div { class: "flex h-8 min-w-0 flex-1 items-center cursor-grab", onmousedown: move |_| {
                    #[cfg(not(target_os = "android"))]
                    dioxus::desktop::window().drag();
                }, "RustDesk" }
                if cfg!(not(target_os = "android")) {
                    Button { css_class: "h-8 w-8 bg-transparent p-0 text-muted-foreground hover:bg-destructive hover:text-white", title: "Close", variant: ButtonVariant::Ghost, size: ButtonSize::IconSm, onclick: move |_| {
                        #[cfg(not(target_os = "android"))]
                        spawn(close_manager(true));
                    }, "×" }
                }
            }
            Panel { standalone: true }
        }
    }
}
