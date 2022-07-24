use reqwasm::http::Request;
use serde::Deserialize;
use yew::prelude::*;

#[derive(Clone, PartialEq, Deserialize, Debug)]
struct State {
    locked: bool,
}

#[function_component(Doorlock)]
fn doorlock() -> Html {
    let state = use_state(|| State { locked: false });
    {
        let state = state.clone();
        use_effect_with_deps(
            move |_| {
                let state = state.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match Request::get("/doorlock/state").send().await {
                        Ok(result) => match result.json().await {
                            Ok(fetched_state) => {
                                log::info!("FETCHED: {:?}", fetched_state);
                                state.set(fetched_state);
                            }
                            Err(e) => {
                                log::warn!("Error decoding API response: {:?}", e);
                            }
                        },
                        Err(e) => {
                            log::warn!("Error retrieving lock state: {:?}", e);
                        }
                    }
                });
                || ()
            },
            (),
        );
    }

    let on_lock_toggle = {
        let state = state.clone();
        Callback::from(move |_| {
            let path = if state.locked {
                "/doorlock/unlock"
            } else {
                "/doorlock/lock"
            };
            wasm_bindgen_futures::spawn_local(async move {
                log::info!("PATH: {:?}", path);
                match Request::put(path).send().await {
                    Ok(_r) => {}
                    Err(e) => {
                        log::warn!("Error setting lock state: {:?}", e);
                    }
                }
            });
        })
    };

    let state_desc = if state.locked { "Unlock" } else { "Lock" };

    html! {
        <button type="button" onclick={on_lock_toggle.clone()}>{state_desc}</button>
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <div>
        <h1>{ "Doorlock" }</h1>
        <Doorlock />
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
