use std::collections::BTreeSet;
use yew::prelude::*;
use zipeek_core::HskSystem;
use zipeek_exercises::practice_draft::HskBand;

#[derive(Properties, PartialEq)]
pub(crate) struct HskPreferencesSelectorProps {
    /// 2, 3 or 0 (Mixed).
    pub standard: u8,
    pub bands: BTreeSet<HskBand>,
    pub on_standard: Callback<u8>,
    pub on_toggle: Callback<HskBand>,
}

#[function_component(HskPreferencesSelector)]
pub(crate) fn hsk_preferences_selector(props: &HskPreferencesSelectorProps) -> Html {
    html! {
        <div data-component="hsk-preferences-selector" class="flex flex-col gap-3">
            <div role="group" aria-label="HSK standard" class="inline-flex w-fit flex-wrap gap-1 rounded-xl border border-line bg-raised p-1">
                {for [(2, "2.0"), (3, "3.0"), (0, "Mixed")].into_iter().map(|(system, label)| {
                    let active = props.standard == system;
                    let on_standard = props.on_standard.clone();
                    html! { <button key={system} type="button" aria-pressed={active.to_string()} class={classes!("min-h-10", "rounded-lg", "px-4", "text-sm", "font-semibold", if active { "bg-brand-action text-white" } else { "text-muted hover:bg-surface hover:text-ink" })} onclick={Callback::from(move |_| on_standard.emit(system))}>{label}</button> }
                })}
            </div>
            {for HskSystem::ALL.into_iter().filter(|system| props.standard == 0 || system.number() == props.standard).map(|system| {
                let mixed = props.standard == 0;
                html! { <div key={system.number()} class="flex flex-col gap-2">
                    {if mixed { html! { <span class="text-xs text-muted">{system.label()}</span> } } else { Html::default() }}
                    <div role="group" aria-label={format!("{} levels", system.label())} class="flex flex-wrap gap-2">
                        {for system.levels().iter().copied().map(|level| {
                            let band = HskBand::from_level(level);
                            let active = props.bands.contains(&band);
                            let on_toggle = props.on_toggle.clone();
                            let label = if level.as_i32() == 7 { "7–9".to_owned() } else { level.as_i32().to_string() };
                            html! { <button key={format!("{}:{}", band.system, band.level)} type="button" aria-pressed={active.to_string()} class={classes!("min-h-10", "rounded-lg", "border", "px-4", "text-sm", if active { "border-brand-action bg-brand-action text-white" } else { "border-line bg-raised hover:bg-surface" })} onclick={Callback::from(move |_| on_toggle.emit(band))}>{label}</button> }
                        })}
                    </div>
                </div> }
            })}
        </div>
    }
}
