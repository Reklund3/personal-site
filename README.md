# Purpose

To follow a guided lesson from Zero to Production in Rust by Luca Palmieri.

After completing the book I have worked to take the project further and continue learning.

### Requirements
Before you can run the project you will need to generate and store a `cert.pem` and `key.pem` inside the `security` directory i.e.`security/cert.pem` relative to the project root.

### Commands
curl command for subscribing
> curl -i -X POST -d "email=rob@test.com&name=Rob" http://localhost:8080/subscriptions

## Docker

### Building the Docker Image
```bash
docker build -t site:latest .
```

To build with a specific version tag:
```bash
docker build -t site:1.0.0 .
```

### Running the Docker Container
To run the docker container you will need to pass the security files from the host via the following run option:

```bash
docker run -v '/hostpath/':'/run/secrets':'ro' site:latest
```

The configured container path unless modified uses `/run/secrets/` so that swarm users can provide docker secrets.

## NOTES
+ Invest time into `rust-musl-builder` to generate tiny containers.

## Improvements
+ Need to expire the idempotency keys
+ Enhance issue_delivery_queue to use retry count and exponential back off.