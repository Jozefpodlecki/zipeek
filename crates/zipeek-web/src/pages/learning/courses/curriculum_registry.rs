use super::curriculum::{GrammarCourse, authored_grammar_region};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyllabusStatus {
    PendingIndependentVerification,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeachingStage {
    BeginnerFoundation,
    BeginnerExtension,
}

#[derive(Clone, Copy, Debug)]
pub struct CourseRegistryEntry {
    pub region: usize,
    pub id: &'static str,
    pub prerequisites: &'static [&'static str],
    pub point_ids: &'static [&'static str],
    pub stage: TeachingStage,
    pub syllabus: SyllabusStatus,
}

impl CourseRegistryEntry {
    pub fn teaching_stage_label(&self) -> &'static str {
        match self.stage {
            TeachingStage::BeginnerFoundation => "Beginner foundations (local sequence)",
            TeachingStage::BeginnerExtension => "Beginner extension (local sequence)",
        }
    }

    pub fn syllabus_status_label(&self) -> &'static str {
        match self.syllabus {
            SyllabusStatus::PendingIndependentVerification => {
                "Official HSK syllabus mapping: pending independent verification."
            }
        }
    }
}

pub const GRAMMAR_REGISTRY: [CourseRegistryEntry; 5] = [
    CourseRegistryEntry {
        region: 1,
        id: "grammar-1",
        prerequisites: &[],
        point_ids: &[
            "g01-subject-predicate",
            "g01-shi-identity",
            "g01-bu-negation",
            "g01-ma-question",
        ],
        stage: TeachingStage::BeginnerFoundation,
        syllabus: SyllabusStatus::PendingIndependentVerification,
    },
    CourseRegistryEntry {
        region: 2,
        id: "grammar-2",
        prerequisites: &["grammar-1"],
        point_ids: &[
            "g02-time-placement",
            "g02-zai-location",
            "g02-time-location",
            "g02-qu-destination",
            "g02-nali-question",
        ],
        stage: TeachingStage::BeginnerExtension,
        syllabus: SyllabusStatus::PendingIndependentVerification,
    },
    CourseRegistryEntry {
        region: 3,
        id: "grammar-3",
        prerequisites: &["grammar-1"],
        point_ids: &[
            "g03-you-possession",
            "g03-meiyou-negation",
            "g03-classifiers",
            "g03-ji-count",
            "g03-duoshao-quantity",
        ],
        stage: TeachingStage::BeginnerExtension,
        syllabus: SyllabusStatus::PendingIndependentVerification,
    },
    CourseRegistryEntry {
        region: 4,
        id: "grammar-4",
        prerequisites: &["grammar-1"],
        point_ids: &[
            "g04-adjective-predicate",
            "g04-hen-degree",
            "g04-bu-adjective",
            "g04-tai-le",
            "g04-bi-comparison",
        ],
        stage: TeachingStage::BeginnerExtension,
        syllabus: SyllabusStatus::PendingIndependentVerification,
    },
    CourseRegistryEntry {
        region: 5,
        id: "grammar-5",
        prerequisites: &["grammar-1"],
        point_ids: &[
            "g05-xiang-intent",
            "g05-yao-want",
            "g05-hui-learned-ability",
            "g05-neng-possibility",
            "g05-keyi-permission",
        ],
        stage: TeachingStage::BeginnerExtension,
        syllabus: SyllabusStatus::PendingIndependentVerification,
    },
];

pub fn registry_entry(region: usize) -> Option<&'static CourseRegistryEntry> {
    GRAMMAR_REGISTRY.iter().find(|entry| entry.region == region)
}

pub fn registry_course(id: &str) -> Option<&'static GrammarCourse> {
    GRAMMAR_REGISTRY
        .iter()
        .find(|entry| entry.id == id)
        .and_then(|entry| authored_grammar_region(entry.region).map(|(course, _)| course))
}

#[derive(Debug, PartialEq, Eq)]
pub enum RegistryError {
    DuplicateRegion(usize),
    DuplicateCourseId(&'static str),
    DuplicatePointId(&'static str),
    UnauthoredCourse(usize),
    CourseIdMismatch(usize),
    PointCountMismatch(usize),
    EmptyIdentifier(usize),
    InvalidPrerequisite(&'static str),
    SelfPrerequisite(&'static str),
    ForwardPrerequisite(&'static str),
}

pub fn validate_registry(entries: &[CourseRegistryEntry]) -> Result<(), RegistryError> {
    let mut regions = BTreeSet::new();
    let mut courses = BTreeSet::new();
    let mut points = BTreeSet::new();
    for entry in entries {
        if !regions.insert(entry.region) {
            return Err(RegistryError::DuplicateRegion(entry.region));
        }
        if entry.id.is_empty() || entry.point_ids.iter().any(|id| id.is_empty()) {
            return Err(RegistryError::EmptyIdentifier(entry.region));
        }
        if !courses.insert(entry.id) {
            return Err(RegistryError::DuplicateCourseId(entry.id));
        }
        let (course, checks) = authored_grammar_region(entry.region)
            .ok_or(RegistryError::UnauthoredCourse(entry.region))?;
        if course.id != entry.id {
            return Err(RegistryError::CourseIdMismatch(entry.region));
        }
        if entry.point_ids.len() != course.points.len() || checks.len() != course.points.len() {
            return Err(RegistryError::PointCountMismatch(entry.region));
        }
        for &id in entry.point_ids {
            if !points.insert(id) {
                return Err(RegistryError::DuplicatePointId(id));
            }
        }
    }
    for entry in entries {
        for &required in entry.prerequisites {
            if required == entry.id {
                return Err(RegistryError::SelfPrerequisite(entry.id));
            }
            let prerequisite = entries
                .iter()
                .find(|candidate| candidate.id == required)
                .ok_or(RegistryError::InvalidPrerequisite(required))?;
            if prerequisite.region >= entry.region {
                return Err(RegistryError::ForwardPrerequisite(entry.id));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_registry_has_stable_unique_points_and_valid_prerequisites() {
        assert_eq!(validate_registry(&GRAMMAR_REGISTRY), Ok(()));
        assert_eq!(
            GRAMMAR_REGISTRY
                .iter()
                .map(|course| course.point_ids.len())
                .sum::<usize>(),
            24
        );
        assert_eq!(registry_entry(3).unwrap().prerequisites, &["grammar-1"]);
        assert_eq!(registry_course("grammar-2").unwrap().points.len(), 5);
        assert_eq!(registry_entry(4).unwrap().id, "grammar-4");
        assert_eq!(registry_entry(5).unwrap().id, "grammar-5");
        assert!(registry_entry(6).is_none());
        assert!(
            GRAMMAR_REGISTRY
                .iter()
                .all(|entry| entry.syllabus == SyllabusStatus::PendingIndependentVerification)
        );
    }

    #[test]
    fn duplicated_point_ids_are_rejected() {
        let mut entries = GRAMMAR_REGISTRY;
        entries[1].point_ids = &[
            "g01-subject-predicate",
            "g02-zai-location",
            "g02-time-location",
            "g02-qu-destination",
            "g02-nali-question",
        ];
        assert_eq!(
            validate_registry(&entries),
            Err(RegistryError::DuplicatePointId("g01-subject-predicate"))
        );
    }

    #[test]
    fn forward_and_missing_prerequisites_are_rejected() {
        let mut entries = GRAMMAR_REGISTRY;
        entries[0].prerequisites = &["grammar-2"];
        assert_eq!(
            validate_registry(&entries),
            Err(RegistryError::ForwardPrerequisite("grammar-1"))
        );
        entries[0].prerequisites = &["grammar-99"];
        assert_eq!(
            validate_registry(&entries),
            Err(RegistryError::InvalidPrerequisite("grammar-99"))
        );
    }

    #[test]
    fn mismatched_course_id_and_point_count_are_rejected() {
        let mut entries = GRAMMAR_REGISTRY;
        entries[1].id = "another-id";
        assert_eq!(
            validate_registry(&entries),
            Err(RegistryError::CourseIdMismatch(2))
        );
        entries[1] = GRAMMAR_REGISTRY[1];
        entries[1].point_ids = &["g02-time-placement"];
        assert_eq!(
            validate_registry(&entries),
            Err(RegistryError::PointCountMismatch(2))
        );
    }
}
