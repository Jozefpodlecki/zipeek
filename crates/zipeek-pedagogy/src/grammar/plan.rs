pub const GRAMMAR_REGION_PLAN: [&str; 100] = [
    "Basic Sentence Structure",
    "Time, Place & Everyday Actions",
    "Possession, Existence & Quantities",
    "Descriptions & Comparisons",
    "Wants, Abilities & Polite Requests",
    "Aspect: completed events",
    "Experience and ongoing states",
    "Duration and frequency",
    "Location and existence",
    "Direction and movement",
    "Result complements",
    "Possibility complements",
    "Degree and manner complements",
    "The 把 construction",
    "Passive constructions",
    "Noun modification and 的",
    "Topic and emphasis",
    "Conditionals and cause",
    "Contrast and concession",
    "Sequence and complex actions",
    "Serial-verb sequences",
    "Pivotal constructions with 让 and 请",
    "Directional complements: compound forms",
    "Result complements: advanced usage",
    "Potential complements: extended patterns",
    "Quantifying events and actions",
    "Verb reduplication",
    "Adjective reduplication",
    "Existential sentences with 着",
    "Existential sentences with 有",
    "Comparison with 没有",
    "Comparison with 一样",
    "Comparison with 越来越",
    "Comparison with 越 ... 越 ...",
    "Comparison of quantities",
    "Expressing approximate numbers",
    "Fractions and percentages",
    "Numbers and measurements",
    "Classifiers for people and objects",
    "Verb measure words",
    "Demonstratives and deixis",
    "Interrogative pronouns",
    "Indefinite pronouns",
    "Reflexives and reciprocity",
    "Noun phrase coordination",
    "Complex prepositional phrases",
    "Temporal connectors",
    "Spatial and direction locatives",
    "Relative time and sequence",
    "Aspect in extended narratives",
    "Aspect with duration",
    "Experiential aspect variations",
    "Negation in complex sentences",
    "Modal verbs: obligation",
    "Modal verbs: probability",
    "Epistemic and deontic modality",
    "Choice and alternative questions",
    "Rhetorical questions",
    "A-not-A questions",
    "Embedded questions",
    "Requests and imperatives",
    "Suggestions and invitations",
    "Polite refusal and mitigation",
    "Sentential particles 吧 and 呢",
    "Sentential particles 啊 and 呀",
    "Emphatic and explanatory particles",
    "Sentence-final 了: nuances",
    "Discourse functions of 是",
    "Existential and presentational clauses",
    "Topic chains in discourse",
    "Focus and contrast structures",
    "Cleft and pseudo-cleft structures",
    "Reported speech",
    "Quoting and evidentiality",
    "Purposes and intentions",
    "Results and consequences",
    "Causatives and agency",
    "Passive alternatives and affectedness",
    "把 with multiple complements",
    "Giving and transfer constructions",
    "Benefactive constructions",
    "Comitative and instrumental phrases",
    "Comparison of alternatives",
    "Paired conjunctions",
    "Concessive conditions",
    "Counterfactual scenarios",
    "Hypothetical reasoning",
    "Logical relations in argument",
    "Formal written connectors",
    "Formal noun modification",
    "Nominalization and abstractions",
    "Economy and ellipsis",
    "Cohesion and reference",
    "Information sequencing",
    "Pragmatics: politeness and stance",
    "Pragmatics: emphasis and implication",
    "Register and formality",
    "Integrated narrative structures",
    "Integrated argumentative structures",
    "Grammar consolidation and review",
];

pub fn planned_title(region: usize) -> Option<&'static str> {
    region.checked_sub(1).and_then(|index| GRAMMAR_REGION_PLAN.get(index).copied())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exactly_one_hundred_named_regions() {
        assert_eq!(GRAMMAR_REGION_PLAN.len(), 100);
        assert!(GRAMMAR_REGION_PLAN.iter().all(|title| !title.is_empty()));
        assert_eq!(planned_title(100), Some("Grammar consolidation and review"));
        let mut labels = std::collections::BTreeSet::new();
        let mut examples = std::collections::BTreeSet::new();
        let mut total = 0;
        for region in 1..=100 {
            let course = if region <= 5 {
                super::super::curriculum::authored_grammar_region(region)
                    .map(|(course, _)| course)
            } else {
                super::super::expansion::expanded_course(region)
            }
            .unwrap_or_else(|| panic!("Missing authored Grammar course for region {region}"));
            assert_eq!(course.id, format!("grammar-{region}"));
            for point in course.points {
                total += 1;
                assert!(labels.insert(point.label), "Duplicate pattern label: {}", point.label);
                assert!(examples.insert(point.example), "Duplicate example: {}", point.example);
                assert!(!point.explanation.is_empty());
                assert!(!point.translation.is_empty());
            }
        }
        assert_eq!(total, 550);
    }
}
