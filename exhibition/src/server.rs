use axum::{
    extract::{ws::WebSocketUpgrade, State, Multipart},
    response::{Html, Response, IntoResponse},
    routing::{get, post},
    Router, Json,
};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use anyhow::Result;
use tracing::{info, error, warn};
use serde::{Deserialize, Serialize};

use crate::bridge::MavlinkBridge;
use crate::simulation::SimulationState;
use crate::telemetry;

pub struct ExhibitionServer {
    port: u16,
    mavlink_bridge: Arc<MavlinkBridge>,
    simulation_state: Arc<SimulationState>,
    telemetry_tx: broadcast::Sender<telemetry::TelemetryMessage>,
    drawing_tx: broadcast::Sender<DrawingMessage>,
}

impl ExhibitionServer {
    pub async fn new(port: u16) -> Result<Self> {
        let (telemetry_tx, _) = broadcast::channel(1000);
        let (drawing_tx, _) = broadcast::channel(1000);
        
        let mavlink_bridge = Arc::new(MavlinkBridge::new().await?);
        let simulation_state = Arc::new(SimulationState::new());

        Ok(Self {
            port,
            mavlink_bridge,
            simulation_state,
            telemetry_tx,
            drawing_tx,
        })
    }

    pub async fn run(self) -> Result<()> {
        let port = self.port;
        let app = self.create_router().await?;

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        info!("Exhibition server listening on http://0.0.0.0:{}", port);
        info!("Open http://localhost:{} in your browser", port);

        axum::serve(listener, app).await?;
        Ok(())
    }

    async fn create_router(self) -> Result<Router> {
        let state = AppState {
            mavlink_bridge: self.mavlink_bridge.clone(),
            simulation_state: self.simulation_state.clone(),
            telemetry_tx: self.telemetry_tx.clone(),
            drawing_tx: self.drawing_tx.clone(),
        };

        Ok(Router::new()
            .route("/", get(index_handler))
            .route("/ipad", get(ipad_handler))
            .route("/api/upload", post(upload_handler))
            .route("/api/start", post(start_simulation))
            .route("/api/stop", post(stop_simulation))
            .route("/api/status", get(get_status))
            .route("/ws", get(websocket_handler))
            .layer(CorsLayer::permissive())
            .nest_service("/static", ServeDir::new("static"))
            .with_state(state))
    }
}

#[derive(Clone)]
struct AppState {
    mavlink_bridge: Arc<MavlinkBridge>,
    simulation_state: Arc<SimulationState>,
    telemetry_tx: broadcast::Sender<telemetry::TelemetryMessage>,
    drawing_tx: broadcast::Sender<DrawingMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DrawingMessage {
    DrawingStroke {
        from: Point,
        to: Point,
        color: String,
        size: f32,
    },
    DrawingEnd,
    DrawingClear,
    ImageSubmit {
        image: String,
    },
    DrawingAck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Point {
    x: f32,
    y: f32,
}

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn ipad_handler() -> Html<&'static str> {
    Html(include_str!("../static/ipad.html"))
}

async fn upload_handler(
    State(_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    use axum::http::StatusCode;
    
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Failed to read multipart field: {}", e);
        axum::response::Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("content-type", "application/json")
            .body(axum::body::Body::from(serde_json::json!({
                "success": false,
                "error": format!("Failed to read multipart field: {}", e)
            }).to_string()))
            .unwrap()
            .into_response()
    })? {
        let name = field.name().unwrap_or("");
        if name == "image" {
            let data = field.bytes().await.map_err(|e| {
                error!("Failed to read image data: {}", e);
                axum::response::Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to read image data: {}", e)
                    }).to_string()))
                    .unwrap()
                    .into_response()
            })?;
            
            // Process image and generate path
            match process_image(data.as_ref()).await {
                Ok(path_data) => {
                    info!("Image processed successfully, generated {} waypoints", path_data.waypoints.len());
                    return Ok(Json(serde_json::json!({
                        "success": true,
                        "waypoints": path_data.waypoints.len(),
                        "path": path_data
                    })));
                }
                Err(e) => {
                    error!("Failed to process image: {}", e);
                    return Err(axum::response::Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .header("content-type", "application/json")
                        .body(axum::body::Body::from(serde_json::json!({
                            "success": false,
                            "error": format!("Failed to process image: {}", e)
                        }).to_string()))
                        .unwrap()
                        .into_response());
                }
            }
        }
    }

    Err(axum::response::Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::json!({
            "success": false,
            "error": "No image provided"
        }).to_string()))
        .unwrap()
        .into_response())
}

async fn start_simulation(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    info!("Starting simulation");
    
    // Extract path data from payload
    if let Some(path_data) = payload.get("path") {
        // Send waypoints to MAVLink bridge
        if let Err(e) = state.mavlink_bridge.send_mission(path_data).await {
            error!("Failed to send mission: {}", e);
            return Json(serde_json::json!({"success": false, "error": e.to_string()}));
        }
        
        state.simulation_state.set_running(true).await;
        
        Json(serde_json::json!({"success": true}))
    } else {
        Json(serde_json::json!({"success": false, "error": "No path data provided"}))
    }
}

async fn stop_simulation(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    info!("Stopping simulation");
    state.simulation_state.set_running(false).await;
    Json(serde_json::json!({"success": true}))
}

async fn get_status(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let running = state.simulation_state.is_running().await;
    Json(serde_json::json!({
        "running": running,
        "connected": state.mavlink_bridge.is_connected().await
    }))
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: axum::extract::ws::WebSocket, state: AppState) {
    use axum::extract::ws::Message;
    use futures_util::{SinkExt, StreamExt};
    
    let (mut sender, mut receiver) = socket.split();
    let mut telemetry_rx = state.telemetry_tx.subscribe();
    let mut drawing_rx = state.drawing_tx.subscribe();
    let drawing_tx_clone = state.drawing_tx.clone();
    
    // Channel for sending acks back to this client
    let (ack_tx, mut ack_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    // Spawn task to send telemetry and drawing updates
    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                Ok(msg) = telemetry_rx.recv() => {
                    let json = serde_json::json!({
                        "type": "telemetry",
                        "data": msg
                    });
                    if sender.send(Message::Text(serde_json::to_string(&json).unwrap())).await.is_err() {
                        break;
                    }
                }
                Ok(msg) = drawing_rx.recv() => {
                    // Send all drawing messages including ImageSubmit
                    let json = serde_json::to_string(&msg).unwrap();
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
                Some(ack_msg) = ack_rx.recv() => {
                    if sender.send(Message::Text(ack_msg)).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Spawn task to receive messages
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                match serde_json::from_str::<DrawingMessage>(&text) {
                    Ok(drawing_msg) => {
                        match &drawing_msg {
                            DrawingMessage::ImageSubmit { image } => {
                                // Process image and broadcast to all clients
                                match base64_to_bytes(image) {
                                    Ok(image_bytes) => {
                                        match process_image(&image_bytes).await {
                                            Ok(path_data) => {
                                                info!("Image processed from drawing, generated {} waypoints", path_data.waypoints.len());
                                                // Broadcast the image to all clients (including sender)
                                                let _ = drawing_tx_clone.send(DrawingMessage::ImageSubmit {
                                                    image: image.to_string(),
                                                });
                                                // Send acknowledgment back to sender via channel
                                                let ack = serde_json::json!({
                                                    "type": "ack",
                                                    "success": true,
                                                    "waypoints": path_data.waypoints.len(),
                                                    "path": path_data
                                                });
                                                let _ = ack_tx.send(serde_json::to_string(&ack).unwrap());
                                            }
                                            Err(e) => {
                                                error!("Failed to process drawing image: {}", e);
                                                // Send error back to client
                                                let error_ack = serde_json::json!({
                                                    "type": "ack",
                                                    "success": false,
                                                    "error": format!("Failed to process image: {}", e)
                                                });
                                                let _ = ack_tx.send(serde_json::to_string(&error_ack).unwrap());
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to decode base64 image: {}", e);
                                        // Send error back to client
                                        let error_ack = serde_json::json!({
                                            "type": "ack",
                                            "success": false,
                                            "error": format!("Failed to decode image: {}", e)
                                        });
                                        let _ = ack_tx.send(serde_json::to_string(&error_ack).unwrap());
                                    }
                                }
                            }
                            _ => {
                                // Broadcast drawing strokes/clear/end to all clients
                                let _ = drawing_tx_clone.send(drawing_msg.clone());
                            }
                        }
                    }
                    Err(_) => {
                        // Try to parse as telemetry message or other format
                        info!("Received WebSocket message (not drawing): {}", text);
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}

fn base64_to_bytes(data_url: &str) -> Result<Vec<u8>> {
    // Remove data URL prefix (e.g., "data:image/png;base64,")
    let base64_data = if data_url.contains(",") {
        data_url.split(",").nth(1).unwrap_or(data_url)
    } else {
        data_url
    };
    
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(base64_data)
        .map_err(|e| anyhow::anyhow!("Failed to decode base64: {}", e))
}

async fn process_image(data: &[u8]) -> Result<PathData> {
    use plot_planner::{ImageWorldPlacement};
    use plot_planner::optimization::{OptimizationSettings, SpecificEnergyCost};
    use plot_planner::generation::ScreeningGrid;
    
    // Decode image - try multiple formats
    let decoded = {
        // Try PNG first
        let reader = image::ImageReader::new(std::io::Cursor::new(data))
            .with_guessed_format()
            .map_err(|e| anyhow::anyhow!("Failed to create image reader: {}", e))?;
        
        reader.decode().or_else(|_| {
            // Try as PNG explicitly
            image::ImageReader::with_format(
                std::io::Cursor::new(data),
                image::ImageFormat::Png
            ).decode()
        }).or_else(|_| {
            // Try as JPEG
            image::ImageReader::with_format(
                std::io::Cursor::new(data),
                image::ImageFormat::Jpeg
            ).decode()
        }).or_else(|_| {
            // Try as WebP
            image::ImageReader::with_format(
                std::io::Cursor::new(data),
                image::ImageFormat::WebP
            ).decode()
        }).map_err(|e| {
            anyhow::anyhow!("Failed to decode image. Supported formats: PNG, JPEG, WebP. Make sure the image data is valid. Error: {}", e)
        })?
    };
    
    // Convert to grayscale (Luma8 for proper sampling)
    let grayscale = decoded.to_luma8();
    
    // Create image placement - matching sim/src/main.rs parameters
    const PPU: f32 = 0.3; // Pixels per unit
    let image_in_world = ImageWorldPlacement::from_image(
        &grayscale,
        nalgebra::Point2::default(),
        PPU,
    );
    
    // Create screening grid - matching sim/src/main.rs parameters
    let grid = ScreeningGrid {
        resolution: 16.0,
        point_size: 24.0,
        origin: nalgebra::Point2::new(0.0, 0.0),
        orientation: 0.0, // Horizontal grid
        strict: true,
    };
    
    // Use proper screening approach based on UAS structure
    use rand_xoshiro::rand_core::SeedableRng;
    use rand_xoshiro::Xoroshiro64Star;
    use rand::Rng;
    
    let mut rng = Xoroshiro64Star::seed_from_u64(0);
    
    // Prepare screening bounds (similar to prepare_screen logic)
    let bounds = prepare_screening_bounds(&image_in_world, &grid);
    
    // Generate points using frequency-modulated screening
    let mut points = Vec::new();
    let (width, height) = grayscale.dimensions();
    
    // Debug: Log bounds information
    info!("Screening bounds: i_range={:?}, j_range={:?}, im_x_range={:?}, im_y_range={:?}", 
          bounds.i_range, bounds.j_range, bounds.im_x_range, bounds.im_y_range);
    info!("Image dimensions: {}x{}, PPU={}, image position={:?}, image size={:?}", 
          width, height, PPU, image_in_world.position, image_in_world.size());
    info!("Grid: resolution={}, origin={:?}, orientation={}", 
          grid.resolution, grid.origin, grid.orientation);
    
    // Check if ranges are valid
    if bounds.i_range[0] > bounds.i_range[1] || bounds.j_range[0] > bounds.j_range[1] {
        warn!("Invalid grid ranges: i_range={:?}, j_range={:?}. Image may be too small or grid resolution too large.", 
              bounds.i_range, bounds.j_range);
        return Err(anyhow::anyhow!(
            "Invalid grid ranges. Image size: {}x{}, Grid resolution: {}. Try using a larger image or smaller grid resolution.",
            width, height, grid.resolution
        ));
    }
    
    let mut total_grid_points = 0;
    let mut inside_bounds = 0;
    let mut valid_pixels = 0;
    let mut passed_threshold = 0;
    
    // Iterate through grid points
    for i in bounds.i_range[0]..=bounds.i_range[1] {
        for j in bounds.j_range[0]..=bounds.j_range[1] {
            total_grid_points += 1;
            
            // Compute local grid coordinates
            let local_x = (i as f32) * grid.resolution;
            let local_y = (j as f32) * grid.resolution;
            let p_local = nalgebra::Point2::new(local_x, local_y);
            
            // Transform to world space
            let p_world = bounds.grid_to_world.transform_point(&p_local);
            
            // World-space containment check
            let is_inside_x = p_world.x >= bounds.im_x_range[0] && p_world.x <= bounds.im_x_range[1];
            let is_inside_y = p_world.y >= bounds.im_y_range[0] && p_world.y <= bounds.im_y_range[1];
            if !is_inside_x || !is_inside_y {
                continue;
            }
            inside_bounds += 1;
            
            // Convert world coordinates to image pixel coordinates (normalized 0.0 to 1.0)
            let pixel_coords_f = (p_world - image_in_world.position).zip_map(&image_in_world.size(), |a, b| a / b);
            
            // Clamp normalized coordinates to [0, 1] to handle edge cases
            let norm_x = pixel_coords_f.x.max(0.0).min(1.0);
            let norm_y = pixel_coords_f.y.max(0.0).min(1.0);
            
            // Use sample_nearest like the original screen_fm function does
            // sample_nearest expects normalized coordinates (0.0 to 1.0)
            // It returns Option, so we need to handle that
            let pixel = match image::imageops::sample_nearest(&grayscale, norm_x, norm_y) {
                Some(p) => p,
                None => continue, // Skip if out of bounds (shouldn't happen with clamping, but be safe)
            };
            valid_pixels += 1;
            
            // Get the sample value (Luma8 has one channel)
            let sample_value = pixel[0] as f32;
            
            // Frequency-modulated screening: compare to random threshold
            // For spraying, we want to spray DARK areas, so we invert the comparison
            // (original logic: sample > threshold generates points for bright areas)
            // Inverted: threshold > sample generates points for dark areas
            let threshold = rng.random_range(0.0..=255.0);
            
            // Invert logic for spraying dark areas (where there's something to spray)
            // Also ensure very dark pixels (< 50) always generate points
            if sample_value < 50.0 || threshold > sample_value {
                passed_threshold += 1;
                points.push(plot_planner::path::Point::new(
                    p_world,
                    grid.point_size,
                ));
            }
        }
    }
    
    info!("Generated {} points from screening (image size: {}x{}, grid resolution: {})", 
          points.len(), width, height, grid.resolution);
    info!("Debug stats: total_grid_points={}, inside_bounds={}, valid_pixels={}, passed_threshold={}", 
          total_grid_points, inside_bounds, valid_pixels, passed_threshold);
    
    if points.is_empty() {
        return Err(anyhow::anyhow!(
            "No waypoints generated. This can happen if:\n\
            - The image is too bright (try a darker image)\n\
            - The grid resolution ({}) is too coarse\n\
            - The image is too small\n\
            Try using a darker image or adjusting screening parameters.", 
            grid.resolution
        ));
    }
    
    // Optimize path using proper optimization settings
    let characteristics = SpecificEnergyCost::from_penalties(1.5, 0.1, 1.0);
    
    let optimizer = OptimizationSettings {
        specific_energy: characteristics,
        penalty: plot_planner::optimization::DirectionChangePenalty(200.0),
        start: nalgebra::Point2::new(0., 0.0),
        include_start: true,
    };
    
    // Use nearest-neighbor optimization (optimize_points is commented out in the library)
    let order = optimized_nearest_neighbor(&points, &optimizer);
    
    // Convert to waypoints
    let waypoints: Vec<Waypoint> = order
        .iter()
        .map(|&idx| {
            let point = &points[idx];
            Waypoint {
                x: point.position.x,
                y: point.position.y,
                z: 1.0, // Default altitude
                size: point.size,
            }
        })
        .collect();
    
    Ok(PathData { waypoints })
}

// Helper struct for screening bounds
struct ScreeningBounds {
    grid_to_world: nalgebra::Isometry2<f32>,
    i_range: [i64; 2],
    j_range: [i64; 2],
    im_x_range: [f32; 2],
    im_y_range: [f32; 2],
}

fn prepare_screening_bounds(
    im: &plot_planner::ImageWorldPlacement,
    grid: &plot_planner::generation::ScreeningGrid,
) -> ScreeningBounds {
    // Calculate corner points of the image rectangle in world space
    let width = (im.im_width as f32) / im.ppu;
    let height = (im.im_height as f32) / im.ppu;
    
    let top_left = im.position;
    let top_right = nalgebra::Translation2::new(width, 0.0).transform_point(&top_left);
    let bottom_left = nalgebra::Translation2::new(0.0, height).transform_point(&top_left);
    let bottom_right = nalgebra::Translation2::new(width, height).transform_point(&top_left);
    
    let corners_world = [top_left, top_right, bottom_right, bottom_left];
    
    // Transform to grid space
    let grid_to_world = nalgebra::Isometry2::new(grid.origin.coords, grid.orientation);
    let world_to_grid = grid_to_world.inverse();
    let corners_grid = corners_world.map(|point| world_to_grid.transform_point(&point));
    
    // Calculate AABB in grid space
    let mut x_min = f32::MAX;
    let mut x_max = f32::MIN;
    let mut y_min = f32::MAX;
    let mut y_max = f32::MIN;
    for c in corners_grid {
        x_min = x_min.min(c.x);
        x_max = x_max.max(c.x);
        y_min = y_min.min(c.y);
        y_max = y_max.max(c.y);
    }
    
    let (l, r): (fn(f32) -> f32, fn(f32) -> f32) = if grid.strict {
        (f32::ceil, f32::floor)
    } else {
        (f32::floor, f32::ceil)
    };
    
    let i_range = [
        l(x_min / grid.resolution) as i64,
        r(x_max / grid.resolution) as i64,
    ];
    let j_range = [
        l(y_min / grid.resolution) as i64,
        r(y_max / grid.resolution) as i64,
    ];
    
    // Calculate AABB in world space
    let mut x_min_w = f32::MAX;
    let mut x_max_w = f32::MIN;
    let mut y_min_w = f32::MAX;
    let mut y_max_w = f32::MIN;
    for c in corners_world {
        x_min_w = x_min_w.min(c.x);
        x_max_w = x_max_w.max(c.x);
        y_min_w = y_min_w.min(c.y);
        y_max_w = y_max_w.max(c.y);
    }
    
    ScreeningBounds {
        grid_to_world,
        i_range,
        j_range,
        im_x_range: [x_min_w, x_max_w],
        im_y_range: [y_min_w, y_max_w],
    }
}

// Improved nearest-neighbor optimization with energy cost consideration
fn optimized_nearest_neighbor(
    points: &[plot_planner::path::Point],
    settings: &plot_planner::optimization::OptimizationSettings,
) -> Vec<usize> {
    if points.is_empty() {
        return vec![];
    }

    let mut visited = vec![false; points.len()];
    let mut order = Vec::with_capacity(points.len());
    
    // Start at the point closest to the start position
    let mut current = 0;
    let mut min_dist = f32::MAX;
    for (i, point) in points.iter().enumerate() {
        let diff = point.position - settings.start;
        // Apply energy cost weights: up=1.5, down=0.1, forward=1.0
        let cost = (diff.x.abs() * 1.0) + (diff.y.abs() * if diff.y < 0.0 { 1.5 } else { 0.1 });
        if cost < min_dist {
            min_dist = cost;
            current = i;
        }
    }

    order.push(current);
    visited[current] = true;

    // Greedy nearest neighbor with energy-aware distance
    while order.len() < points.len() {
        let mut best = None;
        let mut best_cost = f32::MAX;

        for (i, point) in points.iter().enumerate() {
            if visited[i] {
                continue;
            }

            let diff = point.position - points[current].position;
            // Calculate energy-aware cost: up=1.5, down=0.1, forward=1.0
            let cost = (diff.x.abs() * 1.0) + (diff.y.abs() * if diff.y < 0.0 { 1.5 } else { 0.1 });

            if cost < best_cost {
                best_cost = cost;
                best = Some(i);
            }
        }

        if let Some(next) = best {
            order.push(next);
            visited[next] = true;
            current = next;
        } else {
            break;
        }
    }

    order
}


#[derive(serde::Serialize, serde::Deserialize)]
struct PathData {
    waypoints: Vec<Waypoint>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Waypoint {
    x: f32,
    y: f32,
    z: f32,
    size: f32,
}
