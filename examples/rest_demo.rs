use std::thread;
use std::time::Duration;
// Include the sibling file
#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() {
    let bit_cli = common::build_rest_client();

    println!("==============================");
    println!("Query UM account mode");
    match bit_cli.get_um_account_mode().await {
        Ok(response) => println!("Response: {:?}", response),
        Err(e) => eprintln!("Error: {}", e),
    }

    // query positions
    println!("==============================");    
    println!("Query positions");
    match bit_cli.linear_get_positions(&mut serde_json::json!({
        "currency": "USDT",
    })).await {
        Ok(response) => println!("Response: {:?}", response),
        Err(e) => eprintln!("Error: {}", e),
    }

    // new batch orders
    println!("==============================");    
    println!("New batch orders");
    let mut batch_new_req = serde_json::json!({
        "currency": "USDT",
        "orders_data": [
            {
                "instrument_id": "BTC-USDT-PERPETUAL",
                "price": "20000",
                "qty": "1.2",
                "side": "buy"
            },
            {
                "instrument_id": "ETH-USDT-PERPETUAL",
                "price": "1800",
                "qty": "23",
                "side": "buy"
            }
        ]
    });

    println!("New batch req: {}", batch_new_req.to_string());
    match bit_cli.linear_batch_new_orders( &mut batch_new_req).await {
        Ok(response) => println!("Response: {:?}", response),
        Err(e) => eprintln!("Error: {}", e),
    }

    // query open orders
    println!("==============================");    
    println!("Query open orders");
    match bit_cli.linear_get_open_orders(&mut serde_json::json!({
        "currency": "USDT",
    })).await {
        Ok(response) => println!("Response: {:?}", response),
        Err(e) => eprintln!("Error: {}", e),
    }

    // cancel orders
    thread::sleep(Duration::from_secs(1));
    println!("Cancel orders: {}", batch_new_req.to_string());
    match bit_cli.linear_cancel_order( &mut serde_json::json!({
        "currency": "USDT"
    })).await {
        Ok(response) => println!("Response: {:?}", response),
        Err(e) => eprintln!("Error: {}", e),
    }

    // query open orders
    thread::sleep(Duration::from_secs(1));
    println!("==============================");    
    println!("Query open orders after cancellation");
    match bit_cli.linear_get_open_orders(&mut serde_json::json!({
        "currency": "USDT",
    })).await {
        Ok(response) => println!("Response: {:?}", response),
        Err(e) => eprintln!("Error: {}", e),
    }

}