use common::token::ApiToken;
use leptos::NodeRef;
use leptos::{component, create_action, create_node_ref, view, IntoView, SignalSet, SignalWith};
use leptos_use::storage::use_session_storage;
use leptos_use::utils::FromToStringCodec;
use phosphor_leptos::IconWeight;
use phosphor_leptos::Lock;
use reqwest::Client;

use crate::BASE_API_URL;

#[component]
pub fn Login() -> impl IntoView {
    let input_element: NodeRef<leptos::html::Input> = create_node_ref();
    let use_secret_action = create_action(|s: &String| use_secret(s.to_owned()));

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let value = input_element().expect("<input> should be mounted").value();
        use_secret_action.dispatch(value);
    };
    let token = use_secret_action.value().read_only();
    let (_, set_api_token, _) = use_session_storage::<String, FromToStringCodec>("api_token");

    view! {
        {move || {
            token
                .with(|t| {
                    if let Some(Ok(t)) = t {
                        set_api_token.set(t.as_str().to_string());
                    }
                })
        }}

        <div id="login_wrapper">
            <span id="login_title">HARMONY</span>
            <form on:submit=on_submit>
                <input
                    type="password"
                    node_ref=input_element
                    id="login_secret"
                    placeholder="secret"
                />
                <button type="submit" value="submit">
                    <Lock id="login_lock" weight=IconWeight::Fill size="60px"/>
                </button>
            </form>
        </div>
    }
}

/// Requests an ApiToken by providing the secret
async fn use_secret(secret: String) -> anyhow::Result<ApiToken> {
    let client = Client::new();
    let res = client
        .post(format!("{}use_secret", BASE_API_URL.as_str()))
        .body(secret)
        .send()
        .await?
        .error_for_status()?;
    Ok(bitcode::decode(&res.bytes().await?)?)
}
