use std::{net::SocketAddr, env::Args, convert::Infallible};
use hyper::{Body, Request, Response, service::{service_fn, make_service_fn}, Server, Client, client::HttpConnector, Uri, Method, StatusCode, body::Bytes};
use hyper_tls::HttpsConnector;

const REDIRECT_LIMIT: u8 = 5;

#[tokio::main]
async fn main() {
    let mut args: Args = std::env::args();
    let mut port: u16 = 3000;
    while let Some(arg) = args.next() {
        if arg == "-p" || arg == "--port" {
            if let Some(port2) = args.next() {
                if let Ok(port2) = port2.parse::<u16>() {
                    port = port2;
                }
            }
        }
    }
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let make_service = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(proxy))
    });
    let server = Server::bind(&address).serve(make_service);
    if let Err(err) = server.await {
        eprintln!("Proxy error: {}", err);
    }
}

async fn proxy(mut req: Request<Body>) -> Result<Response<Body>, Infallible> {
    if req.method() == Method::OPTIONS {
        return Ok(Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "*")
            .header("Access-Control-Allow-Headers", "*")
            .header("Access-Control-Allow-Credentials", "true")
            .body(Body::empty())
            .unwrap()
        );
    }

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let uri = req.uri().clone();
    let mut path = String::from(&uri.path()[1..]);
    if !path.starts_with("https://") && !path.starts_with("http://") {
        path = format!("https://{}", path);
    }
    let mut headers: Vec<(String, String)> = Vec::new();
    let mut filtered_query: Vec<String> = Vec::new();
    if let Some(query) = req.uri().query() {
        for item in query.split("&") {
            if let Some((key, value)) = item.split_once("=") {
                if key == "soshiki_set_header" {
                    let value = decode(value.to_string());
                    if let Some((hkey, hvalue)) = value.split_once(":") {
                        headers.push((hkey.to_string(), hvalue.to_string()));
                    }
                } else {
                    filtered_query.push(format!("{}={}", key, value));
                }
            } else {
                filtered_query.push(item.to_string());
            }
        }
    }
    for header in req.headers() {
        if header.0.clone().as_str() != "host" {
            headers.push((header.0.to_string(), header.1.to_str().unwrap_or("").to_string()));
        }
    }

    let mut filtered_query = filtered_query.join("&");
    if !filtered_query.is_empty() {
        filtered_query = format!("?{}", filtered_query);
    }

    path.truncate(path.find("&").unwrap_or(path.len()));
    let new_uri = format!("{}{}", path, filtered_query).parse().unwrap_or(Uri::default());

    let body = if let Ok(body) = hyper::body::to_bytes(req.body_mut()).await { Some(body) } else { None };
    if let Some(mut response) = request(&client, new_uri, req.method().clone(), headers, body, 0).await {
        let mut cors_response = Response::builder().status(response.status());
        for header in response.headers() {
            cors_response = cors_response.header(header.0.as_str(), header.1.to_str().unwrap_or(""));
        }
        cors_response = cors_response
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "*")
            .header("Access-Control-Allow-Headers", "*")
            .header("Access-Control-Allow-Credentials", "true");
        let body = if let Ok(body) = hyper::body::to_bytes(response.body_mut()).await { Body::from(body) } else { Body::empty() };
        if let Ok(response) = cors_response.body(body) {
            return Ok(response);
        }
    }
    return Ok(Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::empty()).unwrap());
}

#[async_recursion::async_recursion]
async fn request(client: &Client<HttpsConnector<HttpConnector>>, uri: Uri, method: Method, headers: Vec<(String, String)>, bytes: Option<Bytes>, redirects: u8) -> Option<Response<Body>> {
    if redirects >= REDIRECT_LIMIT {
        return None;
    }
    let mut builder = Request::builder()
        .method(method.clone())
        .uri(uri.clone());
    for header in headers.clone() {
        builder = builder.header(header.0, header.1);
    }
    let body = if let Some(bytes) = bytes.clone() { Body::from(bytes) } else { Body::empty() };
    if let Ok(req) = builder.body(body) {
        if let Ok(response) = client.request(req).await {
            if response.status().is_redirection() {
                if let Some(redirect_path) = response.headers().get("Location") {
                    let redirect_path = redirect_path.to_str().unwrap_or("");
                    let redirect_uri: String;
                    if redirect_path.starts_with("/") {
                        let host = uri.host().unwrap_or("");
                        redirect_uri = format!("{}{}", host, redirect_path);
                    } else {
                        redirect_uri = String::from(redirect_path);
                    }
                    return request(client, redirect_uri.parse().unwrap(), method, headers, bytes, redirects + 1).await;
                }
            } else {
                return Some(response);
            }
        }
    }
    return None;
}

// from "urldecode" crate
pub fn decode(url: String) -> String {
    let mut decoded = String::from("");
    let mut skip = 0;
    for i in 0..url.len() {
        if skip != 0 {
            skip -= 1;
            continue;
        }
        let c: char = url.chars().nth(i).unwrap();
        if c == '%' {
            let left = url.chars().nth(i + 1).unwrap();
            let right = url.chars().nth(i + 2).unwrap();
            let byte = u8::from_str_radix(&format!("{}{}", left, right), 16).unwrap();
            decoded += &(byte as char).to_string();
            skip = 2;
        } else {
            decoded += &c.to_string();
        }
    }
    decoded
}