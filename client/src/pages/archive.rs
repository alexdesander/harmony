use std::str::FromStr;

use common::candidate::Candidate;
use leptos::{
    component, create_action, create_node_ref, create_signal, html, view, IntoView, NodeRef,
    SignalSet,
};
use leptos_use::{use_cookie, utils::FromToStringCodec};

use crate::requests::archive_track;

#[component]
pub fn Archive() -> impl IntoView {
    let (api_token, _) = use_cookie::<String, FromToStringCodec>("api_token");
    let (hint, set_hint) = create_signal(
        String::from_str("Please fill out the fields below and make sure you made no mistakes.")
            .unwrap(),
    );

    let url_element: NodeRef<html::Input> = create_node_ref();
    let title_element: NodeRef<html::Input> = create_node_ref();
    let artists_element: NodeRef<html::Input> = create_node_ref();
    let send_action = create_action(move |candidate: &Candidate| {
        let candidate = candidate.clone();
        async move {
            match archive_track(api_token, candidate).await {
                Ok(_) => set_hint.set("Archive request sent successfully!".to_string()),
                Err(e) => set_hint.set(format!("Failed to send request: {}", e)),
            }
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let url = url_element().unwrap().value().trim().to_owned();
        let title = url_element().unwrap().value().trim().to_owned();
        let artists = url_element().unwrap().value().trim().to_owned();
        let mut artists = artists
            .split(", ")
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();
        artists.sort();

        let candidate = Candidate {
            url,
            title: if title.is_empty() { None } else { Some(title) },
            artists,
        };
        let candidate = match candidate.validated() {
            Ok(candidate) => candidate,
            Err(e) => {
                set_hint.set(e.to_string());
                return;
            }
        };
        send_action.dispatch(candidate);
    };

    view! {
        <div class="archive_card">
            <span class="title">TRACK ARCHIVING REQUEST</span>
            <span class="archive_card_hint">{move || hint()}</span>
            <form on:submit=on_submit>
                <label>URL</label>
                <input type="text" placeholder="Required" node_ref=url_element/>
                <label>Track Title</label>
                <input type="text" placeholder="Optional" node_ref=title_element/>
                <label>Artists</label>
                <input
                    type="text"
                    placeholder="Optional1, Optional2, ..."
                    node_ref=artists_element
                />
                <button type="submit">SUBMIT</button>
            </form>
        </div>
    }
}
