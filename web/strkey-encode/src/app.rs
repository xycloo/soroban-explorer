use log::info;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlInputElement;
use yew::prelude::*;

use hex;
use stellar_strkey::*;

#[derive(Default)]
pub struct App {
    hash: String,
    strkey: String,
}

pub enum AppMsg {
    Hash(String),
    Encoded(String),
}

fn read_and_process_tx(f: impl Fn(String), id: String) {
    let encoded = Strkey::Contract(stellar_strkey::Contract(
        hex::decode(id).unwrap().as_slice().try_into().unwrap(),
    ))
    .to_string();

    f(encoded);
}

trait Extend {
    fn read_id(&self) -> &str;
}

impl Extend for App {
    fn read_id(&self) -> &str {
        &self.hash
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
            AppMsg::Encoded(encoded) => {
                self.strkey = encoded;
                true
            }
            AppMsg::Hash(hash) => {
                let link = ctx.link().clone();
                read_and_process_tx(
                    move |encoded| link.send_message(AppMsg::Encoded(encoded)),
                    hash,
                );

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let processed_transaction = self.strkey.clone();
        let link = ctx.link().clone();

        let oninput = Callback::from(move |e: InputEvent| {
            let target = e.target().unwrap();
            let input = target.unchecked_into::<HtmlInputElement>();
            link.send_message(AppMsg::Hash(input.value()))
        });

        html! {
                <main>
        <div id="heading">
                    <h1>{ "Hex to StrKey" }</h1>
                    <p>{ "Convert soroban contract hashes to StrKey. Made with " }<span class="heart"></span> { " by " } <a href="https://github.com/xycloo">{ "Xycloo" }</a></p>
                    <div>
        <input oninput={oninput} />

        </div>
        </div>

        <pre><code class="language-json"> {&self.strkey
        } </code></pre>
        </main>

            }
    }
}
