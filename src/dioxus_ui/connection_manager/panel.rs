use super::super::prelude::*;
use super::backend::{ManagerState, STATE};
use super::client_card::ClientCard;

#[component]
pub(crate) fn Panel(#[props(default)] standalone: bool) -> Element {
    let mut state = use_signal(ManagerState::default);
    let mut deploy_token = use_signal(String::new);
    let mut deploying = use_signal(|| false);
    let mut deploy_error = use_signal(String::new);
    use_future(move || async move {
        let mut rx = STATE.subscribe();
        loop {
            state.set(rx.borrow_and_update().clone());
            if rx.changed().await.is_err() {
                break;
            }
        }
    });
    let snapshot = state.read().clone();
    if !standalone && snapshot.clients.is_empty() && !snapshot.needs_deploy {
        return rsx! {};
    }
    rsx! {
        details { class: "max-h-[calc(100vh-64px)] overflow-y-auto bg-background p-5 text-foreground [&_summary]:mb-4 [&_summary]:cursor-pointer [&_summary]:text-xl [&_h2]:mb-4 [&_h2]:text-xl [&_h3]:m-0 [&_h3]:text-base [&_>summary]:mb-4", open: true,
            summary { "Incoming connections" }
            if snapshot.needs_deploy {
                p { "This device requires deployment before it can accept connections." }
                form { class: "mt-3 flex gap-2 [&>.inline-flex:first-child]:min-w-0 [&>.inline-flex:first-child]:flex-1 [&_input]:rounded-[5px] [&_input]:border [&_input]:p-2 [&>.inline-flex:last-child]:min-h-9 [&>.inline-flex:last-child]:rounded-[5px] [&>.inline-flex:last-child]:border [&>.inline-flex:last-child>button]:min-h-9 [&>.inline-flex:last-child>button]:px-[14px] [&>.inline-flex:last-child>button]:py-2", onsubmit: move |event| {
                    event.prevent_default();
                    if deploying() || deploy_token().trim().is_empty() { return; }
                    let token = deploy_token().trim().to_owned();
                    deploying.set(true);
                    deploy_error.set(String::new());
                    spawn(async move {
                        let result = hbb_common::tokio::task::spawn_blocking(move || crate::ui_interface::deploy_device(token, None)).await;
                        match result {
                            Ok(crate::ui_interface::DeployResult::Ok) => deploy_token.set(String::new()),
                            Ok(result) => deploy_error.set(result.message()),
                            Err(error) => deploy_error.set(format!("Deployment failed: {error}")),
                        }
                        deploying.set(false);
                    });
                },
                    Input { r#type: InputType::Password, value: deploy_token(), placeholder: "API token", disabled: deploying(), oninput: move |event: FormEvent| deploy_token.set(event.value()) }
                    Button { button_type: "submit", disabled: deploying() || deploy_token().trim().is_empty(), "Deploy" }
                }
                if !deploy_error().is_empty() { p { role: "alert", "{deploy_error}" } }
            }
            if standalone && snapshot.clients.is_empty() && !snapshot.needs_deploy { p { "Waiting for a connection…" } }
            if !snapshot.notice.is_empty() { p { role: "status", "{snapshot.notice}" } }
            for client in snapshot.clients.values() {
                ClientCard {
                    key: "{client.id}", client: client.clone(),
                    messages: snapshot.messages.iter().filter(|(id, _)| *id == client.id).map(|(_, text)| text.clone()).collect::<Vec<_>>(),
                    show_elevation: snapshot.show_elevation,
                }
            }
        }
    }
}
