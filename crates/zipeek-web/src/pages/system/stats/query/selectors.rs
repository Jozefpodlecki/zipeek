use super::*;

fn search_term(search: &str) -> Result<String, QueryError> {
    if search.len() > MAX_QUERY_LENGTH {
        return Err(QueryError::InvalidSearch);
    }
    Ok(search.trim().to_lowercase())
}

fn slice_page<T: Clone>(rows: Vec<&T>, request: PageRequest) -> QueryPage<T> {
    let total = rows.len();
    let records = rows
        .into_iter()
        .skip(request.offset)
        .take(request.limit)
        .cloned()
        .collect();
    QueryPage {
        records,
        total,
        offset: request.offset,
        limit: request.limit,
    }
}

fn skill_evidence(record: &WordRecord, skill: SkillGap) -> EvidenceStatus {
    match skill {
        SkillGap::Recognition => record.recognition,
        SkillGap::Listening => record.listening,
        SkillGap::Speaking => record.speaking,
        SkillGap::Writing => record.writing,
        SkillGap::Sentence => record.sentence,
    }
}

fn needs_review(record: &WordRecord) -> bool {
    SkillGap::ALL
        .into_iter()
        .any(|skill| skill_evidence(record, skill) == EvidenceStatus::NeedsReview)
}

pub(in super::super) fn query_words(
    rows: &[WordRecord],
    query: &WordQuery,
) -> Result<QueryPage<WordRecord>, QueryError> {
    let search = search_term(&query.search)?;
    let mut matching: Vec<_> = rows
        .iter()
        .filter(|row| {
            (search.is_empty()
                || row.hanzi.contains(&search)
                || row.pinyin.to_lowercase().contains(&search)
                || row.meaning.to_lowercase().contains(&search))
                && query
                    .gap
                    .is_none_or(|skill| skill_evidence(row, skill) != EvidenceStatus::Observed)
        })
        .collect();
    match query.sort {
        WordSort::Alphabetical => {
            matching.sort_by(|a, b| a.hanzi.cmp(&b.hanzi).then(a.id.cmp(&b.id)))
        }
        WordSort::AlphabeticalReverse => {
            matching.sort_by(|a, b| b.hanzi.cmp(&a.hanzi).then(a.id.cmp(&b.id)))
        }
        WordSort::Encounters => matching.sort_by(|a, b| {
            b.encounters
                .cmp(&a.encounters)
                .then(a.hanzi.cmp(&b.hanzi))
                .then(a.id.cmp(&b.id))
        }),
        WordSort::FewestEncounters => matching.sort_by(|a, b| {
            a.encounters
                .cmp(&b.encounters)
                .then(a.hanzi.cmp(&b.hanzi))
                .then(a.id.cmp(&b.id))
        }),
        WordSort::Review => matching.sort_by(|a, b| {
            needs_review(b)
                .cmp(&needs_review(a))
                .then(a.hanzi.cmp(&b.hanzi))
                .then(a.id.cmp(&b.id))
        }),
    }
    Ok(slice_page(matching, query.page))
}

pub(in super::super) fn query_grammar(
    rows: &[GrammarRecord],
    query: &GrammarQuery,
) -> Result<QueryPage<GrammarRecord>, QueryError> {
    let search = search_term(&query.search)?;
    let mut matching: Vec<_> = rows
        .iter()
        .filter(|row| {
            query.region.contains(&row.id)
                && (search.is_empty()
                    || row.pattern.to_lowercase().contains(&search)
                    || row.description.to_lowercase().contains(&search)
                    || row.course.to_lowercase().contains(&search)
                    || row.example.contains(&search)
                    || row.id.to_lowercase().contains(&search))
        })
        .collect();
    match query.sort {
        GrammarSort::TeachingOrder => matching.sort_by(|a, b| a.id.cmp(&b.id)),
        GrammarSort::PatternAscending => {
            matching.sort_by(|a, b| a.pattern.cmp(&b.pattern).then(a.id.cmp(&b.id)))
        }
        GrammarSort::PatternDescending => {
            matching.sort_by(|a, b| b.pattern.cmp(&a.pattern).then(a.id.cmp(&b.id)))
        }
        GrammarSort::MostAttempts => {
            matching.sort_by(|a, b| b.attempts.cmp(&a.attempts).then(a.id.cmp(&b.id)))
        }
        GrammarSort::FewestAttempts => {
            matching.sort_by(|a, b| a.attempts.cmp(&b.attempts).then(a.id.cmp(&b.id)))
        }
    }
    Ok(slice_page(matching, query.page))
}

pub(in super::super) fn query_history(
    rows: &[SessionRecord],
    query: &HistoryQuery,
) -> Result<QueryPage<SessionRecord>, QueryError> {
    let search = search_term(&query.search)?;
    let mut matching: Vec<_> = rows
        .iter()
        .filter(|row| {
            query.category.matches(&row.category)
                && (search.is_empty()
                    || row.date.contains(&search)
                    || row.started_at.contains(&search)
                    || row.activity_title.to_lowercase().contains(&search)
                    || row.activity_id.to_lowercase().contains(&search)
                    || row.focus.to_lowercase().contains(&search))
        })
        .collect();
    matching.sort_by(|a, b| b.started_at.cmp(&a.started_at).then(a.id.cmp(&b.id)));
    Ok(slice_page(matching, query.page))
}

pub(in super::super) fn query_activity(
    rows: &[SessionRecord],
    query: &ActivityQuery,
) -> Result<QueryPage<ActivityBucket>, QueryError> {
    let search = search_term(&query.search)?;
    let records = query
        .dates
        .iter()
        .map(|date| {
            let mut practice = 0;
            let mut games = 0;
            for record in rows.iter().filter(|record| {
                record.date == date.as_str()
                    && query.category.matches(&record.category)
                    && (search.is_empty()
                        || record.date.contains(&search)
                        || record.activity_title.to_lowercase().contains(&search)
                        || record.activity_id.to_lowercase().contains(&search)
                        || record.focus.to_lowercase().contains(&search))
            }) {
                if record.category == "Practice" {
                    practice += 1;
                } else {
                    games += 1;
                }
            }
            ActivityBucket {
                date: date.clone(),
                count: practice + games,
                practice,
                games,
            }
        })
        .collect();
    Ok(QueryPage {
        records,
        total: query.dates.len(),
        offset: 0,
        limit: query.dates.len(),
    })
}

pub(in super::super) async fn matching_word_ids(
    source: &dyn StatisticsQuerySource,
    mut query: WordQuery,
    max: usize,
) -> Result<Vec<String>, QueryError> {
    let mut ids = Vec::new();
    let mut offset = 0;
    loop {
        query.page = PageRequest {
            offset,
            limit: MAX_QUERY_PAGE_SIZE,
        };
        let page = source.words(query.clone()).await?;
        page.validate(query.page)?;
        if page.total > max {
            return Err(QueryError::TooManyMatches);
        }
        let count = page.records.len();
        ids.extend(page.records.into_iter().map(|item| item.id));
        offset += count;
        if offset >= page.total || count == 0 {
            break;
        }
    }
    if ids.iter().collect::<std::collections::BTreeSet<_>>().len() != ids.len() {
        return Err(QueryError::InconsistentPage);
    }
    Ok(ids)
}

pub(in super::super) async fn matching_grammar_ids(
    source: &dyn StatisticsQuerySource,
    mut query: GrammarQuery,
    max: usize,
) -> Result<Vec<String>, QueryError> {
    let mut ids = Vec::new();
    let mut offset = 0;
    loop {
        query.page = PageRequest {
            offset,
            limit: MAX_QUERY_PAGE_SIZE,
        };
        let page = source.grammar(query.clone()).await?;
        page.validate(query.page)?;
        if page.total > max {
            return Err(QueryError::TooManyMatches);
        }
        let count = page.records.len();
        ids.extend(page.records.into_iter().map(|item| item.id));
        offset += count;
        if offset >= page.total || count == 0 {
            break;
        }
    }
    if ids.iter().collect::<std::collections::BTreeSet<_>>().len() != ids.len() {
        return Err(QueryError::InconsistentPage);
    }
    Ok(ids)
}
