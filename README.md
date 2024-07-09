# Ticks
Simple, ergonomic Rust wrapper for the TickTick Open API

![docs.rs](https://img.shields.io/docsrs/ticks)
## Getting Started
First, register your application with the TickTick Developer Center:
> To get started using the TickTick Open API, you will need to register your application and obtain a client ID and client secret. You can register your application by visiting the [TickTick Developer Center](https://developer.ticktick.com/manage). Once registered, you will receive a client ID and client secret which you will use to authenticate your requests.

Once you have registered your application, add `ticks` to your project's `Cargo.toml`:
```
cargo add ticks
```
To use the TickTick API, you must authorize your app at runtime. Let's talk about **Authorization**.

## Authorization
The TickTick API uses [OAuth](https://oauth.net/2/) for authentication.
Ticks handles most of the authentication for you, leaving only reading the API's HTTP OAuth response to the you.
```rust
/// Get Authorization URL, this is the link the user must visit to allow our Application access to their account.
/// redirect_uri must be the same URL specified in the TickTick Developer Center.
let auth = Authorization::begin_auth(/* client_id */, /* redirect_uri */)?;
println!("Browse to: {:?}", auth.get_url());
/// Wait for response from TickTick's API. TickTick will send the required auth info over HTTP to the redirect_uri we specified.
let (access_code, state) = /* Get access_code & state from redirect_uri over HTTP */;
/// Get access token
let token = auth.finish_auth({client_secret}, code, state).await?;
/// Done! Create TickTick instance using AccessToken.
let ticktick = TickTick::new(token.clone())?;
```
For testing, you may want to set your redirect_uri to `localhost`. To read the OAuth HTTP Response locally, try a `TcpListener`
```rust
let listener = TcpListener::bind("127.0.0.1:{port of redirect_uri}")?;
let (mut stream, _) = listener.accept()?;
let mut stream_reader = BufReader::new(&stream);
let mut response = String::new();
stream_reader.read_line(&mut response)?;
stream.write_all("HTTP/1.1 200 OK".as_bytes())?;
println!("Response {:?}", response);
```

## Documentation
The docs can be found at https://docs.rs/ticks/latest.