#ifndef CYBERSPACE_ENGINE_H
#define CYBERSPACE_ENGINE_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef void* CyberspaceEngineHandle;

typedef struct {
    float position[3];
    float color[3];
    float brightness;
    float size;
} CPoint;

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

CyberspaceEngineHandle cyberspace_engine_create(
    float chunk_size,
    float render_distance,
    uint32_t seed
);

void cyberspace_engine_destroy(CyberspaceEngineHandle handle);

void cyberspace_engine_update_camera(
    CyberspaceEngineHandle handle,
    float x,
    float y,
    float z
);

uint32_t cyberspace_engine_get_points(
    CyberspaceEngineHandle handle,
    CPoint* out_points,
    uint32_t max_points
);

bool cyberspace_engine_get_network_params(
    CyberspaceEngineHandle handle,
    CNetworkParams* out_params
);

uint32_t cyberspace_engine_get_chunk_count(CyberspaceEngineHandle handle);

uint32_t cyberspace_engine_get_point_count(CyberspaceEngineHandle handle);

#ifdef __cplusplus
}
#endif

#endif

