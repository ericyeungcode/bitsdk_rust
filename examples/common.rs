use bitsdk_rust::bit_rest_client;
use std::env;

pub fn build_rest_client() -> bit_rest_client::BitRestClient {
    let api_host = env::var("BITCOM_REST_HOST").unwrap();
    let ws_host = env::var("BITCOM_WS_HOST").unwrap();
    let access_key = env::var("BITCOM_AK").unwrap();
    let private_key = env::var("BITCOM_SK").unwrap();

    // Use api_host, access_key, private_key...
    println!("### API host:{}, WS host:{}, access-key:{}", api_host, ws_host, access_key);
    bit_rest_client::BitRestClient::new( &access_key, &private_key, &api_host)
}