use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

use crate::hooks::use_dictionary;
use crate::pages::record::readings::{self, ReadingGlyph};
use crate::pages::record::state::{RecordState, TargetMode, comparable};

#[derive(Properties, PartialEq)]
pub(super) struct RecognitionResultProps {
    pub state: UseReducerHandle<RecordState>,
}

#[function_component(RecognitionResult)]
pub(super) fn recognition_result(props: &RecognitionResultProps) -> Html {
    let state = &props.state;
    let dictionary = use_dictionary();
    let heard_readings = use_state(|| None::<(String, Vec<ReadingGlyph>)>);
    let heard_key = state.transcript.clone();
    {
        let dictionary = dictionary.clone();
        let heard_readings = heard_readings.clone();
        use_effect_with(heard_key, move |heard| {
            let active = Rc::new(Cell::new(true));
            if let Some(heard) = heard.clone() {
                let active_request = active.clone();
                spawn_local(async move {
                    let resolved = async {
                        let db = dictionary.database().ok()?;
                        let candidates = db.text_reading_candidates(&heard).await.ok()?;
                        Some(readings::resolve(&heard, &candidates))
                    }.await;
                    if active_request.get() {
                        heard_readings.set(resolved.map(|readings| (heard, readings)));
                    }
                });
            }
            move || active.set(false)
        });
    }
    let Some(heard) = state.transcript.as_deref() else {
        return Html::default();
    };
    let expected = state.target().unwrap_or_default();
    let expected_normalized = comparable(expected);
    let heard_normalized = comparable(heard);
    let matches = expected_normalized == heard_normalized;
    let expected_pinyin = if state.mode == TargetMode::Hsk {
        state.selected.as_ref().map(|word| word.pinyin.as_str())
    } else {
        None
    };
    let resolved = (*heard_readings).as_ref()
        .filter(|(text, _)| text == heard)
        .map(|(_, readings)| readings.as_slice());
    let characters: Vec<char> = heard.chars().collect();
    let aligned = resolved.filter(|readings| readings.len() == characters.len()
        && readings.iter().zip(characters.iter()).all(|(glyph, character)| glyph.character == *character));
    let status_text = if matches {
        "Recognized text matches the reference"
    } else {
        "Recognized text differs from the reference. Check the recording and retry if needed."
    };

    html! {
        <section class="rounded-2xl border border-line bg-surface px-4 py-3 shadow-sm sm:px-5" aria-label="Speech recognition results">
            <div class="flex flex-wrap items-start gap-x-10 gap-y-3">
                <div class="min-w-0">
                    <span class="text-xs text-muted">{"Expected"}</span>
                    <p class="break-words text-2xl font-semibold" lang="zh-CN">{expected}</p>
                    if let Some(pinyin) = expected_pinyin {
                        <p class="text-xs text-muted" lang="zh-Latn">{pinyin}</p>
                    }
                </div>
                <div class="min-w-0">
                    <div class="inline-flex items-center gap-1.5">
                        <span class="text-xs text-muted">{"Heard"}</span>
                        <span class={classes!("inline-flex", "h-4", "w-4", "items-center", "justify-center", if matches {"text-green-500"} else {"text-amber-400"})}
                            role="img" aria-label={status_text} title={status_text}>
                            <Icon data={if matches {IconData::LUCIDE_CHECK} else {IconData::LUCIDE_ALERT_TRIANGLE}} width="15px" height="15px" />
                        </span>
                    </div>
                    <p data-record-transcript="" class={classes!("flex", "flex-wrap", "gap-x-0.5", "gap-y-2", "break-words", "text-2xl", "font-semibold", if matches {"text-green-500"} else {"text-amber-400"})} lang="zh-CN">
                        {for characters.iter().enumerate().map(|(index, character)| {
                            let pinyin = aligned.and_then(|readings| readings[index].pinyin.as_deref());
                            html! {
                                <span key={index} class="inline-flex min-w-0 flex-col items-center justify-end">
                                    <span class="min-h-4 text-xs font-normal leading-4 text-muted" lang="zh-Latn">{pinyin.unwrap_or("\u{00a0}")}</span>
                                    <span>{character}</span>
                                </span>
                            }
                        })}
                    </p>
                </div>
            </div>
        </section>
    }
}
