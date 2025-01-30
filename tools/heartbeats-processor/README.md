# Heartbeats Processor

This application processes "heartbeat" entries from Firestore. It fetches data, groups it by time windows, and stores the results into a local SQLite database for further analysis or reporting.

## Environment Variables

The following environment variables control the program's behavior.

These variables can be set in your shell environment or in a `.env` file in the project root directory.

### Required Variables
* `DATABASE_PATH` - SQLite database path (e.g., "./data.db")
* `GOOGLE_CLOUD_PROJECT` - Google Cloud project ID
* `WINDOW_RANGE_START` - Start time for window creation in RFC3339 format
* `WINDOW_RANGE_END` - End time for window creation in RFC3339 format

### Optional Variables
* `GOOGLE_APPLICATION_CREDENTIALS` - Path to Google Cloud credentials file
* `DISABLED_WINDOWS` - Comma-separated list of time ranges to disable in RFC3339 format (e.g., `2023-01-01T00:00:00Z/2023-01-02T00:00:00Z,2023-02-01T00:00:00Z/2023-02-02T00:00:00Z`)

## Development With Firestore Emulator

To develop locally using the Firestore Emulator, do the following:

1. Set these environment variables in your shell:
   
   ```
   FIRESTORE_EMULATOR_HOST=127.0.0.1:8080
   GOOGLE_CLOUD_PROJECT=staging
   ```

2. From the "frontend/firestore" directory, start the emulator by running:
   
   ```
   npm run serve
   ```

3. Authenticate on your local machine with Google Cloud to allow proper credential usage:
   
   ```
   gcloud auth application-default login
   ```

Once these steps are complete, the application can connect to the local emulator to simulate production-like Firestore behavior for debugging or development.
