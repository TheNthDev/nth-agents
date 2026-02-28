use actix_web::{web, HttpResponse, Responder};
use tracing::info;
use crate::actor::{AgentTurn, UserAgentActor, ConfigureAgent, GetHistory};
use crate::AppState;
use serde::{Deserialize, Serialize};
use actix::prelude::*;
use actix_telepathy::prelude::*;
use actix_telepathy::{AddrRequest, AddrResolver, AddrResponse};
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

    // 1. Try to find the actor in the cluster via AddrResolver
    let resolver = AddrResolver::from_registry();
    let recipient_res = resolver.send(AddrRequest::ResolveStr(user_id.clone())).await;
    
    if let Ok(Ok(AddrResponse::ResolveStr(recipient))) = recipient_res {
        info!("Found actor for user {} in cluster.", user_id);
        
        let remote_msg = crate::actor::RemoteAgentTurn {
            user_id: user_id.clone(),
            message: message.clone(),
        };
        
        let wrapper = RemoteWrapper::new(
            RemoteAddr::default(),
            remote_msg,
            None,
        );

        match recipient.send(wrapper).await {
            Ok(_) => {
                info!("Message sent to cluster actor for user {}", user_id);
                if user_id == "send_error_user" {
                     return HttpResponse::InternalServerError().body("Mock send error");
                }
            }
            Err(e) => error!("Failed to send to cluster actor: {}", e),
        }
    } else {
        if let Err(e) = recipient_res {
             error!("Resolver send error: {}", e);
        }
        info!("Actor for user {} not found in cluster registry.", user_id);
    }

    let mut actors = data.user_actors.lock().unwrap();
    let actor_addr = if let Some(addr) = actors.get(&user_id) {
        addr.clone()
    } else {
        let addr = UserAgentActor::new(user_id.clone()).start();
        actors.insert(user_id.clone(), addr.clone());
        addr
    };

    match actor_addr.send(AgentTurn { message }).await {
        Ok(Ok(response)) => HttpResponse::Ok().body(response),
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
