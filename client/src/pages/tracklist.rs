use common::track::Track;
use leptos::wasm_bindgen::JsCast;
use leptos::{
    component, create_resource, create_signal, view, CollectView, IntoView, ReadSignal, SignalGet,
    SignalGetUntracked, SignalSet, SignalUpdate, WriteSignal,
};
use leptos_use::use_cookie;
use leptos_use::utils::FromToStringCodec;
use phosphor_leptos::{
    ArrowCircleLeft, ArrowCircleRight, Download, IconWeight, MagnifyingGlass, PlayCircle, Queue, X,
};
use web_sys::{HtmlFormElement, HtmlInputElement};

use crate::requests::get_all_tracks;
use crate::BASE_API_URL;

#[component]
pub fn TrackList() -> impl IntoView {
    let (api_token, _) = use_cookie::<String, FromToStringCodec>("api_token");
    let track_resource = create_resource(
        || (),
        move |_| async move { get_all_tracks(api_token).await },
    );
    let (page, set_page) = create_signal(0);
    let (page_count, set_page_count) = create_signal(0);

    let page_listing_count = 10;
    let (viewed_track, set_viewed_track): (ReadSignal<Option<Track>>, WriteSignal<Option<Track>>) =
        create_signal(None);
    view! {
        <div class="track_list">
            <TrackListFilter/>
            {move || match track_resource.get() {
                Some(Ok(tracks)) => {
                    set_page_count.set(tracks.len() as u32 / page_listing_count + 1);
                    tracks
                        .into_iter()
                        .map(|track| {
                            view! { <TrackListing track set_viewed_track/> }
                        })
                        .collect_view()
                }
                Some(Err(e)) => format!("Failed loading tracks: {e}").into_view(),
                None => view! { "LOADING..." }.into_view(),
            }}

            {move || {
                if viewed_track.get().is_some() {
                    view! { <TrackCard track=viewed_track.get().unwrap() set_viewed_track/> }
                } else {
                    view! {}.into_view()
                }
            }}

            <TrackListNavigator page set_page page_count/>
        </div>
    }
}

#[component]
pub fn TrackListing(track: Track, set_viewed_track: WriteSignal<Option<Track>>) -> impl IntoView {
    view! {
        <div
            class="track_listing"
            on:click={
                let _track = track.clone();
                move |_| { set_viewed_track.set(Some(_track.clone())) }
            }
        >

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
pub fn TrackCard(track: Track, set_viewed_track: WriteSignal<Option<Track>>) -> impl IntoView {
    let (api_token, _) = use_cookie::<String, FromToStringCodec>("api_token");
    view! {
        <div class="track_card_wrapper">
            <div class="track_card">
                <div class="track_card_top">
                    <X
                        weight=IconWeight::Regular
                        size="60px"
                        class="hoverable"
                        on:click=move |_| {
                            set_viewed_track.set(None);
                        }
                    />

                    <div class="track_card_top_stamp">
                        <span class="track_card_id">Track # {track.id}</span>
                        <span class="track_card_date">{track.date_archived.to_string()}</span>
                    </div>
                </div>

                <div class="track_card_play_actions">
                    <PlayCircle weight=IconWeight::Regular size="60%" class="hoverable"/>
                    <Queue weight=IconWeight::Regular size="60%" class="hoverable"/>
                    <Download
                        weight=IconWeight::Regular
                        size="60%"
                        class="hoverable"
                        on:click=move |_| {
                            let window = web_sys::window().expect("no global `window` exists");
                            let document = window
                                .document()
                                .expect("should have a document on window");
                            let body = document.body().expect("document should have a body");
                            let single_track_download_form: web_sys::Element = document
                                .get_element_by_id("single_track_download_form")
                                .expect(
                                    "Expected invisible_form input with id: single_track_download_form to exist",
                                );
                            let single_track_download_form = single_track_download_form
                                .dyn_into::<HtmlFormElement>()
                                .unwrap();
                            let input_ids: web_sys::Element = document
                                .get_element_by_id("track_ids_to_download")
                                .expect(
                                    "Expected invisible_form input with id: track_ids_to_download to exist",
                                );
                            let input_ids = input_ids.dyn_into::<HtmlInputElement>().unwrap();
                            input_ids.set_value(&serde_json::to_string(&vec![track.id]).unwrap());
                            single_track_download_form.submit().unwrap();
                        }
                    />

                </div>
                <span class="track_card_title">{track.title}</span>
                <span class="track_card_artists">
                    {track.artists.into_iter().intersperse(", ".to_string()).collect::<String>()}
                </span>
                <a target="_blank" href=format!("https://{}", track.url) class="track_card_url">
                    {track.url}
                </a>
            </div>
        </div>
        <form
            id="single_track_download_form"
            target="_blank"
            class="invisible_form"
            method="post"
            action=format!(
                "{}download_tracks?api_token={}",
                *BASE_API_URL,
                api_token.get_untracked().unwrap_or("".to_string()),
            )
        >

            <input id="track_ids_to_download" name="ids" type="hidden" value=""/>
        </form>
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
pub fn TrackListNavigator(
    page: ReadSignal<u32>,
    set_page: WriteSignal<u32>,
    page_count: ReadSignal<u32>,
) -> impl IntoView {
    view! {
        <div class="track_list_navigator">
            <ArrowCircleLeft
                weight=IconWeight::Fill
                size="50px"
                class="track_list_navigator_button"
                on:click=move |_| {
                    if page.get_untracked() != 0 {
                        set_page.update(|v| *v -= 1)
                    }
                }
            />

            <span class="track_list_navigator_number">{move || page()}</span>
            <ArrowCircleRight
                weight=IconWeight::Fill
                size="50px"
                class="track_list_navigator_button"
                on:click=move |_| {
                    if page.get_untracked() + 1 < page_count.get_untracked() {
                        set_page.update(|v| *v += 1)
                    }
                }
            />

        </div>
    }
}
