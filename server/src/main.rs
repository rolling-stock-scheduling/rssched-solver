#[tokio::main]
pub async fn main() {
    let app = axum::Router::new()
        .fallback(axum::routing::get(|| async {
            "No Route! Use /health or /solve"
        }))
        .route("/health", axum::routing::get(healthy))
        .route("/solve", axum::routing::post(solve));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on port 3000 (http://localhost:3000/health)");
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
