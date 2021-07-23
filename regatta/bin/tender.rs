// use tender::Event;
// use regatta::tender::app::App;

#[macro_use]
extern crate clap;
extern crate clap_verbosity_flag;

use futures::stream;
use futures::StreamExt;
use lol_core::connection::{self, gateway};
use lol_core::{core_message, proto_compiled};
use std::collections::{HashMap, HashSet};
use std::io;
use std::time::Duration;
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tokio::sync::watch;
use tonic::transport::channel::Endpoint;
use tui::{backend::TermionBackend, Terminal};

// #[derive(Clap)]
#[derive(Debug, PartialEq)]
struct Opts {
    id: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let opts: Opts = Opts::parse();
    let opts = clap::App::new("tender")
                          .version("0.1.0")
                          .arg(clap::Arg::with_name("id")
                               .help("Set the peer ID to use")
                               .required(true)
                               .index(1))
                          .arg(clap::Arg::with_name("v")
                               .short("v")
                               .multiple(true)
                               .help("Set the level of verbosity"))
                              .get_matches();

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = regatta::tender::Events::new();

    let mut initial = HashSet::new();
    let id = opts.value_of("id")
                        .expect("Peer ID is required")
                        .to_string();
    initial.insert(id);
    let gateway = gateway::watch(initial);

    let s1_0 = stream::unfold(gateway.clone(), |gateway| async move {
        let endpoints = gateway.borrow().list.clone();
        let res = gateway::exec(endpoints, |id| async move {
            let msg = core_message::Req::ClusterInfo;
            let req = proto_compiled::ProcessReq {
                message: core_message::Req::serialize(&msg),
                core: true,
            };
            let endpoint = Endpoint::from_shared(id)
                .unwrap()
                .timeout(Duration::from_secs(1));
            let mut conn = connection::connect(endpoint).await?;
            let res = conn.request_process(req).await?.into_inner();
            let msg = core_message::Rep::deserialize(&res.message).unwrap();
            if let core_message::Rep::ClusterInfo {
                leader_id,
                membership,
            } = msg
            {
                Ok(regatta::tender::app::Membership {
                    leader_id,
                    membership,
                })
            } else {
                unreachable!()
            }
        })
        .await;
        match res {
            Ok(x) => Some((x, gateway)),
            Err(_) => None,
        }
    });
    let (tx1, s1_1) = watch::channel(regatta::tender::app::Membership {
        leader_id: None,
        membership: vec![],
    });
    tokio::spawn(async move {
        tokio::pin!(s1_0);
        while let Some(x) = s1_0.next().await {
            tx1.send(x);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
    let it1 = std::iter::repeat_with(|| s1_1.borrow().clone());
    let s1 = async_stream::stream! {
        for x in it1.into_iter() {
            yield x
        }
    };
    tokio::pin!(s1);

    let s2_0 = stream::unfold(gateway.clone(), |gateway| async move {
        let mut h = HashMap::new();

        let endpoints = gateway.borrow().list.clone();
        let res = gateway::parallel(endpoints.clone(), |id| async move {
            let msg = core_message::Req::LogInfo;
            let req = proto_compiled::ProcessReq {
                message: core_message::Req::serialize(&msg),
                core: true,
            };
            let endpoint = Endpoint::from_shared(id)
                .unwrap()
                .timeout(Duration::from_secs(1));
            let mut conn = connection::connect(endpoint).await?;
            let res = conn.request_process_locally(req).await?.into_inner();
            let msg = core_message::Rep::deserialize(&res.message).unwrap();
            if let core_message::Rep::LogInfo {
                snapshot_index,
                last_applied,
                commit_index,
                last_log_index,
            } = msg
            {
                Ok(regatta::tender::app::LogInfo {
                    snapshot_index,
                    last_applied,
                    commit_index,
                    last_log_index,
                })
            } else {
                unreachable!()
            }
        })
        .await;

        let n = endpoints.len();
        for i in 0..n {
            let id = &endpoints[i];
            if let Ok(x) = &res[i] {
                h.insert(id.to_owned(), x.clone());
            }
        }
        Some((h, gateway))
    });
    let (tx2, s2_1) = watch::channel(HashMap::new());
    tokio::spawn(async move {
        tokio::pin!(s2_0);
        while let Some(x) = s2_0.next().await {
            tx2.send(x);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
    let it2 = std::iter::repeat_with(|| s2_1.borrow().clone());
    let s2 = async_stream::stream! {
        for x in it2.into_iter() {
            yield x
        }
    };
    tokio::pin!(s2);

    let s3_0 = stream::unfold(gateway.clone(), |gateway| async move {
        let mut h = HashSet::new();

        let endpoints = gateway.borrow().list.clone();
        let res = gateway::parallel(endpoints.clone(), |id| async move {
            let msg = core_message::Req::HealthCheck;
            let req = proto_compiled::ProcessReq {
                message: core_message::Req::serialize(&msg),
                core: true,
            };
            let endpoint = Endpoint::from_shared(id)
                .unwrap()
                .timeout(Duration::from_secs(1));
            let mut conn = connection::connect(endpoint).await?;
            let res = conn.request_process_locally(req).await?.into_inner();
            let msg = core_message::Rep::deserialize(&res.message).unwrap();
            if let core_message::Rep::HealthCheck { ok } = msg {
                Ok(regatta::tender::app::HealthCheck { ok })
            } else {
                unreachable!()
            }
        })
        .await;

        let n = endpoints.len();
        for i in 0..n {
            let id = &endpoints[i];
            if let Ok(regatta::tender::app::HealthCheck { ok }) = &res[i] {
                if *ok {
                    h.insert(id.to_owned());
                }
            }
        }
        Some((h, gateway))
    });
    let (tx3, s3_1) = watch::channel(HashSet::new());
    tokio::spawn(async move {
        tokio::pin!(s3_0);
        while let Some(x) = s3_0.next().await {
            tx3.send(x);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
    let it3 = std::iter::repeat_with(|| s3_1.borrow().clone());
    let s3 = async_stream::stream! {
        for x in it3.into_iter() {
            yield x
        }
    };
    tokio::pin!(s3);

    let mut app = regatta::tender::app::App::new(s1, s2, s3).await;

    loop {
        if !app.running {
            break;
        }

        let model = app.make_model().await;
        terminal.draw(|f| regatta::tender::ui::draw(f, model));

        tokio::time::sleep(Duration::from_millis(100)).await;

        if let Ok(evt) = events.next() {
            match evt {
                regatta::tender::Event::Input(key) => match key {
                    Key::Char(c) => app.on_key(c),
                    _ => {}
                },
            }
        }
    }
    Ok(())
}
