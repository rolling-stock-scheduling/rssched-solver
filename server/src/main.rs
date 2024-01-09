#[tokio::main]
pub async fn main() {
    sbb_server::start();

    let app = axum::Router::new()
        .fallback(axum::routing::get(|| async {
            "No Route! Use /health or /solve"
        }))
        .route("/health", axum::routing::get(|| async { "Healthy" }))
        .route("/solve", axum::routing::post(solve));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

pub async fn solve(
    axum::extract::Json(input_data): axum::extract::Json<serde_json::Value>,
) -> axum::response::Json<serde_json::Value> {
    let output = sbb_solver::run(input_data);
    axum::response::Json(output)
}
