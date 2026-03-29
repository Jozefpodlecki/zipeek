use log::*;
use yew::prelude::*;
use web_sys::{console::info, window, HtmlElement, SpeechSynthesisUtterance, SpeechSynthesisVoice};
use wasm_bindgen::JsCast;
use yew_icons::{Icon, IconData};
use crate::{contexts::SearchContext, models::{Lexeme, LexemeBreakdown}, services::ApiClient};

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {
    // pub path: String
}




// fn parse(path: &str) -> Option<Lexeme> {
//     match path {
//         "纪录片" => Some(Lexeme {
//             value: "纪录片".to_string(),
//             pinyin: "jì lù piàn".to_string(),
//             breakdown: vec![
//                 LexemeBreakdown {
//                     grapheme: "纪".to_string(),
//                     pinyin: "jì".to_string(),
//                 },
//                 LexemeBreakdown {
//                     grapheme: "录".to_string(),
//                     pinyin: "lù".to_string(),
//                 },
//                 LexemeBreakdown {
//                     grapheme: "片".to_string(),
//                     pinyin: "piàn".to_string(),
//                 },
//             ],
//             part_of_speech: "noun".to_string(),
//             meanings: vec![
//                 "documentary".to_string(),
//                 "documentary film".to_string(),
//             ],
//         }),
//         _ => None,
//     }
// }

// fn render_lexeme(lexeme: &Lexeme) -> Html {

//     let on_play: Callback<MouseEvent> = {

//         Callback::from(move |event: MouseEvent| {
//             let current_target = event.target().unwrap();
//             let html_element = current_target.unchecked_into::<HtmlElement>();
//             let button = unsafe { html_element.closest("button")
//                 .unwrap_unchecked()
//                 .unwrap_unchecked()
//                 .unchecked_into::<HtmlElement>()
//             };

//             let dataset = button.dataset();
//             let value = unsafe { dataset.get("value").unwrap_unchecked() };
//             info!("{value}");
//             speak(&value);
//         })
//     };

//     html! {
//         <div class="flex flex-col items-center gap-6">

//             <div class="flex gap-2 text-9xl leading-none">
//                 { for lexeme.breakdown.iter().map(|b| {
//                     html! {
//                         <div class="flex flex-col items-center">
//                             <span>{ &b.grapheme }</span>
//                             <span class="text-xl text-gray-400">
//                                 { &b.pinyin }
//                             </span>
//                         </div>
//                     }
//                 })}
//             </div>

      
//             <div class="text-3xl text-gray-300">
//                 { &lexeme.pinyin }
//                 <button data-value={lexeme.pinyin.clone()} class="dark:text-white" onclick={on_play}>
//                     <Icon data={IconData::LUCIDE_PLAY} width={"20px".to_owned()}/>
//                 </button>
//             </div>


//             <div class="text-xl text-gray-400">
//                 { for lexeme.meanings.iter().map(|m| html! { <div>{ m }</div> }) }
//             </div>

//         </div>
//     }
// }

#[function_component(Home)]
pub fn home(props: &Props) -> Html {
    let api_client = unsafe { use_context::<ApiClient>().unwrap_unchecked() };
    let search_context = unsafe { use_context::<SearchContext>().unwrap_unchecked() };

    let on_search: Callback<String> = {

        Callback::from(move |value| {
            

           
        })
    };

    {
        let api_client = api_client.clone();
        let search_context = search_context.clone();

        use_effect(move || {
            let api_client = api_client.clone();
            let value = search_context.get().clone();

            if value.is_empty() {
                return;
            }

            wasm_bindgen_futures::spawn_local(async move {
                match api_client.search(&value).await {
                    Ok(result) => {
                        info!("ok");
                    },
                    Err(err) => {
                        info!("{}", err);
                    },
                }
            });
        });
    }   
    
    
    html! {
        <div data-home="" class="">
            {search_context.get()}
        </div>
    }
}