use std::{collections::HashMap, sync::Arc, time::{Duration, SystemTime}};
use actix_web::{post, rt, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_ws::{AggregatedMessage, Session};
use futures::{lock::Mutex, StreamExt as _};
use log::{info, warn};

struct AppState {
    connections: Mutex<HashMap<String, Session>>,
    ping_reqs: Mutex<HashMap<String, SystemTime>>,
    pings: Mutex<HashMap<String, Duration>>
}

async fn index(data: web::Data<AppState>, _req: HttpRequest) -> impl Responder {
    let conns = data.connections.lock().await;
    info!("Sending data (len: {})", conns.len());
    for mut conn in conns.clone().into_iter() {
        info!("Sending to connection {}", conn.0);
        if let Err(e) = conn.1.text("action=do;param=caps;value=toggle").await {
            warn!("Failed to send {}", e);
        }
    }
    drop(conns);
    return HttpResponse::Ok().body(include_str!("website/index.html"))
}

#[post("/send/{action}/{param}/{value}")] 
async fn action(data: web::Data<AppState>, _req: HttpRequest, path: web::Path<(String, String, String)>) -> impl Responder {
    let (action, param, value) = path.into_inner();

    let conns = data.connections.lock().await;
    for mut conn in conns.clone().into_iter() {
        info!("Sending {}={} to connection {}", param,value, conn.0);
        if let Err(e) = conn.1.text(format!("action={};param={};value={}", action, param, value)).await {
            warn!("Failed to send");
        }
    }
    drop(conns);
    return HttpResponse::Ok()
}

async fn data_ws(data: web::Data<AppState>, req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    info!("Recieved data event");
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;
    
    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    let mut conns = data.connections.lock().await;
                    let mut ping_reqs = data.ping_reqs.lock().await;
                    for mut conn in conns.clone().into_iter() {
                        ping_reqs.insert(conn.0.clone(), std::time::SystemTime::now());
                        if let Err(e) = conn.1.text("ping").await {
                            warn!("Failed to send");
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
    info!("Recieved mouse event");
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;
    
    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));

    

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
                        if let Err(e) = conn.1.text(format!("action=do;param={};value={}",event_type, pos)).await {
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

    let state = web::Data::new(AppState {
                connections: Mutex::new(HashMap::new()),
                pings: Mutex::new(HashMap::new()),
                ping_reqs: Mutex::new(HashMap::new()),
            });
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/", web::get().to(index))
            .service(action)
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