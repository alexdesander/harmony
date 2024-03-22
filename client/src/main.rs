#![feature(iter_intersperse)]

use leptos::DynAttrs;
use leptos::{component, mount_to_body, view, IntoView};
use leptos_meta::{provide_meta_context, Html, Meta, Title};
use leptos_router::{Outlet, Route, Router, Routes, A};
use leptos_use::storage::use_session_storage;
use leptos_use::utils::FromToStringCodec;
use once_cell::sync::Lazy;
use phosphor_leptos::{ArchiveBox, Database, IconWeight, Playlist};

use crate::pages::archive::Archive;
use crate::pages::playlists::Playlists;
use crate::pages::{login::Login, tracklist::TrackList};

pub mod pages;
pub mod requests;

pub static BASE_API_URL: Lazy<String> = Lazy::new(|| {
    if !cfg!(debug_assertions) {
        let base_url = web_sys::window().unwrap().origin();
        format!("{}/api/", base_url)
    } else {
        "http://localhost:7000/".to_string()
    }
});

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! { <App/> }
    })
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let (api_token, _, _) = use_session_storage::<String, FromToStringCodec>("api_token");

    view! {
        <Html lang="en" dir="ltr" attr:data-theme="light"/>
        <Title text="Harmony"/>
        <Meta charset="UTF-8"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <div id="base">
            <Router>
                <Routes>
                    <Route
                        path="/"
                        view=move || {
                            view! {
                                {move || {
                                    if !api_token().is_empty() {
                                        view! {
                                            <NavBar/>
                                            <Outlet/>
                                            <Footer/>
                                        }
                                            .into_view()
                                    } else {
                                        view! { <Login/> }.into_view()
                                    }
                                }}
                            }
                        }
                    >

                        <Route path="/" view=TrackList/>
                        <Route path="/archive" view=Archive/>
                        <Route path="/playlists" view=Playlists/>
                    </Route>

                </Routes>
            </Router>
        </div>
    }
}

#[component]
pub fn NavBar() -> impl IntoView {
    view! {
        <nav id="main_nav">
            <A href="/">
                <Database weight=IconWeight::Regular size="70px" class="hoverable"/>
            </A>
            <A href="/playlists">
                <Playlist weight=IconWeight::Regular size="70px" class="hoverable"/>
            </A>
            <A href="/archive">
                <ArchiveBox weight=IconWeight::Regular size="70px" class="hoverable"/>
            </A>
        </nav>
    }
}

#[component]
pub fn Footer() -> impl IntoView {
    view! { <nav></nav> }
}
