use actix_web::{web, HttpResponse, Responder};
use tracing::info;
use crate::actor::{AgentTurn, UserAgentActor, ConfigureAgent, GetHistory, GetConfig};
use crate::AppState;
use serde::{Deserialize, Serialize};
use actix::prelude::*;
use actix_telepathy::prelude::*;
use actix_telepathy::AddrResolver;
use tracing::error;

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
        llm_api_key: req.llm_api_key.clone(),
        weather_api_key: req.weather_api_key.clone(),
    };

    match actor_addr.send(config_msg).await {
        Ok(Ok(_)) => HttpResponse::Ok().body("User configured"),
        Ok(Err(e)) => HttpResponse::InternalServerError().body(e.to_string()),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
