pub mod cassette;

use libp2p::futures::StreamExt;
use libp2p::{
    core::upgrade,
    floodsub::{Floodsub, FloodsubEvent, Topic},
    identity,
    mdns::{Mdns, MdnsEvent},
    mplex,
    noise::{Keypair, NoiseConfig, X25519Spec},
    swarm::{NetworkBehaviourEventProcess, Swarm, SwarmBuilder},
    tcp::TokioTcpConfig,
    NetworkBehaviour, PeerId, Transport,
};
use log::{error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tokio::{fs, io::AsyncBufReadExt, sync::mpsc};

const STORAGE_FILE_PATH: &str = "./recipes.json";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;
type Recipes = Vec<Recipe>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Recipe {
    id: usize,
    name: String,
    ingredients: String,
    instructions: String,
    public: bool,
}

pub static KEYS: Lazy<identity::Keypair> = Lazy::new(|| identity::Keypair::generate_ed25519());
pub static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(KEYS.public()));
pub static TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("recipes"));

// #[derive(Debug, Serialize, Deserialize)]
// struct Recipe {
//     id: usize,
//     name: String,
//     ingredients: String,
//     instructions: String,
//     public: bool,
// }

#[derive(Debug, Serialize, Deserialize)]
pub enum ListMode {
    ALL,
    One(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListRequest {
    mode: ListMode,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListResponse {
    mode: ListMode,
    data: Recipes,
    receiver: String,
}

pub enum EventType {
    Response(ListResponse),
    Input(String),
}

#[derive(NetworkBehaviour)]
pub struct RecipeBehaviour {
    pub floodsub: Floodsub,
    pub mdns: Mdns,
    #[behaviour(ignore)]
    pub response_sender: mpsc::UnboundedSender<ListResponse>,
}

impl NetworkBehaviourEventProcess<FloodsubEvent> for RecipeBehaviour {
    fn inject_event(&mut self, event: FloodsubEvent) {
        match event {
            FloodsubEvent::Message(msg) => {
                if let Ok(resp) = serde_json::from_slice::<ListResponse>(&msg.data) {
                    if resp.receiver == PEER_ID.to_string() {
                        info!("Response from {}:", msg.source);
                        resp.data.iter().for_each(|r| info!("{:?}", r));
                    }
                } else if let Ok(req) = serde_json::from_slice::<ListRequest>(&msg.data) {
                    match req.mode {
                        ListMode::ALL => {
                            info!("Received ALL req: {:?} from {:?}", req, msg.source);
                            respond_with_public_recipes(
                                self.response_sender.clone(),
                                msg.source.to_string(),
                            );
                        }
                        ListMode::One(ref peer_id) => {
                            if peer_id == &PEER_ID.to_string() {
                                info!("Received req: {:?} from {:?}", req, msg.source);
                                respond_with_public_recipes(
                                    self.response_sender.clone(),
                                    msg.source.to_string(),
                                );
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }
}

/// # Examples
/// # Panics
/// # Errors
/// # Safety
/// # Aborts
/// # Undefined Behavior
fn respond_with_public_recipes(sender: mpsc::UnboundedSender<ListResponse>, receiver: String) {
    tokio::spawn(async move {
        match cassette::read_local_recipes().await {
            Ok(recipes) => {
                let resp = ListResponse {
                    mode: ListMode::ALL,
                    receiver,
                    data: recipes.into_iter().filter(|r| r.public).collect(),
                };
                if let Err(e) = sender.send(resp) {
                    error!("error sending response via channel, {}", e);
                }
            }
            Err(e) => error!("error fetching local recipes to answer ALL request, {}", e),
        }
    });
}

impl NetworkBehaviourEventProcess<MdnsEvent> for RecipeBehaviour {
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(discovered_list) => {
                for (peer, _addr) in discovered_list {
                    self.floodsub.add_node_to_partial_view(peer);
                }
            }
            MdnsEvent::Expired(expired_list) => {
                for (peer, _addr) in expired_list {
                    if !self.mdns.has_node(&peer) {
                        self.floodsub.remove_node_from_partial_view(&peer);
                    }
                }
            }
        }
    }
}

/// # Examples
/// # Panics
/// # Errors
/// # Safety
/// # Aborts
/// # Undefined Behavior
async fn create_new_recipe(name: &str, ingredients: &str, instructions: &str) -> Result<()> {
    let mut local_recipes = cassette::read_local_recipes().await?;
    let new_id = match local_recipes.iter().max_by_key(|r| r.id) {
        Some(v) => v.id + 1,
        None => 0,
    };
    local_recipes.push(Recipe {
        id: new_id,
        name: name.to_owned(),
        ingredients: ingredients.to_owned(),
        instructions: instructions.to_owned(),
        public: false,
    });
    write_local_recipes(&local_recipes).await?;

    info!("Created recipe:");
    info!("Name: {}", name);
    info!("Ingredients: {}", ingredients);
    info!("Instructions:: {}", instructions);

    Ok(())
}

/// # Examples
/// # Panics
/// # Errors
/// # Safety
/// # Aborts
/// # Undefined Behavior
async fn publish_recipe(id: usize) -> Result<()> {
    let mut local_recipes = cassette::read_local_recipes().await?;
    local_recipes
        .iter_mut()
        .filter(|r| r.id == id)
        .for_each(|r| r.public = true);
    write_local_recipes(&local_recipes).await?;
    Ok(())
}

// /// # Examples
// /// # Panics
// /// # Errors
// /// # Safety
// /// # Aborts
// /// # Undefined Behavior
async fn read_local_recipes() -> Result<Recipes> {
    let content = fs::read(STORAGE_FILE_PATH).await?;
    let result = serde_json::from_slice(&content)?;
    Ok(result)
}

/// # Examples
/// See integration test: []
/// # Panics
/// # Errors
/// # Safety
/// # Aborts
/// # Undefined Behavior
async fn write_local_recipes(recipes: &Recipes) -> Result<()> {
    let json = serde_json::to_string(&recipes)?;
    fs::write(STORAGE_FILE_PATH, &json).await?;
    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn write_local_recipes_unit() {
//         let rcps = Recipes.new();
//         let result = write_local_recipes(rcps);
//         assert_eq!(result, 20);
//     }
// }

async fn handle_list_peers(swarm: &mut Swarm<RecipeBehaviour>) {
    info!("Discovered Peers:");
    let nodes = swarm.behaviour().mdns.discovered_nodes();
    let mut unique_peers = HashSet::new();
    for peer in nodes {
        unique_peers.insert(peer);
    }
    unique_peers.iter().for_each(|p| info!("{}", p));
}

async fn handle_list_recipes(cmd: &str, swarm: &mut Swarm<RecipeBehaviour>) {
    let rest = cmd.strip_prefix("ls r ");
    match rest {
        Some("all") => {
            let req = ListRequest {
                mode: ListMode::ALL,
            };
            let json = serde_json::to_string(&req).expect("can jsonify request");
            swarm
                .behaviour_mut()
                .floodsub
                .publish(TOPIC.clone(), json.as_bytes());
        }
        Some(recipes_peer_id) => {
            let req = ListRequest {
                mode: ListMode::One(recipes_peer_id.to_owned()),
            };
            let json = serde_json::to_string(&req).expect("can jsonify request");
            swarm
                .behaviour_mut()
                .floodsub
                .publish(TOPIC.clone(), json.as_bytes());
        }
        None => {
            match cassette::read_local_recipes().await {
                Ok(v) => {
                    info!("Local Recipes ({})", v.len());
                    v.iter().for_each(|r| info!("{:?}", r));
                }
                Err(e) => error!("error fetching local recipes: {}", e),
            };
        }
    };
}

async fn handle_create_recipe(cmd: &str) {
    if let Some(rest) = cmd.strip_prefix("create r") {
        let elements: Vec<&str> = rest.split("|").collect();
        if elements.len() < 3 {
            info!("too few arguments - Format: name|ingredients|instructions");
        } else {
            let name = elements.get(0).expect("name is there");
            let ingredients = elements.get(1).expect("ingredients is there");
            let instructions = elements.get(2).expect("instructions is there");
            if let Err(e) = create_new_recipe(name, ingredients, instructions).await {
                error!("error creating recipe: {}", e);
            };
        }
    }
}

async fn handle_publish_recipe(cmd: &str) {
    if let Some(rest) = cmd.strip_prefix("publish r") {
        match rest.trim().parse::<usize>() {
            Ok(id) => {
                if let Err(e) = publish_recipe(id).await {
                    info!("error publishing recipe with id {}, {}", id, e)
                } else {
                    info!("Published Recipe with id: {}", id);
                }
            }
            Err(e) => error!("invalid id: {}, {}", rest.trim(), e),
        };
    }
}

async fn send_shutdown(cmd: &str) {}

pub async fn match_command_line(line: String, mut swarm: &mut Swarm<RecipeBehaviour>) {
    match line.as_str() {
        "ls p" => handle_list_peers(&mut swarm).await,
        cmd if cmd.starts_with("ls r") => handle_list_recipes(cmd, &mut swarm).await,
        cmd if cmd.starts_with("create r") => handle_create_recipe(cmd).await,
        cmd if cmd.starts_with("publish r") => handle_publish_recipe(cmd).await,
        cmd if cmd.starts_with("exit") => send_shutdown(cmd).await,
        cmd if cmd.starts_with("quit") => send_shutdown(cmd).await,
        _ => error!("unknown command"),
    }
}

pub fn publish_event_type(resp: ListResponse, swarm: &mut Swarm<RecipeBehaviour>) {
    let json = serde_json::to_string(&resp).expect("can json-ify response");
    swarm
        .behaviour_mut()
        .floodsub
        .publish(TOPIC.clone(), json.as_bytes())
}
