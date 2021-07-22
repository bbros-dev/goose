use lol_core::proto_compiled;
use std::time::Duration;
use structopt::StructOpt;
use tonic::transport::channel::Endpoint;

#[derive(StructOpt, Debug)]
#[structopt(name = "raft-access")]
struct Opt {
    #[structopt(name = "ID")]
    id: String,
    #[structopt(subcommand)]
    sub: Sub,
}
#[derive(Debug, StructOpt)]
enum Sub {
    #[structopt(name = "get")]
    Get {
        #[structopt(name = "KEY")]
        key: String,
    },
    #[structopt(name = "set")]
    Set {
        #[structopt(name = "KEY")]
        key: String,
        #[structopt(name = "VALUE")]
        value: String,
        #[structopt(long = "rep", default_value = "1")]
        rep: u32,
    },
    #[structopt(name = "list")]
    List,
}
#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    let endpoint = Endpoint::from_shared(opt.id)
        .unwrap()
        .timeout(Duration::from_secs(5));
    let mut conn = lol_core::connection::connect(endpoint).await.unwrap();
    match opt.sub {
        Sub::Get { key } => {
            let msg = regatta::raft::Req::Get { key };
            let msg = regatta::raft::Req::serialize(&msg);
            let res = conn
                .request_apply(proto_compiled::ApplyReq {
                    core: false,
                    message: msg,
                    mutation: false,
                })
                .await
                .unwrap()
                .into_inner();
            let res = regatta::raft::Rep::deserialize(&res.message).unwrap();
            let res = if let regatta::raft::Rep::Get { found, value } = res {
                if found {
                    regatta::raft::Get(Some(value))
                } else {
                    regatta::raft::Get(None)
                }
            } else {
                unreachable!()
            };
            let json = serde_json::to_string(&res).unwrap();
            println!("{}", json);
        }
        Sub::Set { key, value, rep } => {
            let mut value_rep = String::new();
            for _ in 0..rep {
                value_rep.push_str(&value)
            }
            let msg = regatta::raft::Req::Set {
                key,
                value: value_rep,
            };
            let msg = regatta::raft::Req::serialize(&msg);
            conn.request_commit(proto_compiled::CommitReq {
                core: false,
                message: msg,
            })
            .await
            .unwrap();
            println!("OK");
        }
        Sub::List => {
            let msg = regatta::raft::Req::List;
            let msg = regatta::raft::Req::serialize(&msg);
            let res = conn
                .request_apply(proto_compiled::ApplyReq {
                    core: false,
                    message: msg,
                    mutation: false,
                })
                .await
                .unwrap()
                .into_inner();
            let res = regatta::raft::Rep::deserialize(&res.message).unwrap();
            let res = if let regatta::raft::Rep::List { values } = res {
                regatta::raft::List(values)
            } else {
                unreachable!()
            };
            let json = serde_json::to_string(&res).unwrap();
            println!("{}", json);
        }
    }
}
