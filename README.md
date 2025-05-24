# Bit.com Rust client reference demo

### API request/response format
https://www.bit.com/docs/en-us/spot.html#order

### API Host:
https://www.bit.com/docs/en-us/spot.html#spot-api-hosts-production



# How to run

## Setup env variables: 


* `BITCOM_REST_HOST`: Rest api host
* `BITCOM_WS_HOST`: Websocket host
* `BITCOM_AK`: Access-key
* `BITCOM_SK`: Private-key



## Run rest api demo

```bash
make rest_demo
```

## Run public websocket demo

```bash
make publicws_demo
```

## Run private websocket demo

```bash
make privatews_demo
```


# Incorporate into your project 

Add this to Cargo.toml

```toml
[dependencies]
bitsdk_rust = { git = "https://github.com/ericyeungcode/bitsdk_rust" }
```