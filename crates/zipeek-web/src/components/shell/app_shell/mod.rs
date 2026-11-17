use crate::preferences::PreferencesStore;
mod header;
mod other_tab;
mod session_background;
mod session_reminder;
use self::header::ShellHeader;
use self::other_tab::OtherTabIndicator;
use self::session_background::SessionBackground;
use self::session_reminder::SessionReminder;
use crate::services::practice_history::{RecordingPreference, recording_preference};
use gloo::events::EventListener;
use web_sys::HtmlElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    components::{AppFooter, LandingPage, Navigation},
    hooks::use_app_environment,
    preferences::KEY as PREFERENCES_KEY,
    route::Route,
};

#[derive(Properties, PartialEq)]
pub struct AppShellProps {
    pub children: Children,
}

#[function_component(AppShell)]
pub fn app_shell(props: &AppShellProps) -> Html {
    let environment = use_app_environment();
    let content = use_node_ref();
    let route = use_route::<Route>();
    let recording_enabled = {
        let storage = environment.local_storage.clone();
        use_state(move || {
            matches!(
                recording_preference(&storage),
                Ok(RecordingPreference::Enabled)
            )
        })
    };
    {
        let storage = environment.local_storage.clone();
        let window = environment.window.clone();
        let state = recording_enabled.clone();
        use_effect_with((), move |_| {
            let state_changed = state.clone();
            let storage_changed = storage.clone();
            let own_change = EventListener::new(&window, "zipeek-recording-change", move |_| {
                state_changed.set(matches!(
                    recording_preference(&storage_changed),
                    Ok(RecordingPreference::Enabled)
                ));
            });
            let storage_changed = storage.clone();
            let cross_tab_state = state.clone();
            let cross_tab_change = EventListener::new(&window, "storage", move |_| {
                cross_tab_state.set(matches!(
                    recording_preference(&storage_changed),
                    Ok(RecordingPreference::Enabled)
                ));
            });
            move || {
                drop(own_change);
                drop(cross_tab_change);
            }
        });
    }
    {
        let content = content.clone();
        let document = environment.document.clone();
        use_effect_with(route.clone(), move |route| {
            document.set_title(&format!(
                "{} · Zipeek",
                route.as_ref().map_or("Page not found", Route::title)
            ));
            if let Some(element) = content.cast::<HtmlElement>()
                && let Err(error) = element.focus()
            {
                log::warn!("Could not focus route content: {error:?}");
            }
        });
    }
    let initial_storage = environment.local_storage.clone();
    let entered = use_state(move || {
        [
            PREFERENCES_KEY,
            "zipeek.user_word_sets.v1",
            "zipeek.excluded_words",
        ]
        .iter()
        .any(|key| initial_storage.get_item(key).ok().flatten().is_some())
    });
    let show_welcome = matches!(route.as_ref(), Some(Route::Home)) && !*entered;
    let on_enter = {
        let entered = entered.clone();
        let storage = environment.local_storage.clone();
        Callback::from(move |_: MouseEvent| {
            if let Err(error) = PreferencesStore::new(&storage).ensure_initialized() {
                log::warn!("Could not save initial preferences: {error}");
            }
            entered.set(true);
        })
    };
    let centered_title = route
        .as_ref()
        .filter(|route| {
            !matches!(route, Route::Home)
                && !matches!(route, Route::Game { slug } if matches!(slug.as_str(), "cloud-match" | "crosswords"))
        })
        .map(Route::title);
    let write_hanzi_workspace = matches!(route.as_ref(), Some(Route::WriteHanzi));
    let game_workspace =
        matches!(route.as_ref(), Some(Route::Game { slug }) if slug == "crosswords");
    let practice_setup_workspace = matches!(route.as_ref(), Some(Route::PracticeSetup));
    let courses_workspace = matches!(route.as_ref(), Some(Route::VocabularyCourse | Route::GrammarCourse | Route::PinyinCourse));
    let expanded = use_state(|| true);
    let on_drawer_toggle = {
        let expanded = expanded.clone();
        Callback::from(move |_| expanded.set(!*expanded))
    };
    let onskip = {
        let content = content.clone();
        Callback::from(move |event: MouseEvent| {
            event.prevent_default();
            if let Some(element) = content.cast::<HtmlElement>()
                && let Err(error) = element.focus()
            {
                log::warn!("Could not focus main content: {error:?}");
            }
        })
    };
    if matches!(
        route.as_ref(),
        Some(Route::Docs | Route::DocsArticle { .. })
    ) {
        return html! { <div data-component="docs-shell" class="min-h-dvh bg-surface text-ink">{props.children.clone()}</div> };
    }
    if matches!(route.as_ref(), Some(Route::LocalPracticeSession { .. })) {
        return html! {
            <div data-component="practice-session-shell" data-shell-mode="practice-session" id="main-content"
                class="relative isolate h-dvh min-h-0 overflow-hidden bg-canvas text-ink"
                tabindex="-1" ref={content}>
                <SessionBackground />
                <div class="relative z-10 h-full min-h-0 px-3 py-4 sm:px-6 sm:py-6">
                    {props.children.clone()}
                </div>
            </div>
        };
    }
    if matches!(
        route.as_ref(),
        Some(
            Route::GrammarLesson { .. }
                | Route::GrammarExpanded { .. }
                | Route::GrammarAssessment { .. }
                | Route::CourseLesson { .. }
        )
    ) {
        return html! {
            <div data-component="course-lesson-shell" data-shell-mode="course-lesson"
                class="relative isolate min-h-dvh bg-canvas text-ink">
                <SessionBackground />
                <main class="relative z-10 min-h-dvh overflow-y-auto px-4 py-6 sm:px-6 sm:py-10" id="main-content" tabindex="-1" ref={content}>
                    {props.children.clone()}
                </main>
            </div>
        };
    }
    if show_welcome {
        return html! { <LandingPage
            on_enter={on_enter}
        /> };
    }
    let standard_navigation_content = {
        if game_workspace {
            Html::default()
        } else {
            html! {
                <ShellHeader
                    centered_title={centered_title.map(str::to_owned)}
                    expanded={*expanded}
                    recording_enabled={*recording_enabled}
                    {game_workspace}
                    {write_hanzi_workspace}
                    on_drawer_toggle={on_drawer_toggle}
                />
            }
        }
    };
    let element_classes_1 = classes!(
        "flex",
        "min-h-dvh",
        "flex-col",
        "items-stretch",
        "md:grid",
        "md:grid-rows-[var(--topbar-height)_1fr]",
        "md:items-start",
        if practice_setup_workspace {
            "md:h-dvh md:min-h-0 md:overflow-hidden"
        } else if courses_workspace {
            "lg:h-dvh lg:min-h-0 lg:overflow-hidden"
        } else {
            ""
        },
        if write_hanzi_workspace {
            classes!(
                "writing-desktop:h-dvh",
                "writing-desktop:min-h-0",
                "writing-desktop:overflow-hidden"
            )
        } else {
            Classes::new()
        },
        if game_workspace {
            classes!(
                "!flex",
                "!min-h-dvh",
                "!flex-col",
                "lg:!h-dvh",
                "lg:!min-h-0",
                "lg:overflow-hidden"
            )
        } else {
            Classes::new()
        },
        if game_workspace {
            "md:grid-cols-1"
        } else if *expanded {
            "md:grid-cols-[15rem_minmax(0,1fr)]"
        } else {
            "md:grid-cols-[4.5rem_minmax(0,1fr)]"
        },
    );
    let element_classes_2 = classes!(
        if game_workspace { "hidden" } else { "flex" },
        "w-full",
        "flex-col",
        "gap-2",
        "border-b",
        "border-line",
        "bg-surface",
        "px-3",
        "py-3",
        "backdrop-blur-[12px]",
        "md:sticky",
        "md:top-(--topbar-height)",
        "md:row-start-2",
        "md:h-[calc(100dvh-var(--topbar-height))]",
        "md:gap-8",
        "md:border-r",
        "md:border-b-0",
        if *expanded {
            "md:p-5"
        } else {
            "md:px-2 md:py-5"
        },
    );
    let element_classes_3 = classes!(
        "flex",
        "min-w-0",
        "flex-1",
        "flex-col",
        if game_workspace {
            "md:row-start-1 md:col-start-1 md:min-h-0"
        } else {
            "md:row-start-2 md:col-start-2 md:min-h-[calc(100dvh-var(--topbar-height))]"
        },
        if practice_setup_workspace {
            "md:!h-[calc(100dvh-var(--topbar-height))] md:!min-h-0 md:overflow-hidden"
        } else if courses_workspace {
            "lg:!h-[calc(100dvh-var(--topbar-height))] lg:!min-h-0 lg:overflow-hidden"
        } else {
            ""
        },
        if write_hanzi_workspace {
            classes!(
                "writing-desktop:h-[calc(100dvh-var(--topbar-height))]",
                "writing-desktop:min-h-0",
                "writing-desktop:overflow-hidden"
            )
        } else {
            Classes::new()
        },
        if game_workspace {
            classes!(
                "!w-full",
                "!min-w-0",
                "!flex-1",
                "md:!col-span-full",
                "md:!row-span-full",
                "lg:!min-h-0"
            )
        } else {
            Classes::new()
        },
    );
    let element_classes_4 = classes!(
        "w-full",
        "flex-1",
        "px-5",
        "py-6",
        "md:px-[clamp(1.5rem,4vw,4.5rem)]",
        if practice_setup_workspace {
            "md:min-h-0 md:overflow-hidden md:!py-3"
        } else if courses_workspace {
            "lg:min-h-0 lg:overflow-hidden lg:!py-3"
        } else {
            ""
        },
        if write_hanzi_workspace {
            classes!(
                "writing-desktop:!h-full",
                "writing-desktop:!min-h-0",
                "writing-desktop:!overflow-hidden",
                "writing-desktop:!px-3",
                "writing-desktop:!py-2"
            )
        } else {
            Classes::new()
        },
        if game_workspace {
            classes!(
                "!p-[clamp(.5rem,1.4vw,1rem)]",
                "lg:!flex",
                "lg:!min-h-0",
                "lg:!overflow-hidden"
            )
        } else {
            Classes::new()
        },
    );
    html! {
        <div
            data-component="app-shell"
            class={element_classes_1}
        >
            <a
                data-skip-link=""
                class="fixed top-4 left-4 z-[100] -translate-y-[200%] rounded-lg border border-accent bg-surface px-4 py-[.8rem] text-accent transition-transform duration-150 focus:translate-y-0 motion-reduce:transition-none"
                href="#main-content"
                onclick={onskip}
            >
                {"Skip to content"}
            </a>
            {standard_navigation_content}
            <SessionReminder route={route.clone()} />
            <OtherTabIndicator />
            <aside
                data-component="navigation-drawer"
                id="app-drawer"
                class={element_classes_2}
                aria-label="Zipeek"
            >
                <Navigation
                    collapsed={!*expanded}
                />
            </aside>
            <div
                class={element_classes_3}
            >
                <main
                    data-component="main-content"
                    id="main-content"
                    class={element_classes_4}
                    tabindex="-1"
                    ref={content}
                >
                    {props.children.clone()}
                </main>
                {if write_hanzi_workspace || game_workspace || practice_setup_workspace || courses_workspace {Html::default()} else {html! {
                    <AppFooter />}}}
            </div>
        </div>
    }
}
