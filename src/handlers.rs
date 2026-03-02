use actix_web::{web, HttpResponse, Responder, HttpRequest, Error};
use actix_web_actors::ws;
use tracing::info;
use crate::actor::{AgentTurn, AgentStreamTurn, UserAgentActor, ConfigureAgent, GetHistory, GetConfig, StreamChunk, ClearHistory};
use crate::AppState;
use serde::{Deserialize, Serialize};
use actix::prelude::*;

#[derive(Deserialize, Serialize)]
pub struct TurnRequest {
    pub message: String,
}

#[derive(Deserialize, Serialize)]
pub struct SignupRequest {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub tools: Vec<String>,
    pub base_url: Option<String>,
    pub system_prompt: Option<String>,
    pub llm_api_key: Option<String>,
    pub weather_api_key: Option<String>,
}

pub async fn signup(
    user_id: web::Path<String>,
    req: web::Json<SignupRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let user_id = user_id.into_inner();
    info!("Signup for user: {}", user_id);

    let mut actors = data.user_actors.lock().unwrap();
    if actors.contains_key(&user_id) {
        return HttpResponse::Conflict().body("User already exists");
    }

    let addr = UserAgentActor::new(user_id.clone()).start();
    
    // Configure the actor
    let config_msg = ConfigureAgent {
        provider: req.provider.clone(),
        model: req.model.clone(),
        tools: req.tools.clone(),
        base_url: req.base_url.clone(),
        system_prompt: req.system_prompt.clone(),
        llm_api_key: req.llm_api_key.clone(),
        weather_api_key: req.weather_api_key.clone(),
    };

    match addr.send(config_msg).await {
        Ok(Ok(_)) => {
            actors.insert(user_id, addr);
            HttpResponse::Ok().body("User signed up and configured")
        }
        Ok(Err(e)) => HttpResponse::InternalServerError().body(e.to_string()),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn agent_turn(
    user_id: web::Path<String>,
    req: web::Json<TurnRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let user_id = user_id.into_inner();
    let message = req.message.clone();

    if user_id == "force_routing_error" {
        return HttpResponse::InternalServerError().body("Forced routing error");
    }

    info!("Routing turn for user: {} to their agent actor.", user_id);

    let mut actors = data.user_actors.lock().unwrap();
    let actor_addr = if let Some(addr) = actors.get(&user_id) {
        addr.clone()
    } else {
        let addr = UserAgentActor::new(user_id.clone()).start();
        actors.insert(user_id.clone(), addr.clone());
        addr
    };

    match actor_addr.send(AgentTurn { message }).await {
        Ok(Ok(response)) => HttpResponse::Ok().json(response),
        Ok(Err(e)) => HttpResponse::InternalServerError().body(e.to_string()),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_history(
    user_id: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let user_id = user_id.into_inner();
    info!("Fetching history for user: {}", user_id);

    let mut actors = data.user_actors.lock().unwrap();
    let actor_addr = if let Some(addr) = actors.get(&user_id) {
        addr.clone()
    } else {
        let addr = UserAgentActor::new(user_id.clone()).start();
        actors.insert(user_id.clone(), addr.clone());
        addr
    };

    match actor_addr.send(GetHistory).await {
        Ok(history) => HttpResponse::Ok().json(history),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_config(
    user_id: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let user_id = user_id.into_inner();
    info!("Fetching configuration for user: {}", user_id);

    let mut actors = data.user_actors.lock().unwrap();
    let actor_addr = if let Some(addr) = actors.get(&user_id) {
        addr.clone()
    } else {
        let addr = UserAgentActor::new(user_id.clone()).start();
        actors.insert(user_id.clone(), addr.clone());
        addr
    };

    match actor_addr.send(GetConfig).await {
        Ok(Ok(config)) => HttpResponse::Ok().json(config),
        Ok(Err(e)) => HttpResponse::NotFound().body(e.to_string()),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn check_user(
    user_id: web::Path<String>,
) -> impl Responder {
    let user_id = user_id.into_inner();
    info!("Checking existence of user: {}", user_id);
    let config_path = format!("memory/{}/config.json", user_id);
    
    if std::path::Path::new(&config_path).exists() {
        HttpResponse::Ok().body("User exists")
    } else {
        HttpResponse::NotFound().body("User not found")
    }
}

pub async fn clear_history(
    user_id: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let user_id = user_id.into_inner();
    info!("Clearing history for user: {}", user_id);

    let mut actors = data.user_actors.lock().unwrap();
    let actor_addr = if let Some(addr) = actors.get(&user_id) {
        addr.clone()
    } else {
        let addr = UserAgentActor::new(user_id.clone()).start();
        actors.insert(user_id.clone(), addr.clone());
        addr
    };

    match actor_addr.send(ClearHistory).await {
        Ok(Ok(_)) => HttpResponse::Ok().body("History cleared"),
        Ok(Err(e)) => HttpResponse::InternalServerError().body(e.to_string()),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn configure_agent(
    user_id: web::Path<String>,
    req: web::Json<SignupRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let user_id = user_id.into_inner();
    info!("Configuring agent for user: {}", user_id);

    let mut actors = data.user_actors.lock().unwrap();
    let actor_addr = if let Some(addr) = actors.get(&user_id) {
        addr.clone()
    } else {
        let addr = UserAgentActor::new(user_id.clone()).start();
        actors.insert(user_id.clone(), addr.clone());
        addr
    };

    let config_msg = ConfigureAgent {
        provider: req.provider.clone(),
        model: req.model.clone(),
        tools: req.tools.clone(),
        base_url: req.base_url.clone(),
        system_prompt: req.system_prompt.clone(),
        llm_api_key: req.llm_api_key.clone(),
        weather_api_key: req.weather_api_key.clone(),
    };

    match actor_addr.send(config_msg).await {
        Ok(Ok(_)) => HttpResponse::Ok().body("User configured"),
        Ok(Err(e)) => HttpResponse::InternalServerError().body(e.to_string()),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

fn chunk_response(content: &str) -> Vec<String> {
    let words: Vec<&str> = content.split_whitespace().collect();
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    
    for word in words {
        if current_chunk.len() + word.len() + 1 > 20 {
            if !current_chunk.is_empty() {
                chunks.push(current_chunk);
                current_chunk = String::new();
            }
        }
        if !current_chunk.is_empty() {
            current_chunk.push(' ');
        }
        current_chunk.push_str(word);
    }
    
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }
    
    if chunks.is_empty() {
        chunks.push(content.to_string());
    }
    
    chunks
}

pub struct WsStreamActor {
    pub user_agent_actor: Addr<UserAgentActor>,
}

impl Actor for WsStreamActor {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsStreamActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                let text = text.to_string();
                let actor_clone = self.user_agent_actor.clone();
                
                let stream_turn = AgentStreamTurn { message: text };
                
                let fut = actor_clone.send(stream_turn);
                
                fut.into_actor(self)
                    .map(|res, _act, ctx| {
                        match res {
                            Ok(Ok(response)) => {
                                let chunks = chunk_response(&response.content);
                                let total = chunks.len();
                                for (i, chunk) in chunks.into_iter().enumerate() {
                                    let is_last = i == total - 1;
                                    let stream_chunk = StreamChunk {
                                        content: chunk,
                                        done: is_last,
                                        timestamp: response.timestamp.clone(),
                                    };
                                    if let Ok(json) = serde_json::to_string(&stream_chunk) {
                                        ctx.text(json);
                                    }
                                }
                            }
                            Ok(Err(e)) => {
                                let error_chunk = StreamChunk {
                                    content: format!("Error: {}", e),
                                    done: true,
                                    timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                };
                                if let Ok(json) = serde_json::to_string(&error_chunk) {
                                    ctx.text(json);
                                }
                            }
                            Err(e) => {
                                let error_chunk = StreamChunk {
                                    content: format!("Connection error: {}", e),
                                    done: true,
                                    timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                };
                                if let Ok(json) = serde_json::to_string(&error_chunk) {
                                    ctx.text(json);
                                }
                            }
                        }
                    })
                    .wait(ctx);
            }
            Ok(ws::Message::Binary(_)) => (),
            _ => (),
        }
    }
}

pub async fn ws_stream(
    req: HttpRequest,
    stream: web::Payload,
    user_id: web::Path<String>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let user_id = user_id.into_inner();
    info!("[WS] Streaming request for user: {}", user_id);
    
    let mut actors = data.user_actors.lock().unwrap();
    let actor_addr = if let Some(addr) = actors.get(&user_id) {
        addr.clone()
    } else {
        let addr = UserAgentActor::new(user_id.clone()).start();
        actors.insert(user_id.clone(), addr.clone());
        addr
    };
    
    ws::start(
        WsStreamActor {
            user_agent_actor: actor_addr,
        },
        &req,
        stream,
    )
}
