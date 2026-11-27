use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::*;

use crate::{hooks::use_app_environment, preferences::PreferencesStore, route::Route};
use gloo::events::EventListener;

fn active(current: Option<&Route>, destination: &Route) -> bool {
    match destination {
        Route::Games => matches!(current, Some(Route::Games | Route::Game { .. })),
        Route::Courses => matches!(
            current,
            Some(
                Route::Courses
                    | Route::GrammarCourse
                    | Route::VocabularyCourse
                    | Route::PinyinCourse
                    | Route::CourseLesson { .. }
                    | Route::GrammarLesson { .. }
                    | Route::GrammarExpanded { .. }
                    | Route::GrammarAssessment { .. }
            )
        ),
        Route::Docs => matches!(current, Some(Route::Docs | Route::DocsArticle { .. })),
        Route::PracticeSetup => matches!(
            current,
            Some(Route::PracticeSetup | Route::LocalPracticeSession { .. })
        ),
        _ => current == Some(destination),
    }
}

pub(crate) struct NavigationItemConfig {
    pub(crate) route: Route,
    pub(crate) label: &'static str,
    pub(crate) icon: IconData,
}

pub(crate) const NAVIGATION_ITEMS: [NavigationItemConfig; 14] = [
    NavigationItemConfig {
        route: Route::Home,
        label: "Home",
        icon: IconData::LUCIDE_HOME,
    },
    NavigationItemConfig {
        route: Route::PracticeSetup,
        label: "Practice",
        icon: IconData::LUCIDE_REFRESH_CW,
    },
    NavigationItemConfig {
        route: Route::Courses,
        label: "Courses",
        icon: IconData::LUCIDE_GRADUATION_CAP,
    },
    NavigationItemConfig {
        route: Route::Browse,
        label: "Browse",
        icon: IconData::LUCIDE_BOOK_OPEN,
    },
    NavigationItemConfig {
        route: Route::Stats,
        label: "Stats",
        icon: IconData::LUCIDE_LAYOUT_GRID,
    },
    NavigationItemConfig {
        route: Route::Profile,
        label: "Profile",
        icon: IconData::LUCIDE_USER,
    },
    NavigationItemConfig {
        route: Route::WriteHanzi,
        label: "Write Hanzi",
        icon: IconData::LUCIDE_FILE_TEXT,
    },
    NavigationItemConfig {
        route: Route::RecognizeHanzi,
        label: "Recognize Hanzi",
        icon: IconData::LUCIDE_SCAN_LINE,
    },
    NavigationItemConfig {
        route: Route::TranslateSentence,
        label: "Translate",
        icon: IconData::LUCIDE_LANGUAGES,
    },
    NavigationItemConfig {
        route: Route::Games,
        label: "Games",
        icon: IconData::LUCIDE_GAMEPAD,
    },
    NavigationItemConfig {
        route: Route::Record,
        label: "Record",
        icon: IconData::LUCIDE_MIC,
    },
    NavigationItemConfig {
        route: Route::Random,
        label: "Random",
        icon: IconData::LUCIDE_SHUFFLE,
    },
    NavigationItemConfig {
        route: Route::Settings,
        label: "Settings",
        icon: IconData::LUCIDE_SETTINGS,
    },
    NavigationItemConfig {
        route: Route::Docs,
        label: "Docs",
        icon: IconData::LUCIDE_INFO,
    },
];

#[derive(Properties, PartialEq)]
pub struct NavigationItemProps {
    pub route: Route,
    pub label: &'static str,
    pub icon: IconData,
    pub collapsed: bool,
}

#[function_component(NavigationItem)]
fn navigation_item(props: &NavigationItemProps) -> Html {
    let current = use_route::<Route>();
    let selected = active(current.as_ref(), &props.route);
    let label = if props.collapsed {
        Html::default()
    } else {
        html! {
            <span>
                {props.label}
            </span>
        }
    };

    let element_classes_1 = classes!(
        "flex",
        "min-h-10",
        "flex-1",
        "items-center",
        "justify-center",
        "rounded-[.65rem]",
        "px-[.6rem]",
        "py-[.6rem]",
        "text-[.8rem]",
        "font-semibold",
        "no-underline",
        "transition-colors",
        "duration-150",
        "motion-reduce:transition-none",
        "md:min-h-12",
        "md:py-[.8rem]",
        "md:text-sm",
        if selected {
            "bg-brand-action text-white shadow-sm"
        } else {
            "text-muted hover:bg-raised hover:text-ink"
        },
        if props.collapsed {
            "md:justify-center md:px-2"
        } else {
            "md:justify-start md:gap-[.9rem] md:px-[.9rem]"
        },
    );
    html! {
        <a
            href={format!("#{}", props.route.to_path())}
            data-navigation-link=""
            class={element_classes_1}
            aria-current={if selected { "page" } else { "false" }}
            title={props.label}
        >
            <span
                aria-hidden="true"
                class="inline-flex shrink-0 items-center justify-center"
            >
                <Icon
                    data={props.icon}
                    width="19px"
                    height="19px"
                />
            </span>
            {label}
        </a>
    }
}

#[derive(Properties, PartialEq)]
pub struct NavigationProps {
    #[prop_or_default]
    pub collapsed: bool,
}

pub(crate) fn drawer_item_is_required(route: &Route) -> bool {
    matches!(route, Route::Profile | Route::Settings)
}

pub(crate) fn drawer_item_visible(route: &Route, hidden: &[String]) -> bool {
    drawer_item_is_required(route) || !hidden.iter().any(|key| key == &route.to_path())
}

#[function_component(Navigation)]
pub fn navigation(props: &NavigationProps) -> Html {
    let environment = use_app_environment();
    let storage = environment.local_storage.clone();
    let hidden = use_state(move || PreferencesStore::new(&storage).read().hidden_drawer_items);
    {
        let storage = environment.local_storage.clone();
        let window = environment.window.clone();
        let hidden = hidden.clone();
        use_effect_with((), move |_| {
            let reload = {
                let hidden = hidden.clone();
                let storage = storage.clone();
                move || hidden.set(PreferencesStore::new(&storage).read().hidden_drawer_items)
            };
            let same_tab = EventListener::new(&window, "zipeek-navigation-change", {
                let reload = reload.clone();
                move |_| reload()
            });
            let other_tab = EventListener::new(&window, "storage", move |_| reload());
            move || {
                drop(same_tab);
                drop(other_tab);
            }
        });
    }
    let debug_navigation = if cfg!(debug_assertions) {
        let debug_label = if props.collapsed {
            Html::default()
        } else {
            html! {
                <span>
                    {"Debug"}
                </span>
            }
        };
        html! {
            <a
                data-debug-navigation=""
                href="#/debug"
                class="flex min-h-10 items-center gap-3 rounded-lg px-3 py-3 text-sm text-muted hover:bg-raised"
                title="Development diagnostics"
            >
                <Icon
                    data={IconData::LUCIDE_TERMINAL}
                    width="19px"
                    height="19px"
                />
                {debug_label}
            </a>
        }
    } else {
        Html::default()
    };

    html! {
        <nav
            data-component="main-navigation"
            class="flex flex-wrap gap-[.4rem] md:flex-col md:gap-2"
            aria-label="Main navigation"
        >
            {for NAVIGATION_ITEMS.iter().filter(|item| drawer_item_visible(&item.route, &hidden)).map(|item| html! {
                <NavigationItem
                    key={item.label}
                    route={item.route.clone()}
                    label={item.label}
                    icon={item.icon}
                    collapsed={props.collapsed}
                />
            })}
            {debug_navigation}
        </nav>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grouped_navigation_entries_follow_their_parent_routes() {
        assert!(active(
            Some(&crate::route::GameId::CloudMatch.route()),
            &Route::Games
        ));
        assert!(active(Some(&Route::PinyinCourse), &Route::Courses));
        assert!(active(
            Some(&Route::LocalPracticeSession {
                id: "session".to_owned()
            }),
            &Route::PracticeSetup
        ));
        assert!(!active(
            Some(&crate::route::GameId::CloudMatch.route()),
            &Route::PracticeSetup
        ));
    }

    #[test]
    fn essential_drawer_entries_cannot_be_hidden() {
        let hidden = vec![
            "/profile".to_owned(),
            "/settings".to_owned(),
            "/games".to_owned(),
        ];
        assert!(drawer_item_visible(&Route::Profile, &hidden));
        assert!(drawer_item_visible(&Route::Settings, &hidden));
        assert!(!drawer_item_visible(&Route::Games, &hidden));
    }

    #[test]
    fn navigation_configurations_have_unique_routes_and_labels() {
        for (index, item) in NAVIGATION_ITEMS.iter().enumerate() {
            assert!(!item.label.is_empty());
            for earlier in &NAVIGATION_ITEMS[..index] {
                assert_ne!(item.route, earlier.route);
                assert_ne!(item.label, earlier.label);
            }
        }
    }
}
