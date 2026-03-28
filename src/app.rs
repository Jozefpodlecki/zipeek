use std::rc::Rc;

use web_sys::{Document, HtmlElement, Navigator, Storage, Window};
use yew::*;
use yew_router::{HashRouter, Switch};

use crate::{api::ApiClient, components::{Background, Layout, Loader}, models::{AppState, Social}, route::{switch, Route}};

async fn fetch_social(client: ApiClient, app_state: UseStateHandle<AppState>) {
    app_state.set(AppState::Loading);

    match client.get_social().await {
        Ok(social) => {
            app_state.set(AppState::Loaded(social));
        }
        Err(err) => {
            app_state.set(AppState::Error(err));
        }
    }
}


#[derive(Debug, Clone, PartialEq, Properties)]
pub struct AppProps {
    pub window: Window,
    pub document: Document,
    pub body: HtmlElement,
    pub local_storage: Storage,
    pub navigator: Navigator,
    pub app_name: Rc<str>,
    pub version: Rc<str>,
}

#[function_component(App)]
pub fn app(props: &AppProps) -> Html {
    let app_state = use_state(AppState::default);
    let AppProps { window, .. } = props;

    let client: ApiClient = ApiClient::new(window.clone());
    
    {
        let app_state = app_state.clone();
        let client = client.clone();

        use_effect_with(
            (),
            move |_| wasm_bindgen_futures::spawn_local(fetch_social(client, app_state)),
        );
    }

    match &*app_state {
        AppState::Loading => {
            html! {
                <Layout>
                    <article data-loading="" class="flex w-full h-full justify-center items-center">
                        <Loader/>
                    </article>
                </Layout>
            }
        },
        AppState::Error(error) => {
            html! {
                <Layout>
                    // <Error error={error.clone()} on_retry={on_retry}/>
                </Layout>
            }
        },
        AppState::Loaded(social) => {
            html! {
                <ContextProvider<ApiClient> context={client}>
                    <ContextProvider<Social> context={social.clone()}>
                        <HashRouter>
                            <Background/>
                            <Layout>
                                <Switch<Route> render={switch} />
                            </Layout>
                        </HashRouter>
                    </ContextProvider<Social>>
                </ContextProvider<ApiClient>>
            }
        },
    }

    // html! {
    //     <ContextProvider<ApiClient> context={client}>
    //         <ContextProvider<Social> context={social.clone()}>
    //             <HashRouter>
    //                 <Background/>
    //                 <Layout>
    //                     <Switch<Route> render={switch} />
    //                 </Layout>
    //             </HashRouter>
    //         </ContextProvider<Social>
    //     </ContextProvider<ApiClient>>
    // }
}
