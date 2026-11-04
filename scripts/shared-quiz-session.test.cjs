const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const read = path => readFileSync(join(__dirname, '..', path), 'utf8');

const translationFiles = ['state.rs', 'settings.rs', 'actions.rs', 'view.rs'];
const translationSource = () => [read('crates/web/src/pages/sentence_translation.rs'), ...translationFiles.map(file => read(`crates/web/src/pages/sentence_translation/${file}`))].join('\n');

test('quiz session provider wraps the router rather than an exercise route', () => {
  const app = read('crates/web/src/app.rs');
  const provider = app.indexOf('<QuizSessionProvider>');
  const router = app.indexOf('<HashRouter>');
  const shell = app.indexOf('<AppShell>');
  assert.ok(provider >= 0 && provider < router && router < shell);
  assert.ok(app.indexOf('</HashRouter>') < app.indexOf('</QuizSessionProvider>'));
});

test('shared session reducer accepts finalized results with stable source IDs and abandonment', () => {
  const provider = read('crates/web/src/components/quiz_session_provider.rs');
  assert.match(provider, /use_reducer_eq\(SharedQuizState::default\)/);
  assert.match(provider, /SharedQuizAction::Submit/);
  assert.match(provider, /next\.session\.record_completed\(instance, result\)/);
  assert.match(provider, /next\.session\.abandon\(&instance_id\)/);
  assert.match(provider, /pub\(crate\) fn allocate_scope/);
  assert.doesNotMatch(provider, /instance\.id = format!/);
  assert.match(provider, /SharedQuizAction::Finish/);
  assert.match(provider, /SharedQuizAction::Restart/);
});

test('translation keeps pending generated exercise local, submitting after AI evaluation', () => {
  const page = translationSource();
  assert.match(page, /let pending_instance = use_mut_ref/);
  assert.match(page, /parse_evaluation\(&raw\)/);
  assert.match(page, /quiz_session\.submit\(instance, ExerciseResult/);
  assert.doesNotMatch(page, /quiz_session\.set\(/);
});

test('matching and MCQ publish only after the round completes', () => {
  const hook = read('crates/web/src/hooks/session.rs');
  assert.match(hook, /if !ready \|\| !state\.active\.complete\(\)/);
  assert.match(hook, /publish_round\(&shared_session, &state, &instance_scope\)/);
  assert.match(hook, /instance\.id = format!/);
  assert.match(hook, /shared\.submit\(instance, result\)/);
  assert.match(hook, /state\.active\.stats\(\)/);
});

test('shared completed-result submission is transactional in the pure domain', () => {
  const session = read('crates/zipeek-exercises/src/session.rs');
  assert.match(session, /pub fn record_completed\(/);
  assert.match(session, /fn completed_result_submission_is_atomic_on_validation_failure\(/);
  assert.match(session, /fn completed_submission_refuses_pending_exercises_without_mutation\(/);
});

test('abandoned exercises cannot later score and carry no proficiency evidence', () => {
  const domain = read('crates/zipeek-exercises/src/session.rs');
  const provider = read('crates/web/src/components/quiz_session_provider.rs');
  const translation = translationSource();
  assert.match(domain, /pub abandoned: Vec<String>/);
  assert.match(domain, /pub fn abandon\(&mut self, instance_id: &str\)/);
  assert.match(domain, /SessionError::AbandonedInstance/);
  assert.match(domain, /fn abandoning_is_idempotent_terminal_and_unscored/);
  assert.match(provider, /SharedQuizAction::Abandon\(generation, instance_id\)/);
  assert.match(translation, /quiz_session\.abandon\(instance\.id\)/);
  assert.match(translation, /pub fn invalidate_request\(&self\)/);
  assert.match(translation, /controller\.invalidate_request\(\)/);
});

test('practice publishes result before attempting next route navigation', () => {
  const hook = read('crates/web/src/hooks/session.rs');
  const advance = hook.slice(hook.indexOf('let target_key = SessionKey::new(target, query.clone()'), hook.indexOf('let restart = {'));
  assert.ok(advance.indexOf('publish_round(') < advance.indexOf('.push_with_query('));
});
