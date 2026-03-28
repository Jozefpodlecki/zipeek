mod loaded;
mod error;

pub use loaded::*;
pub use error::*;

                    // <div class="max-w-4xl mx-auto px-4 py-8">
                    //     <LexemeHeader lexeme={lexeme} />
                    //     <div class="space-y-4 mt-8">
                    //         {lexeme.variants.iter().map(|variant| html! {
                    //             <VariantCard variant={variant} />
                    //         }).collect::<Html>()}
                    //     </div>
                    //     <StandardsList standards={&lexeme.standards} />
                    // </div>

// #[derive(Properties, PartialEq)]
// pub struct LexemeHeaderProps {
//     pub lexeme: ChineseLexeme,
// }

// #[function_component(LexemeHeader)]
// pub fn lexeme_header(props: &LexemeHeaderProps) -> Html {
//     let pos_badges: Html = props.lexeme.part_of_speech.iter()
//         .map(|pos| {
//             let pos_class = match pos {
//                 PartOfSpeech::Noun => "bg-blue-100 text-blue-800",
//                 PartOfSpeech::Verb => "bg-green-100 text-green-800",
//                 PartOfSpeech::Adjective => "bg-purple-100 text-purple-800",
//                 PartOfSpeech::Adverb => "bg-indigo-100 text-indigo-800",
//                 PartOfSpeech::Pronoun => "bg-pink-100 text-pink-800",
//                 PartOfSpeech::Numeral => "bg-orange-100 text-orange-800",
//                 PartOfSpeech::MeasureWord => "bg-yellow-100 text-yellow-800",
//                 PartOfSpeech::Conjunction => "bg-gray-100 text-gray-800",
//                 PartOfSpeech::Preposition => "bg-teal-100 text-teal-800",
//                 PartOfSpeech::Particle => "bg-cyan-100 text-cyan-800",
//                 PartOfSpeech::Interjection => "bg-red-100 text-red-800",
//                 PartOfSpeech::Unknown => "bg-gray-100 text-gray-600",
//             };
            
//             html! {
//                 <span class={classes!("px-2", "py-1", "rounded-full", "text-xs", "font-medium", pos_class)}>
//                     {format!("{:?}", pos).to_lowercase()}
//                 </span>
//             }
//         })
//         .collect();

//     html! {
//         <div class="border-b border-gray-200 pb-6 mb-6">
//             <div class="flex items-baseline gap-4 flex-wrap">
//                 <h1 class="text-5xl font-bold text-gray-900">
//                     {&props.lexeme.simplified}
//                 </h1>
//                 <div class="flex gap-2 flex-wrap">
//                     {pos_badges}
//                 </div>
//             </div>
//         </div>
//     }
// }


// #[derive(Properties, PartialEq)]
// pub struct VariantCardProps {
//     pub variant: LexicalVariant,
// }

// #[function_component(VariantCard)]
// pub fn variant_card(props: &VariantCardProps) -> Html {
//     let pinyin_text: Html = props.variant.pinyin.iter()
//         .enumerate()
//         .map(|(i, p)| html! {
//             <>
//                 <span class="text-gray-600 italic">{p}</span>
//                 {if i < props.variant.pinyin.len() - 1 { ", " } else { "" }}
//             </>
//         })
//         .collect();

//     let classifiers: Html = if !props.variant.classifiers.is_empty() {
//         html! {
//             <div class="mt-3 pt-3 border-t border-gray-100">
//                 <span class="text-sm font-medium text-gray-500 mr-2">{"Classifiers:"}</span>
//                 {props.variant.classifiers.iter().map(|c| html! {
//                     <span class="inline-block px-2 py-1 bg-gray-100 rounded-md text-sm text-gray-700 mr-2">
//                         {&c.word}
//                     </span>
//                 }).collect::<Html>()}
//             </div>
//         }
//     } else {
//         html! {}
//     };

//     html! {
//         <div class="bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden hover:shadow-md transition-shadow">
//             <div class="p-5">
//                 <div class="flex items-baseline gap-3 mb-4">
//                     <span class="text-2xl font-medium text-gray-800">{&props.variant.traditional}</span>
//                     <span class="text-sm text-gray-500">{"·"}</span>
//                     <div class="text-sm text-gray-600">
//                         {pinyin_text}
//                     </div>
//                 </div>
                
//                 <div class="space-y-4">
//                     {props.variant.senses.iter().map(|sense| html! {
//                         <SenseItem sense={sense} />
//                     }).collect::<Html>()}
//                 </div>
                
//                 {classifiers}
//             </div>
//         </div>
//     }
// }


// #[derive(Properties, PartialEq)]
// pub struct Props {
//     pub sense: ChineseSense,
// }

// #[function_component(SenseItem)]
// pub fn sense_item(props: &Props) -> Html {
//     let glosses: Html = props.sense.glosses.iter()
//         .enumerate()
//         .map(|(i, gloss)| html! {
//             <>
//                 <span class="text-gray-800">{gloss}</span>
//                 {if i < props.sense.glosses.len() - 1 { 
//                     html! { <span class="text-gray-400 mx-1">{"·"}</span> }
//                 } else { html! {} }}
//             </>
//         })
//         .collect();

//     let tags: Html = if !props.sense.tags.is_empty() {
//         html! {
//             <div class="flex gap-1 mt-2">
//                 {props.sense.tags.iter().map(|tag| html! {
//                     <span class="px-2 py-0.5 bg-gray-100 text-gray-600 text-xs rounded-full">
//                         {tag}
//                     </span>
//                 }).collect::<Html>()}
//             </div>
//         }
//     } else {
//         html! {}
//     };

//     let qualifier: Html = if let Some(qualifier) = &props.sense.qualifier {
//         html! {
//             <div class="text-sm text-gray-500 italic mt-1">
//                 {format!("({})", qualifier)}
//             </div>
//         }
//     } else {
//         html! {}
//     };

//     let pos_tags: Html = if !props.sense.part_of_speech.is_empty() {
//         html! {
//             <div class="flex gap-1 mt-2">
//                 {props.sense.part_of_speech.iter().map(|pos| {
//                     let pos_class = match pos {
//                         zipseek_core::PartOfSpeech::Noun => "bg-blue-50 text-blue-700",
//                         zipseek_core::PartOfSpeech::Verb => "bg-green-50 text-green-700",
//                         zipseek_core::PartOfSpeech::Adjective => "bg-purple-50 text-purple-700",
//                         zipseek_core::PartOfSpeech::Adverb => "bg-indigo-50 text-indigo-700",
//                         _ => "bg-gray-50 text-gray-600",
//                     };
//                     html! {
//                         <span class={classes!("px-2", "py-0.5", "rounded", "text-xs", "font-medium", pos_class)}>
//                             {format!("{:?}", pos).to_lowercase()}
//                         </span>
//                     }
//                 }).collect::<Html>()}
//             </div>
//         }
//     } else {
//         html! {}
//     };

//     html! {
//         <div class="pl-3 border-l-2 border-gray-200">
//             <div class="text-base leading-relaxed">
//                 {glosses}
//             </div>
//             {qualifier}
//             {tags}
//             {pos_tags}
//         </div>
//     }
// }


// #[derive(Properties, PartialEq)]
// pub struct Props {
//     pub standards: Vec<ReferenceStandard>,
// }

// #[function_component(StandardsList)]
// pub fn standards_list(props: &Props) -> Html {
//     if props.standards.is_empty() {
//         return html! {};
//     }

//     html! {
//         <div class="mt-8 pt-6 border-t border-gray-200">
//             <h3 class="text-sm font-semibold text-gray-500 uppercase tracking-wider mb-3">
//                 {"References"}
//             </h3>
//             <div class="flex flex-wrap gap-2">
//                 {props.standards.iter().map(|std| {
//                     let (bg_color, text_color) = match std.name.as_ref() {
//                         "hsk20" => ("bg-blue-50", "text-blue-700"),
//                         "hsk30" => ("bg-green-50", "text-green-700"),
//                         "cefr" => ("bg-purple-50", "text-purple-700"),
//                         _ => ("bg-gray-50", "text-gray-700"),
//                     };
                    
//                     html! {
//                         <div class={classes!("px-3", "py-1", "rounded-full", "text-sm", bg_color, text_color)}>
//                             <span class="font-medium">{&std.name}</span>
//                             <span class="mx-1 text-gray-400">{"·"}</span>
//                             <span>{&std.value}</span>
//                         </div>
//                     }
//                 }).collect::<Html>()}
//             </div>
//         </div>
//     }
// }