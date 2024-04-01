# API

You can run the API with the following command:

```bash
cargo run
``` 

## Endpoints

### POST endpoints: 


#### `POST` /configuration
To create a new configuration, you can send a POST request to the `/configuration` endpoint with the following body:

<!-- TODO -->

#### `POST` /run
To run a vm with a configuration, you can send a POST request to the `/run` endpoint with the following body:

```json
{
    "id": "vm-id"
}
```

#### `POST` /shutdown

To shutdown a vm, you can send a POST request to the `/shutdown` endpoint with the following body:

```json
{
    "id": "vm-id"
}
```

### GET ENDPOINTS:

#### `GET` /logs/{id}

To get the logs of a vm, you can send a GET request to the `/logs/{id}` endpoint.

#### `GET` /metrics/{id}

To get the metrics of a vm, you can send a GET request to the `/metrics/{id}` endpoint.

