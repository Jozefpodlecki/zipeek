const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');

const root = join(__dirname, '..');
const read = (path) => readFileSync(join(root, path), 'utf8');

const translationFiles = ['state.rs', 'settings.rs', 'actions.rs', 'view.rs'];
const translationSource = () => [read('crates/web/src/pages/sentence_translation.rs'), ...translationFiles.map(file => read(`crates/web/src/pages/sentence_translation/${file}`))].join('\n');

test('word-bank exercise has distinct boundaries and keyboard-accessible tile buttons', () => {
  const view = read('crates/web/src/components/sentence_word_bank.rs');
  assert.match(view, /data-component="sentence-word-bank"/);
  assert.match(view, /data-translation-assembly/);
  assert.match(view, /data-translation-word-bank/);
  assert.match(view, /START/);
  assert.match(view, /END/);
  assert.match(view, /ondrop=/);
  assert.match(view, /ondragstart=/);
  assert.match(view, /on_reorder/);
  assert.match(view, /<button/);
});

test('sentence page reshuffles tiles and submits only the selected order', () => {
  const page = translationSource();
  assert.match(page, /Math::random/);
  const instances = read('crates/web/src/services/translation_instance.rs');
  assert.match(page, /translation_instance::shuffled_bank_order/);
  assert.match(instances, /order\.swap/);
  assert.match(page, /assembled_answer_with_separator\(/);
  assert.match(page, /is_complete\(&state\.selected_tiles, sentence\.word_bank\.len\(\)\)/);
  assert.match(page, /SentenceWordBank/);
  assert.doesNotMatch(page, /HtmlTextAreaElement/);
});

test('prompt roles and classification cache version stay isolated', () => {
  const classifier = read('crates/db-gen/src/llm/prompts/classification.rs');
  assert.match(classifier, /sense-classification-v3/);
  const generator = read('crates/zipeek-ai/src/prompts/sentence_generation.rs');
  const evaluator = read('crates/zipeek-ai/src/prompts/evaluation.rs');
  const simulator = read('crates/zipeek-ai/src/prompts/quiz_simulation.rs');
  for (const source of [generator, evaluator, simulator]) {
    assert.match(source, /pub const SYSTEM: &str = r#"/);
    assert.match(source, /pub const VERSION/);
  }
  assert.match(simulator, /test harness owns/);
});

test('pure sentence tile transitions are isolated in core and covered by native tests', () => {
  const domain = read('crates/zipeek-core/src/sentence_tiles.rs');
  const page = translationSource();
  assert.match(domain, /pub fn apply_tile_move/);
  assert.match(domain, /pub fn is_complete/);
  assert.match(domain, /pub fn assembled_answer/);
  assert.match(domain, /repeated_word_text_uses_distinct_tile_ids/);
  assert.match(domain, /completeness_rejects_duplicates_and_invalid_indices/);
  assert.match(page, /TileMove::Add/);
  assert.match(page, /TileMove::Before/);
});

test('db-gen batch test cleanup references the crate cache-store module', () => {
  const batch = read('crates/db-gen/src/llm/batch.rs');
  assert.doesNotMatch(batch, /super::cache_store::records_directory/);
  assert.match(batch, /crate::llm::cache_store::records_directory/);
});

test('async translation requests ignore stale AI results after configuration changes', () => {
  const page = translationSource();
  assert.match(page, /use_mut_ref\(\|\| 0_u64\)/);
  assert.match(page, /if \*task_state\.request_epoch\.borrow\(\) != request_id/);
  assert.match(page, /if \*task_state\.request_epoch\.borrow\(\) == request_id/g);
  assert.match(page, /let on_set = \{/);
  assert.match(page, /state\.reset_configuration\(\)/);
  assert.match(page, /let on_model = \{/);
  assert.match(page, /let on_endpoint = \{/);
});

test('selected word bank tiles have keyboard-usable movement controls', () => {
  const view = read('crates/web/src/components/sentence_word_bank.rs');
  assert.match(view, /data-tile-move-left/);
  assert.match(view, /data-tile-move-right/);
  assert.match(view, /Move \{\} left/);
  assert.match(view, /Move \{\} right/);
  assert.match(view, /move_left\.is_none\(\)/);
  assert.match(view, /move_right\.is_none\(\)/);
});

test('translation has an explicit seedable mixed direction and a direction-bound JSON contract', () => {
  const domain = read('crates/zipeek-exercises/src/translation.rs');
  const page = translationSource();
  const generator = read('crates/zipeek-ai/src/prompts/sentence_generation.rs');
  const parser = read('crates/zipeek-ai/src/lib.rs');
  assert.match(domain, /enum DirectionSelection/);
  assert.match(domain, /exercise_index % 2/);
  assert.match(page, /data-translation-direction/);
  assert.match(page, /parse_sentence_for_vocabulary/);
  assert.match(parser, /parse_sentence_for_direction/);
  assert.match(page, /evaluation_messages_for_direction/);
  assert.match(generator, /The user request supplies exactly one translation direction/);
  assert.match(parser, /sentence.direction != direction.code\(\)/);
  assert.match(parser, /sentence.solution.concat\(\)/);
});
