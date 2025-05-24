# Bit.com Rust client reference demo

### API request/response format
https://www.bit.com/docs/en-us/spot.html#order

### Guidelines for account mode
https://www.bit.com/docs/en-us/spot.html#guidelines-for-account-mode

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