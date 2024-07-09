// © 2023-2024 ETH Zurich
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// any later version.
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::env;

use axum::extract::DefaultBodyLimit;

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
        .route("/solve", axum::routing::post(solve))
        .layer(DefaultBodyLimit::disable());

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
    println!("\n\n-------------------- New Request --------------------\n");
    let output = server::solve_instance(input_data);
    axum::response::Json(output)
}
