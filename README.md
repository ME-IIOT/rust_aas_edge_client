## Example Structure

``` bash
src/
|-- main.rs           # Entry point, setup of the web server
|-- handlers/         # Request handlers
|   |-- mod.rs
|   `-- user.rs       # Example user-related handlers
|-- models/           # Data models
|   `-- user.rs
|-- routes.rs         # Route definitions
|-- error.rs          # Error types and handling
|-- db.rs             # Database access functions
|-- config.rs         # Configuration management
`-- utils.rs          # Utility functions
```

## Start MongoDB through Docker compose

``` bash
sudo docker compose up -d mongodb
```

## Start backend

### Development

``` bash
cargo run
```

