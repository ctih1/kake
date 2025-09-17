use std::{collections::{HashMap, HashSet}, hash::Hash, sync::Arc, time::{Duration, SystemTime}};
use actix_web::{delete, get, http::uri::Authority, post, rt::{self, System}, web::{self, to, Redirect}, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_ws::{AggregatedMessage, Session};
use futures::{lock::Mutex, StreamExt as _};
use log::{info, warn};
use rand::{distr::Alphanumeric, Rng};

struct AppState {
    connections: Mutex<HashMap<String, Session>>,
    ping_reqs: Mutex<HashMap<String, SystemTime>>,
    pings: Mutex<HashMap<String, Duration>>,
    last_contacts: Mutex<HashMap<String, SystemTime>>,
    // The session id and whether they are an admin (first session is considered admin)
    authorizations: Mutex<HashMap<String, bool>>
}

async fn index(data: web::Data<AppState>, _req: HttpRequest) -> impl Responder {
    let mut conns = data.connections.lock().await;
    let mut last_contacts = data.last_contacts.lock().await;
    info!("Sending data (len: {})", conns.len());
    for mut conn in conns.clone().into_iter() {
        info!("Sending to connection {}", conn.0);
        
        last_contacts.insert(conn.0.clone(), SystemTime::now());
        if let Err(e) = conn.1.text("action=do;param=caps;value=toggle").await {
            let mut pings = data.pings.lock().await;
            let mut ping_reqs = data.ping_reqs.lock().await;

            warn!("Failed to send {}", e);
            conns.remove(&conn.0);
            pings.remove(&conn.0);
            ping_reqs.remove(&conn.0);
        }
    }
    drop(conns);
    return HttpResponse::Ok().body(include_str!("website/index.html"))
}

async fn login(_data: web::Data<AppState>, _req: HttpRequest) -> impl Responder {
    return HttpResponse::Ok().body(include_str!("website/login.html"));
}

async fn create_session(data: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let mut authorized = data.authorizations.lock().await;
    if let Some(cookie) = req.cookie("auth") && *authorized.get(cookie.to_string().split("=").last().unwrap()).unwrap_or(&false) {
        info!("Recieved session creation");
    } else {
        warn!("Auth required");
        return HttpResponse::Forbidden().into();
    }

    let code = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(14)
        .map(char::from)
        .collect::<String>();

    authorized.insert(code.clone(), false);
    return HttpResponse::Ok().body(code);
}

#[delete("/api/session")] 
async fn remove_session(data: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let mut authorized = data.authorizations.lock().await;
    if let Some(cookie) = req.cookie("auth") && *authorized.get(cookie.to_string().split("=").last().unwrap()).unwrap_or(&false) {
        info!("Recieved session deletion");
    } else {
        warn!("Auth required");
        return HttpResponse::Forbidden().into();
    }

    let readable_auths = authorized.clone();
    for key in readable_auths.keys().clone().into_iter() {
        if !readable_auths.get(&key.clone()).unwrap_or(&false) {
            authorized.remove(&key.to_string());
        }
    }
    return HttpResponse::Ok();
}


#[post("/send/{param}/{value}")] 
async fn action(data: web::Data<AppState>, req: HttpRequest, path: web::Path<(String, String)>) -> impl Responder {
    let (param, value) = path.into_inner();
    
    let mut authorized = data.authorizations.lock().await;
    if let Some(cookie) = req.cookie("auth") && authorized.contains_key(cookie.to_string().split("=").last().unwrap()) {
        info!("Recieved send POST");
    } else {
        warn!("Auth required");
        return HttpResponse::Forbidden().into();
    }

    drop(authorized);


    let conns = data.connections.lock().await;
    for mut conn in conns.clone().into_iter() {
        info!("({}): Sending {}={} to connection {}", req.connection_info().peer_addr().unwrap_or("None"), param,value, conn.0);
        if let Err(e) = conn.1.text(format!("param={};value={}", param, value)).await {
            warn!("Failed to send");
        }
    }
    drop(conns);
    return HttpResponse::Ok();
}

async fn data_ws(data: web::Data<AppState>, req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));

    
    let authorized = data.authorizations.lock().await;
    if let Some(cookie) = req.cookie("auth") && authorized.contains_key(cookie.to_string().split("=").last().unwrap()) {
        info!("Recieved dataWs event");
    } else {
        warn!("Auth required");
        let _ = session.text("auth_required=true").await;
        return Ok(res);
    }
    drop(authorized);

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    let mut conns = data.connections.lock().await;
                    let mut ping_reqs = data.ping_reqs.lock().await;
                    for mut conn in conns.clone().into_iter() {
                        ping_reqs.insert(conn.0.clone(), std::time::SystemTime::now());
                        if let Err(e) = conn.1.text("ping").await {
                            warn!("Failed to send to {}", &conn.0);
                            let mut pings = data.pings.lock().await;
                            pings.remove(&conn.0);
                            ping_reqs.remove(&conn.0);
                            conns.remove(&conn.0);
                            drop(pings);
                        }
                    }

                    let mut message = String::new();
                    message += &format!("conns={}", conns.len());

                    drop(conns);
                    drop(ping_reqs);
                    let pings = data.pings.lock().await;
                    for ping in pings.clone().into_iter() {
                        let millis = ping.1.as_millis();
                        message += &format!(";{}={}", ping.0, millis);
                    }

                    let _ = session.text(message).await;
                }
                Ok(AggregatedMessage::Binary(_bin)) => {
                    session.text("Expected text, found bytes").await.unwrap();
                }
                Ok(AggregatedMessage::Ping(msg)) => {
                    session.pong(&msg).await.unwrap();
                }
                _ => {}
            }
        }
    });

    Ok(res)
}

async fn mouse_ws(data: web::Data<AppState>, req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;
    
    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));


    let authorized = data.authorizations.lock().await;
    if let Some(cookie) = req.cookie("auth") && authorized.contains_key(cookie.to_string().split("=").last().unwrap()) {
        info!("Recieved mouseWs event");
    } else {
        warn!("Auth required");
        let _ = session.text("auth_required=true").await;
        return Ok(res);
    }

    drop(authorized);

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    let conns = data.connections.lock().await;

                    let params = parse_params(text.to_string());
                    let pos = params.get("pos").unwrap();
                    let event_type = params.get("type").unwrap();
                    for mut conn in conns.clone().into_iter() {
                        info!("Sending mouse data to connection {}", conn.0);
                        if let Err(e) = conn.1.text(format!("param={};value={}",event_type, pos)).await {
                            warn!("Failed to send {}", e);
                        }
                    }

                    drop(conns);
                }
                Ok(AggregatedMessage::Binary(_bin)) => {
                    session.text("Expected text, found bytes").await.unwrap();
                }

                Ok(AggregatedMessage::Ping(msg)) => {
                    session.pong(&msg).await.unwrap();
                }
                _ => {}
            }
        }
    });

    Ok(res)
}

async fn websocket_endpoint(data: web::Data<AppState>, req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;
    
    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));

    

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    info!("Recieved message {}", text);
                    if text.starts_with("username=") {
                        let username = text.split("username=").last();

                        if let Some(username) = username {
                            let ping_reqs = data.ping_reqs.lock().await;
                            let ping_req = ping_reqs.get(username);

                            if let Some(ping_req) = ping_req {
                                let mut pings = data.pings.lock().await;
                                
                                pings.insert(username.to_string(),ping_req.elapsed().unwrap());
                            }
                            
                        }
                    }
                    let mut connections = data.connections.lock().await;
                    let params = parse_params(text.to_string());
                    if let Some(name) = params.get("name") {
                        info!("Registering new user {}", name.clone());
                        connections.insert(name.to_string(), session.clone());
                        info!("New length: {}", connections.len());
                        drop(connections);
                        session.text("success=true").await.unwrap();
                    }
                }

                Ok(AggregatedMessage::Binary(_bin)) => {
                    session.text("Expected text, found bytes").await.unwrap();
                }

                Ok(AggregatedMessage::Ping(msg)) => {
                    session.pong(&msg).await.unwrap();
                }

                Ok(AggregatedMessage::Close(_)) => {
                    info!("Client disconnected");
                }

                _ => {}
            }
        }
    });

    // respond immediately with response connected to WS session
    Ok(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Debug).init();
    let mut auths = Mutex::new(HashMap::new());
    let code = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(14)
        .map(char::from)
        .collect::<String>();
    let mut auths_lock = auths.lock().await;
    auths_lock.insert(code.clone(), true);
    info!("Admin authentication code: {code}");
    drop(auths_lock);

    let state = web::Data::new(AppState {
                connections: Mutex::new(HashMap::new()),
                pings: Mutex::new(HashMap::new()),
                ping_reqs: Mutex::new(HashMap::new()),
                last_contacts: Mutex::new(HashMap::new()),
                authorizations: auths
            });
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/", web::get().to(index))
            .route("/api/session", web::post().to(create_session))
            .route("/login", web::get().to(login))
            .service(action)
            .service(remove_session)
            .route("/ws", web::get().to(websocket_endpoint))
            .route("/ws/mouse", web::get().to(mouse_ws))
            .route("/ws/info", web::get().to(data_ws))
    })
    .bind(("0.0.0.0", 8040))?
    .run()
    .await
}

pub fn parse_params(query_string: String) -> HashMap<String, String> {
    let mut result = HashMap::new();
    for arg in query_string.split(";") {
        let parts: Vec<&str> = arg.split("=").collect();
        
        if parts.len() != 2 {
            continue;
        }
        
        let [key, value] = parts.as_slice().try_into().unwrap();
        result.insert(key.to_string(), value.to_string());
    }

    return result;
}