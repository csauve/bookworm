use std::convert::Infallible;
use std::net::{SocketAddr, IpAddr};
use std::time::{SystemTime};
use log::*;
use hyper::{Body, Request, Response, Server, Method, StatusCode, body, service::{make_service_fn, service_fn}};
use crate::api::{ApiSnakeConfig, ApiMove};
use crate::game::get_decision;

pub async fn start_server(ip: IpAddr, port: u16) {
    let addr = SocketAddr::new(ip, port);
    println!("
    ┌────────────────────────────────┐
    │ ╖                              │
    │ ╟─╥ ╓─╖ ╓─╖ ║╓  ╖ ╓ ╓─╖ ╓┐ ╓┬╖ │ v{}
    │ ╜─╜ ╙─╜ ╙─╜ ╜╙─ ╙┴╜ ╙─╜ ╨  ╨ ╨ │ {}
    └────────────────────────────────┘
    ",
    env!("CARGO_PKG_VERSION"),
    &addr
    );

    let server = Server::bind(&addr).serve(make_service_fn(|_socket|
        async {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>|
                async {
                    Ok::<_, Infallible>(match (req.method(), req.uri().path()) {
                        (&Method::GET, "/") => {
                            Response::new(Body::from("What'sssss up?"))
                        },
                        (&Method::POST, "/ping") => {
                            debug!("Handled /ping");
                            Response::new(Body::empty())
                        },
                        (&Method::POST, "/start") => {
                            let json = serde_json::to_string(&ApiSnakeConfig {
                                color: String::from("#800080"),
                                head_type: String::from("sand-worm"),
                                tail_type: String::from("round-bum"),
                            }).unwrap();
                            debug!("Handled /start");
                            Response::builder()
                                .header("Content-Type", "application/json")
                                .body(Body::from(json))
                                .unwrap()
                        },
                        (&Method::POST, "/move") => {
                            let bytes = body::to_bytes(req.into_body()).await.unwrap();
                            match serde_json::from_slice(&bytes) {
                                Ok(game_state) => {
                                    let start = SystemTime::now();
                                    let decision = get_decision(&game_state);
                                    let json = serde_json::to_string(&ApiMove {decision}).unwrap();
                                    let duration = SystemTime::now().duration_since(start).unwrap().as_millis();
                                    info!(
                                        "Handled /move: game={}, turn={}, duration={}ms, move={:?}",
                                        &game_state.game.id,
                                        &game_state.turn,
                                        duration,
                                        &decision
                                    );
                                    Response::builder()
                                        .header("Content-Type", "application/json")
                                        .body(Body::from(json))
                                        .unwrap()
                                },
                                Err(_) => {
                                    Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .header("Content-Type", "text/plain")
                                        .body(Body::from("The request body could not be parsed as valid JSON"))
                                        .unwrap()
                                }
                            }
                        },
                        (&Method::POST, "/end") => {
                            debug!("Handled /end");
                            Response::new(Body::empty())
                        },
                        _ => {
                            Response::builder()
                                .status(StatusCode::NOT_FOUND)
                                .body(Body::empty())
                                .unwrap()
                        }
                    })
                }
            ))
        }
    ));

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
