use common::track::Track;
use leptos::{
    component, create_resource, view, CollectView, IntoView, SignalGet, SignalGetUntracked,
};
use leptos_use::{storage::use_session_storage, utils::FromToStringCodec};
use phosphor_leptos::{ArrowCircleLeft, ArrowCircleRight, IconWeight, MagnifyingGlass};

use crate::requests::get_all_tracks;

#[component]
pub fn TrackList() -> impl IntoView {
    let (api_token, _, _) = use_session_storage::<String, FromToStringCodec>("api_token");
    let track_resource = create_resource(
        || (),
        move |_| async move { get_all_tracks(api_token.get_untracked()).await },
    );

    view! {
        <div class="track_list">
            <TrackListFilter/>
            {move || match track_resource.get() {
                Some(Ok(tracks)) => {
                    tracks.into_iter().map(|track| view! { <TrackListing track/> }).collect_view()
                }
                Some(Err(e)) => format!("Failed loading tracks: {e}").into_view(),
                None => view! { "LOADING..." }.into_view(),
            }}

            <TrackListNavigator/>
        </div>
    }
}

#[component]
pub fn TrackListing(track: Track) -> impl IntoView {
    view! {
        <div class="track_listing">
            <div class="track_listing_left">
                <span class="track_title">{track.title}</span>
                <span class="track_artists">
                    {track.artists.into_iter().intersperse(", ".to_string()).collect::<String>()}
                </span>
            </div>
            <div class="track_listing_right">
                <div class="track_date">{track.date_archived.to_string()}</div>
                <a target="_blank" href=format!("https://{}", track.url) class="track_url">
                    {track.url}
                </a>
            </div>
        </div>
    }
}

#[component]
pub fn TrackListFilter() -> impl IntoView {
    view! {
        <div class="track_list_filter">
            <div class="track_list_filter_row0">
                <input type="text"/>
                <MagnifyingGlass weight=IconWeight::Bold size="40px" class="hoverable"/>
            </div>
            <div class="track_list_filter_row1">
                <select>
                    <option>Track Title</option>
                    <option>First Artist</option>
                    <option>Date Archived</option>
                    <option>Origin URL</option>
                </select>
                <button>Edit Mode</button>
            </div>
        </div>
    }
}

#[component]
pub fn TrackListNavigator() -> impl IntoView {
    view! {
        <div class="track_list_navigator">
            <ArrowCircleLeft
                weight=IconWeight::Fill
                size="50px"
                class="track_list_navigator_button"
            />
            <span class="track_list_navigator_number">40</span>
            <ArrowCircleRight
                weight=IconWeight::Fill
                size="50px"
                class="track_list_navigator_button"
            />
        </div>
    }
}
