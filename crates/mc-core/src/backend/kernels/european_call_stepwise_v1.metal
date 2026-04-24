#include <metal_stdlib>
using namespace metal;

constant uint MC_THREADGROUP_WIDTH = 256u;

inline uint hash_u32(uint x) {
    x += 0x9E3779B9u;
    x ^= x >> 16;
    x *= 0x85EBCA6Bu;
    x ^= x >> 13;
    x *= 0xC2B2AE35u;
    x ^= x >> 16;
    return x;
}

inline float open01(uint seed, uint path_idx, uint step_idx, uint lane) {
    uint mixed = seed ^ (path_idx * 747796405u) ^ (step_idx * 2891336453u) ^ (lane * 277803737u);
    uint hashed = hash_u32(mixed);
    return max((float(hashed) + 1.0f) / 4294967297.0f, 1.17549435e-38f);
}

inline float standard_normal(uint seed, uint path_idx, uint step_idx) {
    float u1 = open01(seed, path_idx, step_idx, 0u);
    float u2 = open01(seed, path_idx, step_idx, 1u);
    float radius = sqrt(-2.0f * log(u1));
    float theta = 6.28318530717958647692f * u2;
    return radius * cos(theta);
}

kernel void mc_metal_european_call_stepwise_v1(
    device float* partial_sums [[buffer(0)]],
    device float* partial_sq_sums [[buffer(1)]],
    device float* partial_control_sums [[buffer(2)]],
    device float* partial_control_sq_sums [[buffer(3)]],
    device float* partial_cross_sums [[buffer(4)]],
    constant int& n_paths [[buffer(5)]],
    constant int& n_steps [[buffer(6)]],
    constant float& log_s0 [[buffer(7)]],
    constant float& strike [[buffer(8)]],
    constant float& drift_dt [[buffer(9)]],
    constant float& vol_dt [[buffer(10)]],
    constant float& discount [[buffer(11)]],
    constant uint& seed [[buffer(12)]],
    constant int& technique_mode [[buffer(13)]],
    uint gid [[thread_position_in_grid]],
    uint tid [[thread_index_in_threadgroup]],
    uint group_id [[threadgroup_position_in_grid]]
) {
    threadgroup float local_payoffs[MC_THREADGROUP_WIDTH];
    threadgroup float local_payoff_sq[MC_THREADGROUP_WIDTH];
    threadgroup float local_controls[MC_THREADGROUP_WIDTH];
    threadgroup float local_control_sq[MC_THREADGROUP_WIDTH];
    threadgroup float local_cross[MC_THREADGROUP_WIDTH];

    float payoff = 0.0f;
    float control = 0.0f;
    float cross = 0.0f;

    if (technique_mode == 1) {
        uint pair_count = (static_cast<uint>(n_paths) + 1u) / 2u;
        if (gid < pair_count) {
            float log_a = log_s0;
            float log_b = log_s0;

            for (int step = 0; step < n_steps; ++step) {
                float z = standard_normal(seed, gid, static_cast<uint>(step));
                log_a += drift_dt + vol_dt * z;
                log_b += drift_dt - vol_dt * z;
            }

            float s_a = exp(log_a);
            float s_b = exp(log_b);
            float payoff_a = s_a > strike ? (s_a - strike) * discount : 0.0f;
            float payoff_b = s_b > strike ? (s_b - strike) * discount : 0.0f;
            payoff = 0.5f * (payoff_a + payoff_b);
        }
    } else if (gid < static_cast<uint>(n_paths)) {
        float log_s_t = log_s0;

        for (int step = 0; step < n_steps; ++step) {
            float z = standard_normal(seed, gid, static_cast<uint>(step));
            log_s_t += drift_dt + vol_dt * z;
        }

        float s_t = exp(log_s_t);
        payoff = s_t > strike ? (s_t - strike) * discount : 0.0f;
        if (technique_mode == 2) {
            control = discount * s_t;
            cross = payoff * control;
        }
    }

    local_payoffs[tid] = payoff;
    local_payoff_sq[tid] = payoff * payoff;
    local_controls[tid] = control;
    local_control_sq[tid] = control * control;
    local_cross[tid] = cross;
    threadgroup_barrier(mem_flags::mem_threadgroup);

    for (uint offset = MC_THREADGROUP_WIDTH / 2u; offset > 0; offset >>= 1u) {
        if (tid < offset) {
            local_payoffs[tid] += local_payoffs[tid + offset];
            local_payoff_sq[tid] += local_payoff_sq[tid + offset];
            local_controls[tid] += local_controls[tid + offset];
            local_control_sq[tid] += local_control_sq[tid + offset];
            local_cross[tid] += local_cross[tid + offset];
        }
        threadgroup_barrier(mem_flags::mem_threadgroup);
    }

    if (tid == 0) {
        partial_sums[group_id] = local_payoffs[0];
        partial_sq_sums[group_id] = local_payoff_sq[0];
        partial_control_sums[group_id] = local_controls[0];
        partial_control_sq_sums[group_id] = local_control_sq[0];
        partial_cross_sums[group_id] = local_cross[0];
    }
}

kernel void mc_metal_reduce_sum_f32_v1(
    device const float* input_values [[buffer(0)]],
    device float* output_values [[buffer(1)]],
    constant int& n_values [[buffer(2)]],
    uint gid [[thread_position_in_grid]],
    uint tid [[thread_index_in_threadgroup]],
    uint group_id [[threadgroup_position_in_grid]]
) {
    threadgroup float local_values[MC_THREADGROUP_WIDTH];

    float value = 0.0f;
    if (gid < static_cast<uint>(n_values)) {
        value = input_values[gid];
    }

    local_values[tid] = value;
    threadgroup_barrier(mem_flags::mem_threadgroup);

    for (uint offset = MC_THREADGROUP_WIDTH / 2u; offset > 0; offset >>= 1u) {
        if (tid < offset) {
            local_values[tid] += local_values[tid + offset];
        }
        threadgroup_barrier(mem_flags::mem_threadgroup);
    }

    if (tid == 0) {
        output_values[group_id] = local_values[0];
    }
}
