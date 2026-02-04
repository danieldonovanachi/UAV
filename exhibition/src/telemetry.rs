use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct TelemetryMessage {
    pub timestamp: u64,
    pub message_type: TelemetryType,
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum TelemetryType {
    Position {
        x: f64,
        y: f64,
        z: f64,
        heading: f64,
    },
    Velocity {
        vx: f64,
        vy: f64,
        vz: f64,
    },
    Status {
        armed: bool,
        mode: String,
        battery: f32,
    },
    WaypointReached {
        waypoint: usize,
    },
    SprayTriggered {
        waypoint: usize,
    },
}

impl TelemetryMessage {
    pub fn position(x: f64, y: f64, z: f64, heading: f64) -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            message_type: TelemetryType::Position { x, y, z, heading },
        }
    }

    pub fn status(armed: bool, mode: String, battery: f32) -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            message_type: TelemetryType::Status { armed, mode, battery },
        }
    }

    pub fn waypoint_reached(waypoint: usize) -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            message_type: TelemetryType::WaypointReached { waypoint },
        }
    }

    pub fn spray_triggered(waypoint: usize) -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            message_type: TelemetryType::SprayTriggered { waypoint },
        }
    }
}
