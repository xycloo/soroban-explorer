use explorer_common::{
    operations::get_contract_operations,
    types::{common::Processed, transaction::Response},
};

use log::info;
use stdweb::js;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Default)]
pub struct App {
    contract_id: String,
    processed_ops: Vec<Option<Processed>>,
    next: String,
}

pub enum AppMsg {
    ContractId(String),
    ProcessedOps(Vec<Option<Processed>>),
    NextHref(String),
    LoadMore,
}

async fn read_and_process_ops(
    mut url: String,
    f: impl Fn(Vec<Option<Processed>>),
    send_href: impl Fn(String),
    id: String,
) {
    let mut out: Vec<Option<Processed>> = Vec::new();

    js! {
    document.getElementById("status").innerText = "searching ..."
    }

    while out.len() == 0 {
        let processed =
            get_contract_operations("https://horizon-futurenet.stellar.org", &url, id.as_str())
                .await;
        out = processed
            .0
            .into_iter()
            .map(|proc| Some(proc))
            .collect::<Vec<Option<Processed>>>();

        url = processed.2;
    }

    js! {
    document.getElementById("status").innerText = ""
    }

    send_href(url);
    f(out);
}

trait Extend {
    fn read_id(&self) -> &str;
}

impl Extend for App {
    fn read_id(&self) -> &str {
        &self.contract_id
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
            AppMsg::ProcessedOps(mut processed) => {
                self.processed_ops.append(&mut processed);
                true
            }
            AppMsg::ContractId(id) => {
                let link = ctx.link().clone();
                let link1 = ctx.link().clone();
                self.contract_id = id.clone();
                wasm_bindgen_futures::spawn_local(async {
                    read_and_process_ops(
                        String::from(
                            "https://horizon-futurenet.stellar.org/operations?order=desc&limit=200",
                        ),
                        move |processed| link.send_message(AppMsg::ProcessedOps(processed)),
                        move |next_href| link1.send_message(AppMsg::NextHref(next_href)),
                        id,
                    )
                    .await;
                    js! {
                    document.getElementById("loadbtn").classList.remove("hidden");
                    }

                    js! {
                            var coll = document.getElementsByClassName("collapsible");
                            var i;

                            for (i = 0; i < coll.length; i++) {
                              coll[i].addEventListener("click", function() {
                                this.classList.toggle("active");
                                var content = this.nextElementSibling;
                                if (content.style.display === "block") {
                                  content.style.display = "none";
                                } else {
                                  content.style.display = "block";
                                }
                              });
                            };
                    hljs.highlightAll();
                                        }
                });
                true
            }
            AppMsg::NextHref(next_href) => {
                self.next = next_href;
                true
            }
            AppMsg::LoadMore => {
                let link = ctx.link().clone();
                let link1 = ctx.link().clone();
                let id = self.contract_id.clone();
                let next = self.next.clone();
                wasm_bindgen_futures::spawn_local(async {
                    read_and_process_ops(
                        next,
                        move |processed| link.send_message(AppMsg::ProcessedOps(processed)),
                        move |next_href| link1.send_message(AppMsg::NextHref(next_href)),
                        id,
                    )
                    .await;
                });
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let processed_operations = self.processed_ops.clone();
        let link = ctx.link().clone();
        let link1 = ctx.link().clone();

        let oninput = Callback::from(move |e: InputEvent| {
            let target = e.target().unwrap();
            let input = target.unchecked_into::<HtmlInputElement>();
            link.send_message(AppMsg::ContractId(input.value()))
        });

        let loadmore = Callback::from(move |e: MouseEvent| link1.send_message(AppMsg::LoadMore));

        html! {
                            <main>
                    <div id="heading">
                                <h1>{ "Soroban operations for contract explorer" }</h1>
                <p style="color: #7c7c7c">{ "Made with " }<span class="heart"></span> { " by " } <a href="https://github.com/xycloo">{ "Xycloo" }</a></p>
        <p style="width: 600px;text-align:left;color:#7c7c7c">{ "Paste below your contract's id and see its invocations. Might take a bit if the contract hasn't been invoked in some time. To load more invocations click on \"Load More Operations\". Check out the " }<a href="https://github.com/xycloo/soroban-explorer/tree/main/web">{"repo"}</a>{"."}</p>
                                <div>
                    <input oninput={oninput} />

                <p id="status"></p>

                    </div>
                    </div>

                    {
                        for processed_operations.into_iter().map(|e| {
                    let action = match e.clone().unwrap().body {
                    explorer_common::types::common::Event::Invocation(invocation) => invocation.function,
                    explorer_common::types::common::Event::Deployment(_) => String::from("deploy"),
                    };
                    html! {
                    <div>
                    <button type="button" class="collapsible">{&action}</button>
                    <div class="content">
                    <pre><code class="language-json">
                        {
                    serde_json::to_string_pretty(&e.unwrap()).unwrap()
                        }
                </code></pre>
                    </div>
                        </div>
                    }
                    }
                    )
                    }

                <div class="centered">
                <button id="loadbtn" class="hidden" onclick={loadmore}>{"Load More Operations"}</button>
        </div>
                    </main>

                        }
    }
}
