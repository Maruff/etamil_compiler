# REST API Examples

Simple REST API examples using eTamil compiler.

## Files

- `simple_api.qmz` - Basic REST API server with GET, POST, PUT, DELETE routes

## Running Examples

Compile and run an example:

```bash
cat examples/api/simple_api.qmz | ./run.sh
```

## API Endpoints

The simple_api.qmz example defines:

- **GET /api/hello** - Returns JSON greeting
- **POST /api/users** - Creates new user (simulated)
- **PUT /api/users/1** - Updates user (simulated)
- **DELETE /api/users/1** - Deletes user (simulated)

## Server Configuration

- Host: localhost
- Port: 8080

## Features

✅ HTTP method definitions (GET, POST, PUT, DELETE)
✅ Route path handling
✅ JSON response generation
✅ HTTP status codes
✅ Server lifecycle management

See main documentation for comprehensive API development guide.
