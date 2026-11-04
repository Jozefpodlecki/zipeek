const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const read = path => readFileSync(join(__dirname, '..', path), 'utf8');

test('practice setup uses the typed exercise draft and unlocks conditional settings', () => {
  const source = read('crates/web/src/pages/practice_setup.rs');
  const builder = read('crates/web/src/components/practice_builder.rs');
  const draft = read('crates/zipeek-exercises/src/practice_draft.rs');
  assert.match(source, /use_state\(move \|\| \{/);
  assert.match(source, /<PracticeBuilder section=/);
  assert.match(source, /on_toggle_exercise/);
  assert.match(source, /draft\.has_exercise\("sentence-translation"\)/);
  assert.match(source, /draft\.has_exercise\("speaking"\)/);
  assert.match(builder, /data-practice-tabs/);
  assert.match(builder, /data-practice-tab=/);
  assert.match(builder, /data-practice-config-tabs/);
  assert.match(builder, /props\.draft\.configuration_tabs\(\)/);
  assert.match(builder, /aria-pressed=/);
  assert.match(draft, /pub fn configuration_tabs/);
  assert.match(source, /PracticeVocabularySource/);
  const sourcePicker = read('crates/web/src/components/practice_vocabulary_source.rs');
  assert.match(sourcePicker, /data-vocabulary-hsk-multi/);
  assert.match(sourcePicker, /data-vocabulary-set/);
  assert.match(source, /data-speaking-setup/);
});

test('translation generates from selected local HSK or Word Set vocabulary', () => {
  const selection = read('crates/web/src/services/translation_vocabulary.rs');
  const database = read('crates/web/src/services/database/queries.rs');
  const ai = read('crates/zipeek-ai/src/lib.rs');
  const actions = read('crates/web/src/pages/sentence_translation/actions.rs');
  assert.match(selection, /VocabularySource::Hsk/);
  assert.match(selection, /VocabularySource::Saved/);
  assert.match(selection, /words_by_ids\(&ids\)/);
  assert.match(database, /translation_words_for_hsk/);
  assert.match(ai, /parse_sentence_for_vocabulary/);
  assert.match(ai, /target_words\.iter\(\).*set\.vocabulary\.contains\(word\)/);
  assert.match(actions, /translation_vocabulary::load/);
  assert.match(actions, /parse_sentence_for_vocabulary\(&raw, &vocabulary, direction\)/);
});

test('unstarted session banner stays out of Browse and Home', () => {
  const status = read('crates/web/src/components/quiz_session_provider.rs');
  const shell = read('crates/web/src/components/app_shell.rs');
  assert.match(status, /let visible = completed > 0/);
  assert.match(shell, /Route::SentenceTranslation \| Route::Speaking/);
  assert.match(shell, /QuizSessionStatus/);
});

test('functional browse surfaces keep the background behind legible panels', () => {
  const search = read('crates/web/src/components/browse/search_form.rs');
  const results = read('crates/web/src/components/browse/results/mod.rs');
  const backdrop = read('crates/web/styles/inline.css');
  assert.match(search, /bg-surface/);
  assert.match(results, /bg-surface/);
  assert.match(backdrop, /linear-gradient/);
});
