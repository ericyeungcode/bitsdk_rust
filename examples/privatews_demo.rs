use std::env;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

// Include the sibling file
#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() {
    let bit_cli = common::build_rest_client();
    let rsp = bit_cli.ws_auth().await.unwrap();
    println!("ws_auth rsp: {:?}", &rsp);
    let ws_token = rsp["data"]["token"].as_str().unwrap();

    let ws_host = env::var("BITCOM_WS_HOST").unwrap();
    println!("Connecting to WebSocket server: {}", ws_host);

    let ws_stream = connect_async(ws_host).await.unwrap_or_else(|e| {
        if let tokio_tungstenite::tungstenite::Error::Http(resp) = e {
            let status = resp.status();
            let body = String::from_utf8(resp.into_body().unwrap())
                .unwrap_or("<non-UTF8 body>".to_string());
            panic!("HTTP {}: {}", status, body);
        } else {
            panic!("Connection failed: {}", e);
        }
    });

    println!("WebSocket connection established");

    let (mut write, mut read) = ws_stream.0.split();

    let sub_msg = serde_json::json!({
        "type": "subscribe",
        "channels": ["um_account"],
        "interval": "100ms",
        "token": ws_token,
    });

    let json = serde_json::to_string(&sub_msg).unwrap();
    println!("Sending message: {}", json);

    write.send(Message::Text(json.into())).await.unwrap();

    while let Some(Ok(Message::Text(text))) = read.next().await {
        println!("Received: {}", text);
    }
}

