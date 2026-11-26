use std::collections::BTreeMap;

use zipeek_docs::{DocumentEntry, DocumentHeading};

pub(super) type DocumentGroups = BTreeMap<String, BTreeMap<String, Vec<DocumentEntry>>>;

#[derive(Clone, PartialEq, Eq)]
pub(super) struct DocumentToc {
    pub path: String,
    pub headings: Vec<DocumentHeading>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DocumentationSection {
    GettingStarted,
    ProductUx,
    LearningPractice,
    Games,
    DictionaryData,
    Architecture,
    Engineering,
    TestingQuality,
    BuildDeployment,
    ProjectManagement,
    ArchiveHistory,
    Issues,
}

impl DocumentationSection {
    const ORDER: [Self; 12] = [
        Self::GettingStarted,
        Self::ProductUx,
        Self::LearningPractice,
        Self::Games,
        Self::DictionaryData,
        Self::Architecture,
        Self::Engineering,
        Self::TestingQuality,
        Self::BuildDeployment,
        Self::ProjectManagement,
        Self::ArchiveHistory,
        Self::Issues,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::GettingStarted => "Getting started",
            Self::ProductUx => "Product & UX",
            Self::LearningPractice => "Learning & Practice",
            Self::Games => "Games",
            Self::DictionaryData => "Dictionary & Data",
            Self::Architecture => "Architecture",
            Self::Engineering => "Engineering",
            Self::TestingQuality => "Testing & Quality",
            Self::BuildDeployment => "Build & Deployment",
            Self::ProjectManagement => "Project Management",
            Self::ArchiveHistory => "Archive & History",
            Self::Issues => "Issues",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DocumentStatus {
    Maintained,
    Historical,
    Internal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DocumentMetadata {
    pub section: DocumentationSection,
    pub topic: String,
    pub status: DocumentStatus,
    pub order: u16,
}

pub(super) fn display_name(value: &str) -> String {
    value
        .split(['-', '_', ' '])
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            chars.next().map_or_else(String::new, |first| {
                format!("{}{}", first.to_uppercase(), chars.as_str())
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn document_metadata(path: &str) -> DocumentMetadata {
    use DocumentationSection as Section;
    let name = path.rsplit('/').next().unwrap_or(path);
    let historical = path.starts_with("archive-history/");
    let status = if historical {
        DocumentStatus::Historical
    } else {
        DocumentStatus::Maintained
    };
    let section = if path == "README.md" || path.starts_with("getting-started/") {
        Section::GettingStarted
    } else if path.starts_with("product-ux/") {
        Section::ProductUx
    } else if path.starts_with("learning-practice/") {
        Section::LearningPractice
    } else if path.starts_with("games/") {
        Section::Games
    } else if path.starts_with("dictionary-data/") {
        Section::DictionaryData
    } else if path.starts_with("architecture/") {
        Section::Architecture
    } else if path.starts_with("engineering/") {
        Section::Engineering
    } else if path.starts_with("testing-quality/") {
        Section::TestingQuality
    } else if path.starts_with("build-deployment/") {
        Section::BuildDeployment
    } else if path.starts_with("project-management/") {
        Section::ProjectManagement
    } else if historical {
        Section::ArchiveHistory
    } else if path.starts_with("issues/") {
        Section::Issues
    } else {
        Section::GettingStarted
    };
    let topic = match section {
        Section::GettingStarted => "Overview".into(),
        Section::Issues => {
            let area = path
                .strip_prefix("issues/")
                .unwrap_or(path)
                .split('/')
                .next()
                .unwrap_or("");
            match area {
                "README.md" | "OPEN.md" | "ORIGINAL_REPORT_2026-11-17.md" => {
                    "Index & provenance".into()
                }
                "onboarding" => "Onboarding".into(),
                "practice" => "Practice".into(),
                "vocabulary" => "Vocabulary".into(),
                "courses" => "Courses".into(),
                "games" => "Games".into(),
                "sentences" => "Sentences".into(),
                "translation" => "Translation".into(),
                _ => "Other".into(),
            }
        }
        Section::ProductUx => match path.split('/').nth(1).unwrap_or("") {
            "foundation" => "Foundation".into(),
            "practice" => "Practice interface".into(),
            "pages" => "Pages".into(),
            "references" => "Visual references".into(),
            _ => "Overview".into(),
        },
        Section::DictionaryData => match path.split('/').nth(1).unwrap_or("") {
            "schema" => "Database".into(),
            "personal" => "Personal storage".into(),
            "vocabulary" => "Vocabulary".into(),
            _ => "Dictionary".into(),
        },
        Section::Architecture => {
            if path.starts_with("architecture/modules/") {
                "Components".into()
            } else {
                "System".into()
            }
        }
        Section::Engineering => "Development".into(),
        Section::LearningPractice => match path.split('/').nth(1).unwrap_or("") {
            "hanzi" => "Hanzi writing".into(),
            "courses" => "Courses".into(),
            "translation" => "Sentences".into(),
            "practice" => "Practice".into(),
            _ => "Overview".into(),
        },
        Section::Games => "Game systems".into(),
        Section::TestingQuality => match path.split('/').nth(1).unwrap_or("") {
            "e2e" => "E2E and evidence".into(),
            "acceptance" => "Acceptance and quality".into(),
            _ => "Verification".into(),
        },
        Section::BuildDeployment => {
            if path.starts_with("build-deployment/services/") {
                "Services".into()
            } else {
                "Build and release".into()
            }
        }
        Section::ProjectManagement => {
            if path.starts_with("project-management/decisions/") {
                "Decisions".into()
            } else if path.starts_with("project-management/plans/") {
                "Plans".into()
            } else {
                "Current work".into()
            }
        }
        Section::ArchiveHistory => {
            if path.starts_with("archive-history/completed/") {
                "Completed records".into()
            } else if path.starts_with("archive-history/audits/") {
                "Historical audits".into()
            } else {
                "Release history".into()
            }
        }
    };
    let order = match name {
        "README.md" => 0,
        "OPEN.md" => 1,
        "ORIGINAL_REPORT_2026-11-17.md" => 200,
        "product.md" => 10,
        "ux.md" => 20,
        "architecture.md" => 10,
        "ui-architecture.md" => 20,
        "db.md" => 10,
        "testing.md" => 10,
        "current.md" => 10,
        "tasks.md" => 20,
        "user-reported-issues.md" => 25,
        "technical-debt.md" => 30,
        "release-naming.md" => 10,
        _ => 100,
    };
    DocumentMetadata {
        section,
        topic,
        status,
        order,
    }
}

pub(super) fn category(path: &str) -> String {
    document_metadata(path).section.label().to_owned()
}

pub(super) fn topic(path: &str) -> String {
    document_metadata(path).topic
}

pub(super) fn visible(entry: &DocumentEntry) -> bool {
    entry.markdown && document_metadata(&entry.path).status != DocumentStatus::Internal
}

pub(super) fn matches_search(entry: &DocumentEntry, search: &str) -> bool {
    let search = search.trim().to_lowercase();
    search.is_empty()
        || entry.title.to_lowercase().contains(&search)
        || entry.path.to_lowercase().contains(&search)
        || entry.search_text.to_lowercase().contains(&search)
}

fn search_rank(entry: &DocumentEntry, search: &str) -> u8 {
    if search.is_empty() {
        return 0;
    }
    let name = entry.title.to_lowercase();
    if name == search {
        0
    } else if name.starts_with(search) {
        1
    } else if name.contains(search) {
        2
    } else if entry.path.to_lowercase().contains(search) {
        3
    } else {
        4
    }
}

pub(super) fn group_entries(entries: &[DocumentEntry], search: &str) -> DocumentGroups {
    let search = search.trim().to_lowercase();
    let mut groups = DocumentGroups::new();
    for entry in entries {
        if !visible(entry) || !matches_search(entry, &search) {
            continue;
        }
        let metadata = document_metadata(&entry.path);
        groups
            .entry(metadata.section.label().to_owned())
            .or_default()
            .entry(metadata.topic)
            .or_default()
            .push(entry.clone());
    }
    for topics in groups.values_mut() {
        for documents in topics.values_mut() {
            documents.sort_by(|a, b| {
                search_rank(a, &search)
                    .cmp(&search_rank(b, &search))
                    .then_with(|| {
                        document_metadata(&a.path)
                            .order
                            .cmp(&document_metadata(&b.path).order)
                    })
                    .then_with(|| a.title.cmp(&b.title))
                    .then_with(|| a.path.cmp(&b.path))
            });
        }
    }
    groups
}

pub(super) fn ordered_categories(groups: &DocumentGroups) -> Vec<&String> {
    let mut names: Vec<_> = groups.keys().collect();
    names.sort_by_key(|name| {
        DocumentationSection::ORDER
            .iter()
            .position(|section| section.label() == name.as_str())
            .unwrap_or(usize::MAX)
    });
    names
}

pub(super) fn reading_neighbours<'a>(
    entries: &'a [DocumentEntry],
    current: &str,
) -> (Option<&'a DocumentEntry>, Option<&'a DocumentEntry>) {
    let mut order = entries
        .iter()
        .filter(|entry| entry.markdown && visible(entry))
        .collect::<Vec<_>>();
    order.sort_by(|a, b| {
        let ma = document_metadata(&a.path);
        let mb = document_metadata(&b.path);
        let rank = |section: DocumentationSection| {
            DocumentationSection::ORDER
                .iter()
                .position(|value| *value == section)
                .unwrap_or(usize::MAX)
        };
        rank(ma.section)
            .cmp(&rank(mb.section))
            .then_with(|| ma.topic.cmp(&mb.topic))
            .then_with(|| ma.order.cmp(&mb.order))
            .then_with(|| a.title.cmp(&b.title))
    });
    let Some(index) = order.iter().position(|entry| entry.path == current) else {
        return (None, None);
    };
    (
        index.checked_sub(1).and_then(|i| order.get(i)).copied(),
        order.get(index + 1).copied(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, title: &str, markdown: bool) -> DocumentEntry {
        DocumentEntry {
            path: path.into(),
            title: title.into(),
            markdown,
            search_text: "searchable text".into(),
        }
    }

    #[test]
    fn preserves_routes_while_grouping_by_domain() {
        assert_eq!(
            category("dictionary-data/schema/db.md"),
            "Dictionary & Data"
        );
        assert_eq!(topic("dictionary-data/schema/db.md"), "Database");
        assert_eq!(
            category("archive-history/completed/2026-11-16.md"),
            "Archive & History"
        );
        assert_eq!(
            category("learning-practice/hanzi/write-hanzi-activity.md"),
            "Learning & Practice"
        );
        assert_eq!(
            category("project-management/tasks.md"),
            "Project Management"
        );
        assert_eq!(category("issues/practice/ISS-005-example.md"), "Issues");
        assert_eq!(topic("issues/practice/ISS-005-example.md"), "Practice");
    }

    #[test]
    fn search_ranks_document_titles_first() {
        let entries = vec![
            entry("project-management/tasks.md", "Tasks", true),
            entry("project-management/current.md", "Current", true),
        ];
        let matches = group_entries(&entries, "searchable");
        assert_eq!(
            matches
                .values()
                .flat_map(|group| group.values())
                .map(Vec::len)
                .sum::<usize>(),
            2
        );
        let matching = group_entries(&entries, "tasks");
        assert_eq!(
            matching["Project Management"]["Current work"][0].title,
            "Tasks"
        );
    }

    #[test]
    fn reading_order_uses_navigation_instead_of_input_order() {
        let entries = vec![
            entry("project-management/tasks.md", "Tasks", true),
            entry("README.md", "Home", true),
            entry("logo.png", "Artwork", false),
        ];
        let (previous, next) = reading_neighbours(&entries, "project-management/tasks.md");
        assert_eq!(previous.map(|entry| entry.path.as_str()), Some("README.md"));
        assert!(next.is_none());
    }

    #[test]
    fn current_lowercase_document_names_keep_curated_order() {
        assert_eq!(document_metadata("product-ux/product.md").order, 10);
        assert_eq!(document_metadata("product-ux/ux.md").order, 20);
        assert_eq!(document_metadata("project-management/current.md").order, 10);
        assert_eq!(document_metadata("project-management/tasks.md").order, 20);
        assert_eq!(
            document_metadata("testing-quality/acceptance/testing.md").order,
            10
        );
    }

    #[test]
    fn section_order_is_curated() {
        let entries = [
            entry("project-management/tasks.md", "Tasks", true),
            entry("README.md", "Home", true),
        ];
        let groups = group_entries(&entries, "");
        let names = ordered_categories(&groups);
        assert_eq!(names[0].as_str(), "Getting started");
        assert_eq!(names[1].as_str(), "Project Management");
    }
}
