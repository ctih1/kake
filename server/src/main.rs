use std::{collections::HashMap, sync::Arc};
use actix::fut::stream;
use actix_web::{post, rt, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_ws::{AggregatedMessage, Session};
use futures::{lock::Mutex, StreamExt as _};


struct AppState {
    connections: Mutex<HashMap<String, Session>>
}

async fn index(data: web::Data<AppState>, _req: HttpRequest) -> impl Responder {
    let conns = data.connections.lock().await;
    println!("Sending data (len: {})", conns.len());
    for mut conn in conns.clone().into_iter() {
        println!("Sending to connection {}", conn.0);
        if let Err(e) = conn.1.text("action=do;param=caps;value=toggle").await {
            println!("Failed to send");
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
        println!("Sending {}={} to connection {}", param,value, conn.0);
        if let Err(e) = conn.1.text(format!("action={};param={};value={}", action, param, value)).await {
            println!("Failed to send");
        }
    }
    drop(conns);
    return HttpResponse::Ok()
}

async fn mouse_ws(data: web::Data<AppState>, req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    println!("Recieved mouse event");
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
                        println!("Sending mouse data to connection {}", conn.0);
                        if let Err(e) = conn.1.text(format!("action=do;param={};value={}",event_type, pos)).await {
                            println!("Failed to send");
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
                    println!("Recieved message {}", text);
                    let mut connections = data.connections.lock().await;
                    let params = parse_params(text.to_string());
                    if let Some(name) = params.get("name") {
                        println!("Registering new user {}", name.clone());
                        connections.insert(name.to_string(), session.clone());
                        println!("New length: {}", connections.len());
                        drop(connections);
                        session.text("success=true").await.unwrap();
                    } else {
                        session.text("success=false").await.unwrap();
                    }

                }

                Ok(AggregatedMessage::Binary(_bin)) => {
                    session.text("Expected text, found bytes").await.unwrap();
                }

                Ok(AggregatedMessage::Ping(msg)) => {
                    session.pong(&msg).await.unwrap();
                }

                Ok(AggregatedMessage::Close(_)) => {
                    println!("Client disconnected");
                    let mut conns = data.connections.lock().await;
                    conns.clear();
                    drop(conns);
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
    let state = web::Data::new(AppState {
                connections: Mutex::new(HashMap::new())
            });
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/", web::get().to(index))
            .service(action)
            .route("/ws", web::get().to(websocket_endpoint))
            .route("/ws/mouse", web::get().to(mouse_ws))

            
            
    })
    .bind(("0.0.0.0", 8000))?
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