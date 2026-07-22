// C API Header for Cyberspace Engine
// Use this header in Unreal Engine C++ to integrate with the Rust library

#ifndef CYBERSPACE_ENGINE_H
#define CYBERSPACE_ENGINE_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque handle to engine instance
typedef void* CyberspaceEngineHandle;

// Point structure matching Rust layout
typedef struct {
    float position[3];  // x, y, z
    float color[3];     // r, g, b (0.0 - 1.0)
    float brightness;   // brightness multiplier
    float size;         // point size
} CPoint;

// Network parameters structure
typedef struct {
    float density;
    float chaos;
    float flow;
    float entropy;
    float packet_rate;
    float energy;
    float frequency;
    float curvature;
    float packets_per_sec;
    float bytes_per_sec;
    uint32_t tcp_count;
    uint32_t udp_count;
} CNetworkParams;

// Initialize the engine
// Parameters:
//   chunk_size: Size of each chunk in world units (e.g., 100.0)
//   render_distance: Maximum distance to load chunks (e.g., 500.0)
//   seed: Random seed for procedural generation
// Returns: Handle to engine instance, or NULL on failure
CyberspaceEngineHandle cyberspace_engine_create(
    float chunk_size,
    float render_distance,
    uint32_t seed
);

// Destroy the engine and free resources
// Parameters:
//   handle: Engine handle returned from cyberspace_engine_create
void cyberspace_engine_destroy(CyberspaceEngineHandle handle);

// Update camera position (triggers chunk loading/unloading)
// Parameters:
//   handle: Engine handle
//   x, y, z: Camera position in world space
void cyberspace_engine_update_camera(
    CyberspaceEngineHandle handle,
    float x,
    float y,
    float z
);

// Get visible points for rendering
// Parameters:
//   handle: Engine handle
//   out_points: Buffer to write points to (can be NULL to query size)
//   max_points: Maximum number of points to write
// Returns: Number of points written, or required buffer size if out_points is NULL
uint32_t cyberspace_engine_get_points(
    CyberspaceEngineHandle handle,
    CPoint* out_points,
    uint32_t max_points
);

// Get current network parameters
// Parameters:
//   handle: Engine handle
//   out_params: Pointer to structure to fill with parameters
// Returns: true on success, false on failure
bool cyberspace_engine_get_network_params(
    CyberspaceEngineHandle handle,
    CNetworkParams* out_params
);

// Get number of loaded chunks
// Parameters:
//   handle: Engine handle
// Returns: Number of chunks currently loaded
uint32_t cyberspace_engine_get_chunk_count(CyberspaceEngineHandle handle);

// Get total number of points
// Parameters:
//   handle: Engine handle
// Returns: Total number of points across all loaded chunks
uint32_t cyberspace_engine_get_point_count(CyberspaceEngineHandle handle);

#ifdef __cplusplus
}
#endif

#endif // CYBERSPACE_ENGINE_H
