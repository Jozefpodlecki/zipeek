pub struct RouteCase {
    pub path: &'static str,
    pub marker: &'static str,
}

impl RouteCase {
    pub fn navigation_path(&self) -> String {
        self.path.to_owned()
    }
}

pub const ROUTES: &[RouteCase] = &[
    RouteCase {
        path: "/",
        marker: "[data-component='app-shell']",
    },
    RouteCase {
        path: "/random",
        marker: "[data-component='random-page']",
    },
    RouteCase {
        path: "/practice",
        marker: "[data-practice-setup]",
    },
    RouteCase {
        path: "/practice/session/missing-session",
        marker: "[data-component='main-content'] [role='alert']",
    },
    RouteCase {
        path: "/games",
        marker: "[data-component='games-gallery']",
    },
    RouteCase {
        path: "/games/province-quest",
        marker: "[data-game-setup='Province Quest']",
    },
    RouteCase {
        path: "/games/time-weaver",
        marker: "[data-time-weaver-setup]",
    },
    RouteCase {
        path: "/games/tone-detective",
        marker: "[data-component='tone-detective']",
    },
    RouteCase {
        path: "/games/cloud-match",
        marker: "[data-component='cloud-match']",
    },
    RouteCase {
        path: "/games/crosswords",
        marker: "[data-component='hanzi-crosswords']",
    },
    RouteCase {
        path: "/games/classifier-market",
        marker: "[data-game-setup='Classifier Market']",
    },
    RouteCase {
        path: "/games/listening-navigator",
        marker: "[data-game-setup='Listening Navigator']",
    },
    RouteCase {
        path: "/games/sentence-relay",
        marker: "[data-game-setup='Sentence Relay']",
    },
    RouteCase {
        path: "/games/market-bargain",
        marker: "[data-game-setup='Market Bargain']",
    },
    RouteCase {
        path: "/games/radical-forge",
        marker: "[data-game-setup='Radical Forge']",
    },
    RouteCase {
        path: "/games/dialogue-quest",
        marker: "[data-game-setup='Dialogue Quest']",
    },
    RouteCase {
        path: "/recognize-hanzi",
        marker: "[data-component='recognize-hanzi-page']",
    },
    RouteCase {
        path: "/write-hanzi",
        marker: "[data-component='write-hanzi-page']",
    },
    RouteCase {
        path: "/translate-sentence",
        marker: "[data-component='translate-sentence-page']",
    },
    RouteCase {
        path: "/browse",
        marker: "[data-component='browse-page']",
    },
    RouteCase {
        path: "/courses",
        marker: "[data-component='vocabulary-courses']",
    },
    RouteCase {
        path: "/courses/grammar",
        marker: "[data-component='courses-page']",
    },
    RouteCase {
        path: "/courses/pinyin",
        marker: "[data-component='pinyin-course']",
    },
    RouteCase {
        path: "/courses/pinyin/tones",
        marker: "[data-component='course-lesson']",
    },
    RouteCase {
        path: "/settings",
        marker: "[data-component='settings-page']",
    },
    RouteCase {
        path: "/debug",
        marker: "[data-component='main-content']",
    },
    RouteCase {
        path: "/docs",
        marker: "[data-component='docs-page']",
    },
    RouteCase {
        path: "/word-sets",
        marker: "[data-component='word-sets-page']",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn route_cases_have_unique_absolute_paths_and_nonempty_markers() {
        let mut seen = BTreeSet::new();
        for case in ROUTES {
            assert!(
                case.path.starts_with('/'),
                "Non-absolute route: {}",
                case.path
            );
            assert!(
                seen.insert(case.path),
                "Duplicate route case: {}",
                case.path
            );
            assert!(
                !case.marker.is_empty(),
                "Route {} has no readiness marker",
                case.path
            );
        }
    }

    #[test]
    fn only_setup_and_session_are_practice_routes() {
        let cases: Vec<_> = ROUTES
            .iter()
            .filter(|case| case.path.starts_with("/practice"))
            .map(|case| case.path)
            .collect();
        assert_eq!(
            cases,
            vec!["/practice", "/practice/session/missing-session"]
        );
    }

    #[test]
    fn deep_link_fixtures_cover_parameterized_routes() {
        assert!(
            ROUTES
                .iter()
                .any(|case| case.path.starts_with("/practice/session/"))
        );
        assert!(
            ROUTES
                .iter()
                .any(|case| case.path.starts_with("/courses/pinyin/"))
        );
    }
}
