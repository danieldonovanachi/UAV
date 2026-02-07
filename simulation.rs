use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

pub struct SimulationState {
    running: Arc<Mutex<bool>>,
    current_mission: Arc<Mutex<Option<MissionData>>>,
}

struct MissionData {
    waypoint_count: usize,
    current_waypoint: usize,
}

impl SimulationState {
    pub fn new() -> Self {
        Self {
            running: Arc::new(Mutex::new(false)),
            current_mission: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn set_running(&self, running: bool) {
        *self.running.lock().await = running;
        if running {
            info!("Simulation started");
        } else {
            info!("Simulation stopped");
        }
    }

    pub async fn is_running(&self) -> bool {
        *self.running.lock().await
    }

    pub async fn set_mission(&self, waypoint_count: usize) {
        *self.current_mission.lock().await = Some(MissionData {
            waypoint_count,
            current_waypoint: 0,
        });
    }

    pub async fn get_mission(&self) -> Option<MissionData> {
        self.current_mission.lock().await.clone()
    }
}

impl Clone for MissionData {
    fn clone(&self) -> Self {
        Self {
            waypoint_count: self.waypoint_count,
            current_waypoint: self.current_waypoint,
        }
    }
}
