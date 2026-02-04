# UAV Spraying Exhibition Interface

A web-based interface and simulation infrastructure for demonstrating the UAV spraying system at exhibitions.

## Features

- **iPad Drawing Interface**: Touch-optimized drawing canvas for visitors to create artwork
- **Image Upload**: Upload images to generate spray paths (on both iPad and main interface)
- **Real-time Drawing Sync**: Drawings appear on the main display as they're being created
- **Real-time Visualization**: See the path generation algorithm in action
- **Painting Progress**: Visual feedback showing which parts of the path have been painted by the drone
- **PX4 SITL Integration**: Connect to PX4 Software-In-The-Loop simulation
- **Live Telemetry**: Monitor drone position, velocity, and status
- **Interactive Control**: Start/stop simulations and monitor progress

## Setup

### Prerequisites

1. **Rust** (latest stable version)
2. **PX4 Autopilot** (for SITL simulation)
   - Clone: `git clone https://github.com/aabizri/PX4-Autopilot-SurfaceReferential`
   - Follow PX4 setup instructions

### Installation

1. Build the exhibition server:
```bash
cd uas/exhibition
cargo build --release
```

2. Start the server:
```bash
cargo run --release
# Or specify a port:
cargo run --release -- 8080
```

3. Open your browser to `http://localhost:8080`
   - Main exhibition interface: `http://localhost:8080`
   - iPad drawing interface: `http://localhost:8080/ipad`

### PX4 SITL Setup

1. Start PX4 SITL:
```bash
cd /path/to/PX4-Autopilot-SurfaceReferential
make px4_sitl jmavsim
# Or with Gazebo:
make px4_sitl gazebo
```

2. The exhibition interface will connect to PX4 via MAVLink on UDP port 14540 (default).

## Architecture

- **Server** (`src/server.rs`): Web server with REST API and WebSocket support
- **Bridge** (`src/bridge.rs`): MAVLink bridge to PX4 SITL
- **Simulation** (`src/simulation.rs`): Simulation state management
- **Telemetry** (`src/telemetry.rs`): Telemetry message handling

## API Endpoints

- `GET /` - Main exhibition interface
- `GET /ipad` - iPad-optimized drawing interface
- `POST /api/upload` - Upload image for path generation
- `POST /api/start` - Start simulation
- `POST /api/stop` - Stop simulation
- `GET /api/status` - Get current status
- `WS /ws` - WebSocket for real-time telemetry and drawing synchronization

## Usage

### Main Exhibition Interface

1. **Upload Image**: Drag and drop or click to select an image
2. **Generate Path**: Click "Generate Path" to process the image
3. **Start Simulation**: Click "Start Simulation" to send waypoints to PX4
4. **Monitor**: Watch telemetry updates in real-time and see painting progress

### iPad Drawing Interface

1. **Open iPad Interface**: Navigate to `http://<server-ip>:8080/ipad` on the iPad
2. **Draw or Upload**:
   - **Draw Mode**: Use finger or stylus to draw on the canvas
   - **Upload Mode**: Select a photo from the iPad's photo library
3. **Customize**: Adjust brush size and color (in Draw mode)
4. **Send to Drone**: Tap "Send to Drone" to process and send the drawing/image
5. **Real-time Sync**: The drawing appears on the main display as you create it

### Exhibition Setup

For the best experience:
- **Main Display**: Connect a large monitor/projector showing `http://<server-ip>:8080`
- **iPad**: Open `http://<server-ip>:8080/ipad` on the iPad (ensure both devices are on the same network)
- **Network**: Use a local network or WiFi hotspot for low latency
- **Fullscreen**: On iPad, add to home screen for a fullscreen app-like experience

## Development

The interface uses:
- **Backend**: Rust with Axum web framework
- **Frontend**: Vanilla JavaScript with HTML5 Canvas
- **Communication**: WebSocket for real-time updates
- **MAVLink**: For PX4 communication

## Exhibition Mode

For exhibitions, you can:
- **Dual Display Setup**: 
  - Main display showing the visualization and telemetry
  - iPad for visitor interaction
- **Network Configuration**: 
  - Use a local WiFi network or router
  - Ensure both devices can reach the server IP
  - For best performance, use a wired connection for the server
- **iPad Optimization**:
  - Add the iPad interface to home screen for fullscreen experience
  - Disable auto-lock to keep the interface active
  - Use a stylus for more precise drawing
- **Visual Feedback**:
  - Drawings appear in real-time on the main display
  - Painting progress is shown as the drone follows the path
  - Waypoints light up as they're painted
- **Additional Enhancements**:
  - Connect to physical hardware via serial port
  - Add sound effects for spray triggers
  - Display on large screens/projectors

## Troubleshooting

- **PX4 not connecting**: Check that PX4 SITL is running and listening on UDP port 14540
- **Path generation fails**: Ensure image is in a supported format (PNG, JPG, etc.)
- **WebSocket disconnects**: Check network connectivity and server logs
- **iPad can't connect**: 
  - Ensure iPad and server are on the same network
  - Check firewall settings on the server
  - Verify the server IP address is correct
  - Try accessing from a browser first to test connectivity
- **Drawing not appearing on main display**:
  - Check WebSocket connection status indicators
  - Ensure both interfaces are connected to the same server
  - Check browser console for errors
- **High latency in drawing sync**:
  - Use a local network (avoid internet routing)
  - Ensure WiFi signal strength is good
  - Check server CPU/memory usage
