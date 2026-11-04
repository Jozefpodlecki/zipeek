const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const read = file => readFileSync(join(__dirname, '..', file), 'utf8');

const translationFiles = ['state.rs', 'settings.rs', 'actions.rs', 'view.rs'];
const translationSource = () => [read('crates/web/src/pages/sentence_translation.rs'), ...translationFiles.map(file => read(`crates/web/src/pages/sentence_translation/${file}`))].join('\n');

test('translation registers a visible-only, validated exercise instance', () => {
  const page = translationSource();
  const instances = read('crates/web/src/services/translation_instance.rs');
  assert.match(page, /translation_instance::visible_instance/);
  assert.match(page, /pending_instance\.borrow_mut\(\) = Some\(instance\)/);
  assert.match(read('crates/web/src/components/quiz_session_provider.rs'), /next\.session\.record_completed\(instance, result\)/);
  assert.match(instances, /ExerciseInstance::new\(instance_id, "sentence-translation", visible_input\)/);
  assert.match(instances, /"source": source/);
  assert.match(instances, /"tiles": visible_tiles/);
  assert.match(instances, /"direction": direction\.code\(\)/);
  const payload = instances.slice(instances.indexOf('let visible_input ='), instances.indexOf('ExerciseInstance::new'));
  assert.doesNotMatch(payload, /sentence\.solution|reference|correct_id|word_id/);
  assert.match(instances, /InvalidTileOrder/);
  assert.match(instances, /reverse_direction_hides_chinese_reference_sentence/);
});

test('translation records authoritative result after evaluation, not on tile placement', () => {
  const page = translationSource();
  assert.match(page, /parse_evaluation\(&raw\)/);
  assert.match(page, /quiz_session\.submit\(instance, ExerciseResult/);
  assert.match(read('crates/web/src/components/quiz_session_provider.rs'), /next\.session\.record_completed\(instance, result\)/);
  assert.match(page, /earned: u64::from\(correct\)/);
  assert.match(page, /mistakes: u64::from\(!correct\)/);
  assert.match(page, /data-component="translation-session-stats"/);
  assert.match(page, /quiz_session\.session\(\)\.totals\(\)/);
});

test('changes to translation configuration invalidate local exercise without erasing shared results', () => {
  const page = translationSource();
  assert.equal((page.match(/quiz_session\.abandon\(instance\.id\);/g) || []).length, 1);
  assert.equal((page.match(/state\.reset_configuration\(\);/g) || []).length, 4);
  assert.doesNotMatch(page, /quiz_session\.restart\(\)/);
  assert.doesNotMatch(page, /localStorage|sessionStorage/);
});

test('visual-only waveform consumes amplitude samples and has no microphone side effects', () => {
  const waveform = read('crates/web/src/components/audio_waveform.rs');
  assert.match(waveform, /pub levels: Vec<f32>/);
  assert.match(waveform, /data-component="audio-waveform"/);
  assert.match(waveform, /bar_height\(level\)/);
  assert.match(waveform, /level\.clamp\(0\.0, 1\.0\)/);
  assert.doesNotMatch(waveform, /getUserMedia|MediaRecorder|SpeechRecognition|fetch\(/);
});

test('translation configuration resets direction index and selected tiles together', () => {
  const page = translationSource();
  assert.equal((page.match(/\*self\.exercise_index\.borrow_mut\(\) = 0;/g) || []).length, 1);
  assert.equal((page.match(/self\.selected_tiles\.set\(Vec::new\(\)\);/g) || []).length, 1);
  assert.equal((page.match(/self\.bank_order\.set\(Vec::new\(\)\);/g) || []).length, 1);
});

test('translation instance IDs remain monotonic when the mixed direction sequence resets', () => {
  const page = translationSource();
  assert.match(page, /let next_instance_id = use_mut_ref\(\|\| 0_u64\)/);
  assert.match(page, /"\{\}:translation-\{\}"/);
  assert.match(page, /\*task_state\.next_instance_id\.borrow_mut\(\) \+= 1;/);
  assert.equal((page.match(/\*self\.exercise_index\.borrow_mut\(\) = 0;/g) || []).length, 1);
  assert.doesNotMatch(page, /\*self\.next_instance_id\.borrow_mut\(\) = 0;/);
});
