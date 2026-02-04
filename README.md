# Autonomous Spraying

1. Environment Setup

An "infinite canvas" where the assumed or scanned environment is placed.
Markers can be added (marking the UV (0,0) - (1,1) ),as well as layers:
- red/green zones (either blacklist-based or whitelist-based).
- depth
- "channel width" for the drone to fly in

This can be serialized/deserialized (which format) ?

2. Material Setup

Spray kind (cap kind) and colour

3. Image Placement/Generation

On that canvas, you can load image(s).
Q: Multimaterial support ? (Drone color change) ?
Q: Mapping from colour/image to multimaterial?

4. Spraypath Generation

Once ready, the spraypath is generated

5. Then it can be
- Exported to waypoints ?
- Sent via MAVLink live ?
