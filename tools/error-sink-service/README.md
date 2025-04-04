# Error Sink Service

A simple HTTP service for collecting and cataloging error reports from OpenMina nodes.

## Usage

### Running the service

```bash
# Basic usage with default settings
cargo run

# Specify a custom port and storage directory
ERROR_SINK_PORT=9090 ERROR_SINK_DIR=/path/to/reports cargo run

# Disable signature verification
ERROR_SINK_VERIFY_SIGNATURES=false cargo run
```

### Environment variables

| Variable | Description | Default |
|----------|-------------|---------|
| `ERROR_SINK_PORT` | HTTP port to listen on | `8080` |
| `ERROR_SINK_DIR` | Directory to store reports | `./reports` |
| `ERROR_SINK_VERIFY_SIGNATURES` | Enable/disable signature verification | `true` |

### Docker usage

```bash
docker run -p 8080:8080 -v ./reports:/app/reports openmina/error-sink-service
```

## API Endpoints

### `POST /error-report`

Submit a new error report. The endpoint accepts JSON data with the following structure:

```json
{
  "submitter": "B62qrPN5Y5yq8kGE3FbVKbGTdTAJNdtNtB5sNVpxyRwWGcDEhpMzc8g",
  "category": "blockProofFailure",
  "data": "base64-encoded-binary-data",
  "signature": "base64-encoded-signature"
}
```

Field descriptions:
- `submitter`: Valid base58-encoded Mina public key of the submitting entity
- `category`: Classification of the error type (string, must be one of the valid categories)
- `data`: Base64-encoded binary data containing the error report
- `signature`: Base64-encoded cryptographic signature of the data, created using the private key corresponding to the submitter public key

Example submission:

```bash
curl -X POST -H "Content-Type: application/json" -d '{
  "submitter": "B62qrPN5Y5yq8kGE3FbVKbGTdTAJNdtNtB5sNVpxyRwWGcDEhpMzc8g",
  "category": "blockProofFailure",
  "data": "SGVsbG8gV29ybGQ=",
  "signature": "7mXGPhek8iTKjKrYbg7G9U2X5Bk8P5HBDSdwMCJYdPoE5MvJ9Rdho2C4xNe7LcDNPJM9Lrb3r8CpQyUrSS7bDtvm1ZrqZgL"
}' http://localhost:8080/error-report
```

## File Storage Format

Error reports are stored with descriptive filenames using the following format:
```
{category}-{submitter_public_key}_{timestamp}_{uuid}.report
```

For example:
```
blockProofFailure-B62qrPN5Y5yq8kGE3FbVKbGTdTAJNdtNtB5sNVpxyRwWGcDEhpMzc8g_20230405-123015_550e8400-e29b-41d4-a716-446655440000.report
```


