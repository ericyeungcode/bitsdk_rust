use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};


#[tokio::main]
async fn main() {
    // test public data from production host
    let url = "wss://ws.bit.com";
    println!("Connecting to WebSocket server: {}", url);

    let ws_stream = connect_async(url).await.unwrap_or_else(|e| {
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
        "channels": ["ticker"],
        "instruments": ["BTC-USDT-PERPETUAL"],
        "interval": "100ms",
    });    

    let json = serde_json::to_string(&sub_msg).unwrap();
    println!("Sending message: {}", json);

    write.send(Message::Text(json.into())).await.unwrap();

    while let Some(Ok(Message::Text(text))) = read.next().await {
        println!("Received: {}", text);
    }
}

