use explorer_common::{
    transactions::{get_transaction, process_tx},
    types::{common::Processed, transaction::Response},
};
use log::info;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Default)]
pub struct App {
    transaction: String,
    processed_tx: Vec<Option<Processed>>,
}

pub enum AppMsg {
    TransactionURL(String),
    ProcessedTx(Processed),
}

async fn read_and_process_tx(f: impl Fn(Processed), id: String) {
    let processed = process_tx(
        get_transaction("https://horizon-futurenet.stellar.org/", &id)
            .await
            .unwrap(),
    );

    let object = JsValue::from(processed.clone().source_account);
    f(processed);
    //    extern crate stdweb;
    //    use stdweb::js;
    //    js! {
    //        hljs.highlightAll()
    //    }
}

trait Extend {
    fn read_id(&self) -> &str;
}

impl Extend for App {
    fn read_id(&self) -> &str {
        &self.transaction
    }
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        wasm_logger::init(wasm_logger::Config::default());

        Self::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::ProcessedTx(processed) => {
                self.processed_tx = vec![Some(processed)];
                true
            }
            AppMsg::TransactionURL(id) => {
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async {
                    read_and_process_tx(
                        move |processed| link.send_message(AppMsg::ProcessedTx(processed)),
                        id,
                    )
                    .await;
                });
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let processed_transaction = self.processed_tx.clone();
        let link = ctx.link().clone();

        let oninput = Callback::from(move |e: InputEvent| {
            let target = e.target().unwrap();
            let input = target.unchecked_into::<HtmlInputElement>();
            link.send_message(AppMsg::TransactionURL(input.value()))
        });

        html! {
                <main>
        <div id="heading">
                    <h1>{ "Soroban transaction explorer" }</h1>
                    <p>{ "made with " }<span class="heart"></span> { " by " } <a href="https://github.com/xycloo">{ "Xycloo" }</a></p>
                    <div>
        <input oninput={oninput} />

        </div>
        </div>

        <pre><code class="language-json"> {for processed_transaction.into_iter().map(|e| {
            serde_json::to_string_pretty(&e.unwrap()).unwrap()
        })


        } </code></pre>
        </main>

            }
    }
}

/*
#[function_component(App)]
pub fn app() -> Html {
    wasm_bindgen_futures::spawn_local(async {
        let processed = process_tx(
            get_transaction(
                "https://horizon-futurenet.stellar.org/",
                "4fb3636a9b4351043ab60e0436fd931113719a378473c8fe185bc9988bd13f3f",
            )
            .await,
        );


    });

    html! {
        <main>
            <h1>{ "Soroban transaction explorer" }</h1>
            <p>{ "made with " }<span class="heart"></span> { " by " } <a href="https://github.com/xycloo">{ "Xycloo" }</a></p>
        </main>
    }
}*/
