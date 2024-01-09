use std::env;

#[tokio::main]
pub async fn main() {
    // Parse command line arguments to get the port number
    let args: Vec<String> = env::args().collect();
    let port: u16 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3000);

    let app = axum::Router::new()
        .fallback(axum::routing::get(|| async {
            "No route! Use /health or /solve."
        }))
        .route("/health", axum::routing::get(healthy))
        .route("/solve", axum::routing::post(solve));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    println!(
        "Server running on port {} (http://localhost:{}/health)",
        port, port
    );
    axum::serve(listener, app).await.unwrap();
}

pub async fn healthy() -> &'static str {
    println!("Healthy");
    "Healthy"
}

pub async fn solve(
    axum::extract::Json(input_data): axum::extract::Json<serde_json::Value>,
) -> axum::response::Json<serde_json::Value> {
    println!("Solving");
    let output = server::solve_instance(input_data);
    axum::response::Json(output)
}
