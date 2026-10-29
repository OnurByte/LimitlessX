use crate::app::{Attribution, Footer, Header, call};
use leptos::prelude::*;
use limitless_core::{Post, Profile, Target, Timeline};
use serde::Deserialize;
use serde_json::json;
#[derive(Clone, Deserialize)]
struct Content {
    profile: Option<Profile>,
    post: Option<Post>,
    timeline: Option<Timeline>,
    profile_error: Option<String>,
    timeline_error: Option<String>,
    record_error: Option<String>,
    restricted: bool,
}
fn parameters() -> Result<(String, Target), String> {
    let query = web_sys::window()
        .unwrap()
        .location()
        .search()
        .map_err(|_| "Could not read this address.")?;
    let q = web_sys::UrlSearchParams::new_with_str(&query)
        .map_err(|_| "Could not read this address.")?;
    let token = q
        .get("session")
        .ok_or("Open a profile or post from the extension to start reading.")?;
    let target: Target = serde_json::from_str(
        &q.get("target")
            .ok_or("This reader address is incomplete.")?,
    )
    .map_err(|_| "This reader address is invalid.")?;
    if !target.is_valid() {
        return Err("This reader address is invalid.".into());
    }
    Ok((token, target))
}
#[component]
fn ProfileCard(profile: Profile) -> impl IntoView {
    view! {<section class="profile-card"><div class="profile-heading">{profile.avatar.map(|src|view!{<img class="avatar" src=src alt="" crossorigin="anonymous" referrerpolicy="no-referrer"/>})}<div><h2>{profile.name}</h2><p class="muted">{format!("@{}",profile.handle)}</p></div></div>{profile.biography.map(|bio|view!{<p class="biography">{bio}</p>})}</section>}
}
#[component]
fn PostCard(post: Post, session: String, #[prop(default = false)] detailed: bool) -> impl IntoView {
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let id = post.id.clone();
    let target_url = format!("https://x.com/{}/status/{}", post.author.handle, post.id);
    view! {<article class="post-card"><div class="post-heading"><div><strong>{post.author.name}</strong><span class="muted">{format!(" @{}",post.author.handle)}</span></div>{post.published.map(|date|view!{<time>{date}</time>})}</div><p class="post-text">{post.text}</p>
        <div class="media-grid">{post.media.into_iter().map(|m|if m.kind=="photo"{view!{<img class="post-media" src=m.url alt=m.alt.unwrap_or_else(||"Image from this post".into()) loading="lazy" crossorigin="anonymous" referrerpolicy="no-referrer"/>}.into_any()}else{view!{<video class="post-media" src=m.url poster=m.preview controls=true preload="none" crossorigin="anonymous" aria-label="Video from this post"/>}.into_any()}).collect_view()}</div>
        <div class="post-actions"><a class="source-link" href=target_url target="_blank" rel="noopener noreferrer">"Original on X ↗"</a>{(!detailed).then(||view!{<button class="secondary compact" disabled=move ||busy.get() on:click=move |_|{busy.set(true);let session=session.clone();let id=id.clone();leptos::task::spawn_local(async move{if let Err(e)=call(json!({"op":"open_post","session":session,"id":id})).await{error.set(Some(e));}busy.set(false);});}>"Open post"</button>})}</div>
        {move ||error.get().map(|e|view!{<p class="error" role="alert">{e}</p>})}
    </article>}
}
#[component]
pub fn Reader() -> impl IntoView {
    let params = parameters();
    let content = RwSignal::new(None::<Content>);
    let error = RwSignal::new(None::<String>);
    let busy = RwSignal::new(true);
    let posts = RwSignal::new(Vec::<Post>::new());
    let cursor = RwSignal::new(None::<String>);
    let params = StoredValue::new(params);
    let load = move || {
        let Ok((session, target)) = params.get_value() else {
            error.set(params.get_value().err());
            busy.set(false);
            return;
        };
        busy.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            match call(json!({"op":"read","session":session,"target":target})).await {
                Ok(v) => match serde_json::from_value::<Content>(v) {
                    Ok(c) => {
                        if let Some(t) = &c.timeline {
                            posts.set(t.posts.clone());
                            cursor.set(t.next_cursor.clone());
                        }
                        content.set(Some(c));
                    }
                    Err(_) => error.set(Some("The reader received an unexpected response.".into())),
                },
                Err(e) => error.set(Some(e)),
            }
            busy.set(false);
        });
    };
    load();
    let more = move |_| {
        let Ok((session, target)) = params.get_value() else {
            return;
        };
        busy.set(true);
        error.set(None);
        let next = cursor.get_untracked();
        leptos::task::spawn_local(async move {
            match call(json!({"op":"timeline","session":session,"target":target,"cursor":next}))
                .await
            {
                Ok(v) => match serde_json::from_value::<Timeline>(v) {
                    Ok(t) => {
                        posts.update(|posts| {
                            for post in t.posts {
                                if !posts.iter().any(|p| p.id == post.id) {
                                    posts.push(post);
                                }
                            }
                        });
                        cursor.set(t.next_cursor.filter(|c| Some(c) != next.as_ref()));
                    }
                    Err(_) => error.set(Some("Could not read the next page.".into())),
                },
                Err(e) => error.set(Some(e)),
            }
            busy.set(false);
        });
    };
    view! {<main class="reader"><Header/><div class="reader-title"><div><p class="eyebrow">"ALTERNATIVE SOURCE"</p><h2>{params.get_value().ok().map(|(_,t)|t.label()).unwrap_or_else(||"Public reader".into())}</h2></div><span class="source-pill">"via FxTwitter"</span></div><p class="reader-note">{move ||if content.get().is_some_and(|c|c.restricted){"Opened from a regional restriction notice on X."}else{"Manual public lookup · This view does not add to The Record."}}</p>
        {move ||content.get().map(|c|view!{
            {c.profile.map(|profile|view!{<ProfileCard profile/>})}
            {c.profile_error.map(|e|view!{<p class="notice">{format!("Profile lookup: {e}")}</p>})}
            {c.timeline_error.map(|e|view!{<p class="notice">{format!("Posts: {e}")}</p>})}
            {c.record_error.map(|e|view!{<p class="notice">{format!("Content opened, but The Record could not update: {e}")}</p>})}
            {c.post.map(|post|view!{<PostCard post session=params.get_value().unwrap().0 detailed=true/>})}
            {c.timeline.is_some().then(||view!{<div class="feed-title"><h2>"Public posts"</h2><span class="fine">"Only posts you open count toward The Record."</span></div>})}
        })}
        <div class="posts"><For each=move ||posts.get() key=|p|p.id.clone() children=move |post|view!{<PostCard post session=params.get_value().unwrap().0/>}/></div>
        {move ||(content.get().is_some_and(|c|c.timeline.is_some()) && posts.get().is_empty() && !busy.get()).then(||view!{<p class="empty-state">"No public posts were returned by the provider."</p>})}
        {move ||cursor.get().map(|_|view!{<button class="secondary load-more" on:click=more disabled=move ||busy.get()>"Load more posts"</button>})}
        {move ||busy.get().then(||view!{<p class="loading" role="status">"Loading public content…"</p>})}
        {move ||error.get().map(|e|view!{<section class="error panel" role="alert"><p>{e}</p><button class="secondary" disabled=move ||busy.get() on:click=move |_|load()>"Try again"</button></section>})}
        <Attribution/><Footer/>
    </main>}
}
