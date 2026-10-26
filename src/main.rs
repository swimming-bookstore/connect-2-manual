//! Static preview of the Connect 2 instance manual.
//! Two orgs, one desk. Ask AI can run a diagnosis; the page cannot.

use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

/// A Connect 2 org is a cluster of boxes, same idea as a Teleport cluster.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Org {
    Midnight,
    Cronos,
}

impl Org {
    fn from_path(path: &str) -> Self {
        if path.split('/').any(|part| part == "cronos") {
            Org::Cronos
        } else {
            Org::Midnight
        }
    }

    fn boxes(self) -> &'static [BoxId] {
        match self {
            Org::Midnight => &[BoxId::One, BoxId::Two],
            Org::Cronos => &[BoxId::Evm, BoxId::Pos],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BoxId {
    One,
    Two,
    Evm,
    Pos,
}

impl BoxId {
    fn name(self) -> &'static str {
        match self {
            BoxId::One => "cardano-node-1",
            BoxId::Two => "midnight-node-1",
            BoxId::Evm => "cronos-node-1",
            BoxId::Pos => "cronos-pos-node-1",
        }
    }

    fn tags(self) -> &'static [&'static str] {
        match self {
            BoxId::One => &["Cardano node", "GCP", "mainnet"],
            BoxId::Two => &["Midnight node", "GCP", "mainnet"],
            BoxId::Evm => &["Cronos EVM", "GCP", "mainnet"],
            BoxId::Pos => &["Cronos POS", "GCP", "mainnet"],
        }
    }

}

struct Fix {
    symptom: &'static str,
    cmd: &'static str,
    ask: &'static str,
    reply: &'static str,
}

#[derive(Clone, Copy)]
enum Line {
    You(&'static str),
    Step(&'static str),
    Ai(&'static str),
}

fn cardano_fixes() -> Vec<Fix> {
    vec![
        Fix {
            symptom: "Relay looks stuck or will not answer on its socket",
            cmd: "systemctl is-active cardano-node.service\nexport CARDANO_NODE_SOCKET_PATH=/data/cardano-data/db/node.socket\ncardano-cli conway query tip --mainnet",
            ask: "Check cardano-node.service, then query the tip on /data/cardano-data/db/node.socket with --mainnet. Read only. Report syncProgress only. Do not restart the relay.",
            reply: "cardano-node.service is active. query tip answered and syncProgress is 100.00. The relay is caught up. Do not restart it.",
        },
        Fix {
            symptom: "Need the last relay error, not a restart",
            cmd: "journalctl -u cardano-node.service -n 40 --no-pager",
            ask: "Read the last 40 lines of cardano-node.service. Quote the one line that explains the failure. Do not restart it.",
            reply: "The useful line is TraceNoLedgerView while the unit is still active. That is sync lag. Confirm it with query tip before touching cardano-node.service.",
        },
        Fix {
            symptom: "db-sync is up but may not be writing blocks",
            cmd: "systemctl is-active cardano-db-sync.service\npsql -d cexplorer -tAc \"SELECT block_no, slot_no, time FROM block ORDER BY id DESC LIMIT 1;\"",
            ask: "Check cardano-db-sync.service, then read the latest cexplorer block row. Read only. Say whether a block row came back. Do not restart it.",
            reply: "cardano-db-sync.service is active. The latest block row came back, so it is writing. Leave the unit alone.",
        },
        Fix {
            symptom: "Is db-sync caught up enough to start Midnight?",
            cmd: "psql -d cexplorer -tAc \"SELECT 100 * (EXTRACT(epoch FROM (MAX(time) AT TIME ZONE 'UTC')) - EXTRACT(epoch FROM (MIN(time) AT TIME ZONE 'UTC'))) / (EXTRACT(epoch FROM (NOW() AT TIME ZONE 'UTC')) - EXTRACT(epoch FROM (MIN(time) AT TIME ZONE 'UTC'))) AS sync_percent FROM block;\"",
            ask: "Run the runbook sync_percent query on cexplorer. Read only. Say whether it is at or above 99.99. Do not restart cardano-db-sync.",
            reply: "sync_percent is above 99.99. db-sync is caught up. Do not restart cardano-db-sync.service. Midnight may use this database.",
        },
        Fix {
            symptom: "db-sync journal shows errors instead of applied blocks",
            cmd: "journalctl -u cardano-db-sync.service -n 40 --no-pager",
            ask: "Read the last 40 lines of cardano-db-sync.service. Say if it is applying blocks or failing to reach the socket. Do not restart it. Confirm with the sync_percent query before acting.",
            reply: "It is still applying blocks behind the relay. No socket error. Leave cardano-db-sync.service alone until sync_percent is checked.",
        },
    ]
}

fn midnight_fixes() -> Vec<Fix> {
    vec![
        Fix {
            symptom: "Which midnight-node is installed?",
            cmd: "sudo -u ubuntu /home/ubuntu/.local/bin/midnight-node --version",
            ask: "Run midnight-node --version as the service user. Read only. Say whether the binary answers. Do not treat the version number as part of the manual.",
            reply: "midnight-node answered --version. The binary is in place. Do not download another release.",
        },
        Fix {
            symptom: "Node key or validator seeds look missing",
            cmd: "sudo test -s /secret/data/chains/midnight/network/secret_ed25519 \\\n  && sudo test -s \"$AURA_SEED_FILE\" \\\n  && sudo test -s \"$GRANDPA_SEED_FILE\" \\\n  && sudo test -s \"$CROSS_CHAIN_SEED_FILE\" \\\n  && echo keys-ok",
            ask: "Test that secret_ed25519 and the three seed files named in the env are non-empty. Do not read their contents, do not print a secret phrase, and do not generate replacements.",
            reply: "keys-ok. The node key and the aura, grandpa, and cross-chain seed files are present. Nothing to regenerate.",
        },
        Fix {
            symptom: "WireGuard identity was never generated",
            cmd: "wg --version\nsudo test -s /secret/wireguard/publickey && echo wg-ok",
            ask: "Check wg --version and that /secret/wireguard/publickey is non-empty. Do not print the private key and do not generate a new keypair.",
            reply: "wireguard-tools is installed and the public key file is non-empty. Identity is already there. Do not generate another.",
        },
        Fix {
            symptom: "Is the node producing blocks?",
            cmd: "curl -sf --max-time 4 -H 'content-type: application/json' --data '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"chain_getHeader\",\"params\":[]}' http://127.0.0.1:9933\nsleep 12\ncurl -sf --max-time 4 -H 'content-type: application/json' --data '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"chain_getHeader\",\"params\":[]}' http://127.0.0.1:9933",
            ask: "Call chain_getHeader on 127.0.0.1:9933, wait 12 seconds, and call it again. Read only. Say whether the best block number moved. Do not print the block hash and do not restart midnight-node.",
            reply: "The best block number moved between the two headers. The node is producing blocks. Leave midnight-node.service alone.",
        },
        Fix {
            symptom: "Validator is restarting and I need the reason",
            cmd: "systemctl is-active midnight-node.service\njournalctl -u midnight-node.service -n 40 --no-pager",
            ask: "Check midnight-node.service and the last 40 journal lines. Read only. Is it waiting on db-sync, or missing a validator flag? Do not quote a block height and do not restart it.",
            reply: "The unit is restarting on failure. The journal is still waiting on db-sync to catch up. Leave the unit alone until sync_percent is above 99.99.",
        },
    ]
}

fn evm_fixes() -> Vec<Fix> {
    vec![
        Fix {
            symptom: "cronosd looks stuck or will not answer RPC",
            cmd: "systemctl is-active cronosd.service\ncurl -sf --max-time 4 http://127.0.0.1:26657/status",
            ask: "Check that cronosd.service is active, then read Tendermint /status on 127.0.0.1:26657. Read only. Report catching_up only. Do not restart the node.",
            reply: "cronosd.service is active. /status answered and catching_up is false. The node is caught up. Do not restart it.",
        },
        Fix {
            symptom: "Need the last cronosd error, not a restart",
            cmd: "journalctl -u cronosd.service -n 40 --no-pager",
            ask: "Read the last 40 lines of cronosd.service. Quote the one line that explains the failure. Do not restart it. Confirm with /status before acting.",
            reply: "The useful line is a peer dial timeout while the unit is still active. That is catch-up lag. Confirm catching_up before touching cronosd.service.",
        },
        Fix {
            symptom: "Is the node still catching up?",
            cmd: "curl -sf --max-time 4 http://127.0.0.1:26657/status | python3 -c 'import json,sys; s=json.load(sys.stdin)[\"result\"][\"sync_info\"]; print(\"catching_up\" if s[\"catching_up\"] else \"caught-up\")'",
            ask: "Read Tendermint status on 127.0.0.1:26657 and say only whether catching_up is true. Do not print a height and do not restart cronosd.",
            reply: "catching_up is false. The node is caught up. Leave cronosd.service alone. Do not record a block height.",
        },
        Fix {
            symptom: "How do I check the latest block height?",
            cmd: "curl -sf --max-time 4 -H 'content-type: application/json' --data '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"eth_blockNumber\",\"params\":[]}' http://127.0.0.1:8545 | python3 -c 'import json,sys; print(int(json.load(sys.stdin)[\"result\"], 16))'",
            ask: "Call eth_blockNumber on 127.0.0.1:8545. Read only. Print the latest block height as a decimal. Do not restart cronosd.",
            reply: "eth_blockNumber answered. The latest block height is the decimal printed above. Leave cronosd.service alone.",
        },
        Fix {
            symptom: "Which cronosd is installed?",
            cmd: "sudo -u ubuntu /home/ubuntu/.local/bin/cronosd version",
            ask: "Run cronosd version as the service user. Read only. Say whether the binary answers. Do not treat the version number as part of the manual.",
            reply: "cronosd answered version. The binary is in place. Do not download another release.",
        },
    ]
}

fn pos_fixes() -> Vec<Fix> {
    vec![
        Fix {
            symptom: "Validator is restarting and I need the reason",
            cmd: "systemctl is-active cronos-pos.service\njournalctl -u cronos-pos.service -n 40 --no-pager",
            ask: "Check cronos-pos.service and the last 40 journal lines. Read only. Is it waiting on peers, or missing a consensus key? Do not quote a block height and do not restart it.",
            reply: "The unit is restarting on failure. The journal is still waiting on peers. Confirm catching_up before touching cronos-pos.service.",
        },
        Fix {
            symptom: "Confirm the node was started in validator mode",
            cmd: "tr '\\0' '\\n' < /proc/$(systemctl show -p MainPID --value cronos-pos.service)/cmdline",
            ask: "Read the live chain-maind command line. Do not restart it. Are start, --home /data/.home, and --chain-id present? Do not print a key file.",
            reply: "start, --home /data/.home, and --chain-id crypto-org-chain-mainnet-1 are on the live process. Chain home is /data/.home. Do not restart it.",
        },
        Fix {
            symptom: "Is the validator still catching up?",
            cmd: "curl -sf --max-time 4 http://127.0.0.1:26657/status | python3 -c 'import json,sys; s=json.load(sys.stdin)[\"result\"][\"sync_info\"]; print(\"catching_up\" if s[\"catching_up\"] else \"caught-up\")'",
            ask: "Read Tendermint status on 127.0.0.1:26657 and say only whether catching_up is true. Do not print a height and do not restart cronos-pos.",
            reply: "catching_up is false. The validator is caught up. Leave cronos-pos.service alone. Do not record a block height.",
        },
        Fix {
            symptom: "Consensus key is not loaded",
            cmd: "curl -sf --max-time 4 http://127.0.0.1:26657/status | python3 -c 'import json,sys; v=json.load(sys.stdin)[\"result\"][\"validator_info\"]; print(\"validator-ok\" if v.get(\"voting_power\", \"0\") != \"0\" else \"no-voting-power\")'",
            ask: "Read validator_info from 127.0.0.1:26657/status. Read only. Say whether voting_power is non-zero. Do not print the address or the public key, and do not restart it.",
            reply: "validator-ok. voting_power is non-zero, so the consensus key is loaded. Do not print the key and do not regenerate it.",
        },
        Fix {
            symptom: "How do I check the latest block height?",
            cmd: "curl -sf --max-time 4 http://127.0.0.1:26657/status | python3 -c 'import json,sys; print(json.load(sys.stdin)[\"result\"][\"sync_info\"][\"latest_block_height\"])'",
            ask: "Read Tendermint status on 127.0.0.1:26657. Read only. Print latest_block_height. Do not restart cronos-pos.",
            reply: "latest_block_height is the number printed above. Leave cronos-pos.service alone.",
        },
        Fix {
            symptom: "Which chain-maind is installed?",
            cmd: "sudo -u ubuntu /home/ubuntu/.local/bin/chain-maind version",
            ask: "Run chain-maind version as the service user. Read only. Say whether the binary answers. Do not treat the version number as part of the manual.",
            reply: "chain-maind answered version. The binary is in place. Do not download another release.",
        },
    ]
}

fn fixes_for(id: BoxId) -> Vec<Fix> {
    match id {
        BoxId::One => cardano_fixes(),
        BoxId::Two => midnight_fixes(),
        BoxId::Evm => evm_fixes(),
        BoxId::Pos => pos_fixes(),
    }
}

fn page_path() -> String {
    window()
        .location()
        .pathname()
        .ok()
        .unwrap_or_default()
}

fn page_query() -> String {
    window().location().search().ok().unwrap_or_default()
}

fn shot_mode() -> bool {
    page_query().split('&').any(|part| {
        let part = part.trim_start_matches('?');
        part == "shot" || part.starts_with("shot=")
    })
}

fn box_from_query(org: Org) -> Option<BoxId> {
    page_query().split('&').find_map(|part| {
        let part = part.trim_start_matches('?');
        let name = part.strip_prefix("box=")?;
        org.boxes().iter().copied().find(|id| id.name() == name)
    })
}

#[component]
fn App() -> impl IntoView {
    let org = RwSignal::new(Org::from_path(&page_path()));
    let id = box_from_query(org.get_untracked()).unwrap_or(org.get_untracked().boxes()[0]);
    let selected = RwSignal::new(id);
    // Card capture includes the Ask AI column. Keep the log empty so the
    // shot does not invent a conversation that was never started.
    let chat = RwSignal::new(Vec::<Line>::new());
    if shot_mode() {
        if let Some(doc) = window().document() {
            if let Some(el) = doc.document_element() {
                let _ = el.class_list().add_1("shot");
            }
        }
    }

    view! {
        <div class="app">
            <header class="card-head">
                <p class="card-kicker">"DEVOPS ESSENTIALS"</p>
                <p class="card-title">"Connect 2"</p>
                <p class="card-sub">"Manual"</p>
            </header>
            <div class="desk-frame">
                <div class="app-inner">
                    <Header selected />
                    <div class="desk">
                        <BoxList org selected />
                        <Manual selected chat />
                        <Ask chat />
                    </div>
                </div>
            </div>
            <p class="card-foot">
                "Use AI to control your boxes."
            </p>
            <p class="card-copy">"© 수영 책방 Swimming Bookstore"</p>
        </div>
    }
}

#[component]
fn Moon() -> impl IntoView {
    view! {
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"></path>
        </svg>
    }
}

#[component]
fn Header(selected: RwSignal<BoxId>) -> impl IntoView {
    view! {
        <header class="bar">
            <div class="slot">
                <button class="wordmark" type="button">"Connect 2"</button>
                <div class="spacer">
                    <button class="ghost" type="button" aria-label="Theme"><Moon /></button>
                    <button class="ghost" type="button" aria-label="Menu">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                            <circle cx="5" cy="12" r="1.4"></circle>
                            <circle cx="12" cy="12" r="1.4"></circle>
                            <circle cx="19" cy="12" r="1.4"></circle>
                        </svg>
                    </button>
                </div>
            </div>
            <div class="slot room-slot">
                <span class="center-name">{move || selected.get().name().to_string()}</span>
                <div class="work" role="tablist">
                    <button type="button">"Shell"</button>
                    <button type="button">"Browser"</button>
                    <button type="button">"Agent"</button>
                    <button class="on" type="button">"Manual"</button>
                </div>
                <button class="ghost" type="button" aria-label="Close">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" aria-hidden="true">
                        <path d="M6 6l12 12M18 6L6 18"></path>
                    </svg>
                </button>
            </div>
            <div class="slot">
                <span class="ai-title">"Ask AI"</span>
                <button class="ghost spacer" type="button" aria-label="New chat">
                    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" aria-hidden="true">
                        <path d="M12 5v14M5 12h14"></path>
                    </svg>
                </button>
            </div>
        </header>
    }
}

#[component]
fn BoxList(org: RwSignal<Org>, selected: RwSignal<BoxId>) -> impl IntoView {
    view! {
        <aside class="col side">
            <ul class="list">
                {move || {
                    org.get()
                        .boxes()
                        .iter()
                        .copied()
                        .map(|id| {
                        let on = move || selected.get() == id;
                        view! {
                            <li>
                                <button
                                    class=move || if on() { "box on" } else { "box" }
                                    type="button"
                                    on:click=move |_| {
                                        selected.set(id);
                                    }
                                >
                                    <i class="dot live"></i>
                                    <span class="txt">
                                        <b>{id.name()}</b>
                                        <small>{id.tags().join(" · ")}</small>
                                    </span>
                                </button>
                            </li>
                        }
                    })
                    .collect_view()
                }}
            </ul>
        </aside>
    }
}

fn push_fix(chat: RwSignal<Vec<Line>>, ask: &'static str, cmd: &'static str, reply: &'static str) {
    chat.update(|log| {
        log.push(Line::You(ask));
        log.push(Line::Step(cmd));
        log.push(Line::Ai(reply));
    });
}

#[component]
fn FixList(id: BoxId, chat: RwSignal<Vec<Line>>) -> impl IntoView {
    view! {
        <ul class="fixes">
            {fixes_for(id)
                .into_iter()
                .map(|fix| {
                    let ask = fix.ask;
                    let cmd = fix.cmd;
                    let reply = fix.reply;
                    view! {
                        <li class="fix">
                            <div class="fix-top">
                                <h2>{fix.symptom}</h2>
                                <button
                                    class="ask"
                                    type="button"
                                    on:click=move |_| push_fix(chat, ask, cmd, reply)
                                >
                                    "Ask AI"
                                </button>
                            </div>
                            <div class="cmd"><code>{fix.cmd}</code></div>
                        </li>
                    }
                })
                .collect_view()}
        </ul>
    }
}

#[component]
fn Manual(selected: RwSignal<BoxId>, chat: RwSignal<Vec<Line>>) -> impl IntoView {
    view! {
        <main class="col">
            <div class="scroll">
                {move || {
                    let id = selected.get();
                    let name = id.name();
                    let tags = id.tags();
                    let lede = match id {
                        BoxId::One => "Checks for the Cardano relay and db-sync. Ask AI runs them read only. Do not restart cardano-node or cardano-db-sync.",
                        BoxId::Two => "Checks for the Midnight validator. Ask AI runs them read only. Do not restart midnight-node until db-sync sync_percent is known, and do not regenerate keys.",
                        BoxId::Evm => "Checks for the Cronos EVM node. Ask AI runs them read only. Do not restart cronosd.",
                        BoxId::Pos => "Checks for the Cronos POS validator. Ask AI runs them read only. Do not restart cronos-pos, and do not print a key.",
                    };
                    let facts: &[(&str, &str)] = match id {
                        BoxId::One => &[
                            ("Network", "mainnet"),
                            ("Node unit", "cardano-node.service"),
                            ("db-sync unit", "cardano-db-sync.service"),
                            ("Node socket", "/data/cardano-data/db/node.socket"),
                            ("Data", "/data/cardano-data"),
                            ("Database", "cexplorer"),
                        ],
                        BoxId::Two => &[
                            ("Network", "mainnet"),
                            ("Node unit", "midnight-node.service"),
                            ("Chain spec", "/data/midnight-data/chain-spec-raw.json"),
                            ("Data", "/data/midnight-data"),
                            ("Node key", "/secret/data/chains/midnight/network/secret_ed25519"),
                            ("WireGuard", "/secret/wireguard/publickey"),
                        ],
                        BoxId::Evm => &[
                            ("Network", "mainnet"),
                            ("Chain id", "25"),
                            ("Node unit", "cronosd.service"),
                            ("Chain home", "/data/.home"),
                            ("Node key", "/data/.home/config/node_key.json"),
                            ("Address book", "/data/.home/config/addrbook.json"),
                            ("Genesis", "/data/.home/config/genesis.json"),
                        ],
                        BoxId::Pos => &[
                            ("Network", "mainnet"),
                            ("Chain id", "crypto-org-chain-mainnet-1"),
                            ("Node unit", "cronos-pos.service"),
                            ("Chain home", "/data/.home"),
                            ("Node key", "/data/.home/config/node_key.json"),
                            ("Address book", "/data/.home/config/addrbook.json"),
                            ("Genesis", "/data/.home/config/genesis.json"),
                        ],
                    };
                    view! {
                        <article class="doc" data-box=id.name()>
                            <div class="top">
                                <div>
                                    <h1>{name}</h1>
                                    <TagLine tags=tags />
                                </div>
                                <div class="actions">
                                    <button class="btn" type="button">"Edit"</button>
                                </div>
                            </div>
                            <p class="lede">{lede}</p>
                            <table class="facts">
                                    <tbody>
                                        {facts
                                            .iter()
                                            .map(|(k, v)| {
                                                view! {
                                                    <tr>
                                                        <th scope="row">{*k}</th>
                                                        <td class="mono">{*v}</td>
                                                    </tr>
                                                }
                                            })
                                            .collect_view()}
                                    </tbody>
                                </table>
                            <FixList id chat />
                        </article>
                    }
                }}
            </div>
        </main>
    }
}

#[component]
fn TagLine(tags: &'static [&'static str]) -> impl IntoView {
    view! {
        <ul class="tags" aria-label="AI tags from scan">
            {tags.iter().map(|tag| view! { <li class="tag">{*tag}</li> }).collect_view()}
        </ul>
    }
}

#[component]
fn Ask(chat: RwSignal<Vec<Line>>) -> impl IntoView {
    let draft = RwSignal::new(String::new());
    let log = NodeRef::<leptos::html::Div>::new();
    Effect::new(move |_| {
        chat.track();
        if let Some(el) = log.get() {
            el.set_scroll_top(el.scroll_height());
        }
    });
    view! {
        <aside class="ai-pane" data-testid="ai-chat">
            <div class="chat-log" data-testid="ai-log" node_ref=log>
                <div class="chat-log-spacer"></div>
                {move || {
                    chat.get()
                        .into_iter()
                        .map(|line| match line {
                            Line::Step(text) => view! {
                                <pre class="step">{text}</pre>
                            }.into_any(),
                            Line::You(text) | Line::Ai(text) => {
                                let mine = matches!(line, Line::You(_));
                                view! {
                                    <div class=if mine { "chat chat-end" } else { "chat chat-start" }>
                                        <div class=if mine { "chat-bubble chat-bubble-primary" } else { "chat-bubble" }>{text}</div>
                                    </div>
                                }.into_any()
                            }
                        })
                        .collect_view()
                }}
            </div>
            <form class="composer" on:submit=move |ev| {
                ev.prevent_default();
                let text = draft.get();
                if text.trim().is_empty() {
                    return;
                }
                draft.set(String::new());
            }>
                <input
                    id="ai-q"
                    data-testid="ai-q"
                    autocomplete="off"
                    spellcheck="false"
                    prop:value=move || draft.get()
                    on:input=move |ev| draft.set(event_target_value(&ev))
                />
                <button class="send" type="submit" data-testid="ai-send" aria-label="Send">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                        <path d="m5 12 7-7 7 7"></path>
                        <path d="M12 19V5"></path>
                    </svg>
                </button>
            </form>
        </aside>
    }
}
