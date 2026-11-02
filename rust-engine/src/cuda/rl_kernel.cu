#include <cuda_runtime.h>
#include <stdio.h>
#include <math.h>

#define STATE_DIM 64
#define HIDDEN1_DIM 512
#define HIDDEN2_DIM 256
#define ACTION_DIM 15
#define FORMATION_DIM 6

extern "C" {

int cuda_is_available() {
    int deviceCount = 0;
    cudaError_t error = cudaGetDeviceCount(&deviceCount);
    if (error != cudaSuccess || deviceCount == 0) {
        return 0;
    }
    return deviceCount;
}

void cuda_get_device_name(char* buffer, int max_len) {
    int device = 0;
    if (cudaGetDevice(&device) == cudaSuccess) {
        cudaDeviceProp prop;
        if (cudaGetDeviceProperties(&prop, device) == cudaSuccess) {
            snprintf(buffer, max_len, "%s (%d MB)", prop.name, (int)(prop.totalGlobalMem / (1024 * 1024)));
            return;
        }
    }
    snprintf(buffer, max_len, "NVIDIA CUDA GPU");
}

// Kernel LeakyReLU: x >= 0 ? x : 0.01 * x
__device__ inline float leaky_relu(float x) {
    return x >= 0.0f ? x : 0.01f * x;
}

__device__ inline float leaky_relu_derivative(float x) {
    return x >= 0.0f ? 1.0f : 0.01f;
}

// Forward Layer 1: [batch_size, STATE_DIM] x [STATE_DIM, HIDDEN1_DIM] + [HIDDEN1_DIM] -> [batch_size, HIDDEN1_DIM]
__global__ void forward_l1_kernel(
    const float* d_states,
    const float* d_w1,
    const float* d_b1,
    float* d_h1,
    int batch_size
) {
    int b = blockIdx.y;
    int j = blockIdx.x * blockDim.x + threadIdx.x;
    if (b < batch_size && j < HIDDEN1_DIM) {
        float sum = d_b1[j];
        for (int i = 0; i < STATE_DIM; ++i) {
            sum += d_states[b * STATE_DIM + i] * d_w1[i * HIDDEN1_DIM + j];
        }
        d_h1[b * HIDDEN1_DIM + j] = leaky_relu(sum);
    }
}

// Forward Layer 2: [batch_size, HIDDEN1_DIM] x [HIDDEN1_DIM, HIDDEN2_DIM] + [HIDDEN2_DIM] -> [batch_size, HIDDEN2_DIM]
__global__ void forward_l2_kernel(
    const float* d_h1,
    const float* d_w_h2,
    const float* d_b_h2,
    float* d_h2,
    int batch_size
) {
    int b = blockIdx.y;
    int k = blockIdx.x * blockDim.x + threadIdx.x;
    if (b < batch_size && k < HIDDEN2_DIM) {
        float sum = d_b_h2[k];
        for (int j = 0; j < HIDDEN1_DIM; ++j) {
            sum += d_h1[b * HIDDEN1_DIM + j] * d_w_h2[j * HIDDEN2_DIM + k];
        }
        d_h2[b * HIDDEN2_DIM + k] = leaky_relu(sum);
    }
}

// Forward Action Head: [batch_size, HIDDEN2_DIM] x [HIDDEN2_DIM, ACTION_DIM] + [ACTION_DIM] -> [batch_size, ACTION_DIM]
__global__ void forward_action_kernel(
    const float* d_h2,
    const float* d_w2,
    const float* d_b2,
    float* d_action_q,
    int batch_size
) {
    int b = blockIdx.y;
    int o = blockIdx.x * blockDim.x + threadIdx.x;
    if (b < batch_size && o < ACTION_DIM) {
        float sum = d_b2[o];
        for (int k = 0; k < HIDDEN2_DIM; ++k) {
            sum += d_h2[b * HIDDEN2_DIM + k] * d_w2[k * ACTION_DIM + o];
        }
        d_action_q[b * ACTION_DIM + o] = sum;
    }
}

// Forward Formation Head: [batch_size, HIDDEN2_DIM] x [HIDDEN2_DIM, FORMATION_DIM] + [FORMATION_DIM] -> [batch_size, FORMATION_DIM]
__global__ void forward_formation_kernel(
    const float* d_h2,
    const float* d_w_form,
    const float* d_b_form,
    float* d_form_q,
    int batch_size
) {
    int b = blockIdx.y;
    int f = blockIdx.x * blockDim.x + threadIdx.x;
    if (b < batch_size && f < FORMATION_DIM) {
        float sum = d_b_form[f];
        for (int k = 0; k < HIDDEN2_DIM; ++k) {
            sum += d_h2[b * HIDDEN2_DIM + k] * d_w_form[k * FORMATION_DIM + f];
        }
        d_form_q[b * FORMATION_DIM + f] = sum;
    }
}

// Host function: Batch Forward Pass trên GPU
void cuda_batch_forward(
    const float* h_states,
    const float* h_w1,
    const float* h_b1,
    const float* h_w_h2,
    const float* h_b_h2,
    const float* h_w2,
    const float* h_b2,
    const float* h_w_form,
    const float* h_b_form,
    float* h_action_q,
    float* h_form_q,
    int batch_size
) {
    float *d_states, *d_w1, *d_b1, *d_h1, *d_w_h2, *d_b_h2, *d_h2, *d_w2, *d_b2, *d_w_form, *d_b_form;
    float *d_action_q, *d_form_q;

    cudaMalloc(&d_states, batch_size * STATE_DIM * sizeof(float));
    cudaMalloc(&d_w1, STATE_DIM * HIDDEN1_DIM * sizeof(float));
    cudaMalloc(&d_b1, HIDDEN1_DIM * sizeof(float));
    cudaMalloc(&d_h1, batch_size * HIDDEN1_DIM * sizeof(float));
    cudaMalloc(&d_w_h2, HIDDEN1_DIM * HIDDEN2_DIM * sizeof(float));
    cudaMalloc(&d_b_h2, HIDDEN2_DIM * sizeof(float));
    cudaMalloc(&d_h2, batch_size * HIDDEN2_DIM * sizeof(float));
    cudaMalloc(&d_w2, HIDDEN2_DIM * ACTION_DIM * sizeof(float));
    cudaMalloc(&d_b2, ACTION_DIM * sizeof(float));
    cudaMalloc(&d_w_form, HIDDEN2_DIM * FORMATION_DIM * sizeof(float));
    cudaMalloc(&d_b_form, FORMATION_DIM * sizeof(float));
    cudaMalloc(&d_action_q, batch_size * ACTION_DIM * sizeof(float));
    cudaMalloc(&d_form_q, batch_size * FORMATION_DIM * sizeof(float));

    cudaMemcpy(d_states, h_states, batch_size * STATE_DIM * sizeof(float), cudaMemcpyHostToDevice);
    cudaMemcpy(d_w1, h_w1, STATE_DIM * HIDDEN1_DIM * sizeof(float), cudaMemcpyHostToDevice);
    cudaMemcpy(d_b1, h_b1, HIDDEN1_DIM * sizeof(float), cudaMemcpyHostToDevice);
    cudaMemcpy(d_w_h2, h_w_h2, HIDDEN1_DIM * HIDDEN2_DIM * sizeof(float), cudaMemcpyHostToDevice);
    cudaMemcpy(d_b_h2, h_b_h2, HIDDEN2_DIM * sizeof(float), cudaMemcpyHostToDevice);
    cudaMemcpy(d_w2, h_w2, HIDDEN2_DIM * ACTION_DIM * sizeof(float), cudaMemcpyHostToDevice);
    cudaMemcpy(d_b2, h_b2, ACTION_DIM * sizeof(float), cudaMemcpyHostToDevice);
    cudaMemcpy(d_w_form, h_w_form, HIDDEN2_DIM * FORMATION_DIM * sizeof(float), cudaMemcpyHostToDevice);
    cudaMemcpy(d_b_form, h_b_form, FORMATION_DIM * sizeof(float), cudaMemcpyHostToDevice);

    dim3 block(128);
    dim3 grid_l1((HIDDEN1_DIM + 127) / 128, batch_size);
    forward_l1_kernel<<<grid_l1, block>>>(d_states, d_w1, d_b1, d_h1, batch_size);

    dim3 grid_l2((HIDDEN2_DIM + 127) / 128, batch_size);
    forward_l2_kernel<<<grid_l2, block>>>(d_h1, d_w_h2, d_b_h2, d_h2, batch_size);

    dim3 grid_act((ACTION_DIM + 127) / 128, batch_size);
    forward_action_kernel<<<grid_act, block>>>(d_h2, d_w2, d_b2, d_action_q, batch_size);

    dim3 grid_form((FORMATION_DIM + 127) / 128, batch_size);
    forward_formation_kernel<<<grid_form, block>>>(d_h2, d_w_form, d_b_form, d_form_q, batch_size);

    cudaDeviceSynchronize();

    cudaMemcpy(h_action_q, d_action_q, batch_size * ACTION_DIM * sizeof(float), cudaMemcpyDeviceToHost);
    cudaMemcpy(h_form_q, d_form_q, batch_size * FORMATION_DIM * sizeof(float), cudaMemcpyDeviceToHost);

    cudaFree(d_states);
    cudaFree(d_w1);
    cudaFree(d_b1);
    cudaFree(d_h1);
    cudaFree(d_w_h2);
    cudaFree(d_b_h2);
    cudaFree(d_h2);
    cudaFree(d_w2);
    cudaFree(d_b2);
    cudaFree(d_w_form);
    cudaFree(d_b_form);
    cudaFree(d_action_q);
    cudaFree(d_form_q);
}

// Kernel Backpropagation Mini-Batch Q-learning
__global__ void batch_train_kernel(
    const float* d_states,
    const int* d_actions,
    const int* d_formations,
    const float* d_rewards,
    const float* d_next_states,
    const int* d_dones,
    float* d_w1,
    float* d_b1,
    float* d_w_h2,
    float* d_b_h2,
    float* d_w2,
    float* d_b2,
    float* d_w_form,
    float* d_b_form,
    float lr,
    float gamma,
    int batch_size
) {
    int b = blockIdx.x * blockDim.x + threadIdx.x;
    if (b >= batch_size) return;

    // Forward pass state
    float h1[HIDDEN1_DIM];
    for (int j = 0; j < HIDDEN1_DIM; ++j) {
        float sum = d_b1[j];
        for (int i = 0; i < STATE_DIM; ++i) {
            sum += d_states[b * STATE_DIM + i] * d_w1[i * HIDDEN1_DIM + j];
        }
        h1[j] = leaky_relu(sum);
    }

    float h2[HIDDEN2_DIM];
    for (int k = 0; k < HIDDEN2_DIM; ++k) {
        float sum = d_b_h2[k];
        for (int j = 0; j < HIDDEN1_DIM; ++j) {
            sum += h1[j] * d_w_h2[j * HIDDEN2_DIM + k];
        }
        h2[k] = leaky_relu(sum);
    }

    int act_idx = d_actions[b];
    int form_idx = d_formations[b];

    float cur_q_act = d_b2[act_idx];
    for (int k = 0; k < HIDDEN2_DIM; ++k) {
        cur_q_act += h2[k] * d_w2[k * ACTION_DIM + act_idx];
    }

    float cur_q_form = d_b_form[form_idx];
    for (int k = 0; k < HIDDEN2_DIM; ++k) {
        cur_q_form += h2[k] * d_w_form[k * FORMATION_DIM + form_idx];
    }

    // Forward next state for target Q
    float target_act = d_rewards[b];
    float target_form = d_rewards[b];

    if (!d_dones[b]) {
        float next_h1[HIDDEN1_DIM];
        for (int j = 0; j < HIDDEN1_DIM; ++j) {
            float sum = d_b1[j];
            for (int i = 0; i < STATE_DIM; ++i) {
                sum += d_next_states[b * STATE_DIM + i] * d_w1[i * HIDDEN1_DIM + j];
            }
            next_h1[j] = leaky_relu(sum);
        }

        float next_h2[HIDDEN2_DIM];
        for (int k = 0; k < HIDDEN2_DIM; ++k) {
            float sum = d_b_h2[k];
            for (int j = 0; j < HIDDEN1_DIM; ++j) {
                sum += next_h1[j] * d_w_h2[j * HIDDEN2_DIM + k];
            }
            next_h2[k] = leaky_relu(sum);
        }

        float max_next_q_act = -1e9f;
        for (int o = 0; o < ACTION_DIM; ++o) {
            float q = d_b2[o];
            for (int k = 0; k < HIDDEN2_DIM; ++k) {
                q += next_h2[k] * d_w2[k * ACTION_DIM + o];
            }
            if (q > max_next_q_act) max_next_q_act = q;
        }

        float max_next_q_form = -1e9f;
        for (int f = 0; f < FORMATION_DIM; ++f) {
            float q = d_b_form[f];
            for (int k = 0; k < HIDDEN2_DIM; ++k) {
                q += next_h2[k] * d_w_form[k * FORMATION_DIM + f];
            }
            if (q > max_next_q_form) max_next_q_form = q;
        }

        target_act += gamma * max_next_q_act;
        target_form += gamma * max_next_q_form;
    }

    float td_act = fmaxf(-1.0f, fminf(1.0f, target_act - cur_q_act));
    float td_form = fmaxf(-1.0f, fminf(1.0f, target_form - cur_q_form));

    // Gradient updates to layer 2 & heads
    float grad_h2[HIDDEN2_DIM];
    for (int k = 0; k < HIDDEN2_DIM; ++k) {
        float g_act = d_w2[k * ACTION_DIM + act_idx] * td_act;
        atomicAdd(&d_w2[k * ACTION_DIM + act_idx], lr * td_act * h2[k]);

        float g_form = d_w_form[k * FORMATION_DIM + form_idx] * td_form;
        atomicAdd(&d_w_form[k * FORMATION_DIM + form_idx], lr * td_form * h2[k]);

        grad_h2[k] = g_act + g_form * 0.7f;
    }
    atomicAdd(&d_b2[act_idx], lr * td_act);
    atomicAdd(&d_b_form[form_idx], lr * td_form);

    // Backprop to layer 1
    float grad_h1[HIDDEN1_DIM];
    for (int j = 0; j < HIDDEN1_DIM; ++j) {
        grad_h1[j] = 0.0f;
    }

    for (int k = 0; k < HIDDEN2_DIM; ++k) {
        float delta_k = grad_h2[k] * leaky_relu_derivative(h2[k]);
        for (int j = 0; j < HIDDEN1_DIM; ++j) {
            grad_h1[j] += d_w_h2[j * HIDDEN2_DIM + k] * delta_k;
            atomicAdd(&d_w_h2[j * HIDDEN2_DIM + k], lr * delta_k * h1[j]);
        }
        atomicAdd(&d_b_h2[k], lr * delta_k);
    }

    // Backprop to input layer
    for (int j = 0; j < HIDDEN1_DIM; ++j) {
        float delta_j = grad_h1[j] * leaky_relu_derivative(h1[j]);
        for (int i = 0; i < STATE_DIM; ++i) {
            atomicAdd(&d_w1[i * HIDDEN1_DIM + j], lr * delta_j * d_states[b * STATE_DIM + i]);
        }
        atomicAdd(&d_b1[j], lr * delta_j);
    }
}

// Host function: Batch Train trên GPU
void cuda_batch_train(
    const float* h_states,
    const int* h_actions,
    const int* h_formations,
    const float* h_rewards,
    const float* h_next_states,
    const int* h_dones,
    float* h_w1,
    float* h_b1,
    float* h_w_h2,
    float* h_b_h2,
    float* h_w2,
    float* h_b2,
    float* h_w_form,
    float* h_b_form,
    float lr,
    float gamma,
    int batch_size
) {
    float *d_states, *d_rewards, *d_next_states;
    int *d_actions, *d_formations, *d_dones;
    float *d_w1, *d_b1, *d_w_h2, *d_b_h2, *d_w2, *d_b2, *d_w_form, *d_b_form;

    cudaMalloc(&d_states, batch_size * STATE_DIM * sizeof(float));
    cudaMalloc(&d_actions, batch_size * sizeof(int));
    cudaMalloc(&d_formations, batch_size * sizeof(int));
    cudaMalloc(&d_rewards, batch_size * sizeof(float));
    cudaMalloc(&d_next_states, batch_size * STATE_DIM * sizeof(float));
    cudaMalloc(&d_dones, batch_size * sizeof(int));

    cudaMalloc(&d_w1, STATE_DIM * HIDDEN1_DIM * sizeof(float));
    cudaMalloc(&d_b1, HIDDEN1_DIM * sizeof(float));
    cudaMalloc(&d_w_h2, HIDDEN1_DIM * HIDDEN2_DIM * sizeof(float));
    cudaMalloc(&d_b_h2, HIDDEN2_DIM * sizeof(float));
    cudaMalloc(&d_w2, HIDDEN2_DIM * ACTION_DIM * sizeof(float));
    cudaMalloc(&d_b2, ACTION_DIM * sizeof(float));
    cudaMalloc(&d_w_form, HIDDEN2_DIM * FORMATION_DIM * sizeof(float));
    cudaMalloc(&d_b_form, FORMATION_DIM * sizeof(float));

    cudaMemcpy(d_states, h_states, batch_size * STATE_DIM * sizeof(float), cudaMemcpyHostToDevice);
    cudaMemcpy(d_actions, h_actions, batch_size * sizeof(int), cudaMemcpyHostToDevice);
    cudaMemcpy(d_formations, h_formations, batch_size * sizeof(int), cudaMemcpyHostToDevice);
    cudaMemcpy(d_rewards, h_rewards, batch_size * sizeof(float), cudaMemcpyHostToDevice);
    cudaMemcpy(d_next_states, h_next_states, batch_size * STATE_DIM * sizeof(float), cudaMemcpyHostToDevice);
    cudaMemcpy(d_dones, h_dones, batch_size * sizeof(int), cudaMemcpyHostToDevice);

    cudaMemcpy(d_w1, h_w1, STATE_DIM * HIDDEN1_DIM * sizeof(float), cudaMemcpyHostToDevice);
    cudaMemcpy(d_b1, h_b1, HIDDEN1_DIM * sizeof(float), cudaMemcpyHostToDevice);
    cudaMemcpy(d_w_h2, h_w_h2, HIDDEN1_DIM * HIDDEN2_DIM * sizeof(float), cudaMemcpyHostToDevice);
    cudaMemcpy(d_b_h2, h_b_h2, HIDDEN2_DIM * sizeof(float), cudaMemcpyHostToDevice);
    cudaMemcpy(d_w2, h_w2, HIDDEN2_DIM * ACTION_DIM * sizeof(float), cudaMemcpyHostToDevice);
    cudaMemcpy(d_b2, h_b2, ACTION_DIM * sizeof(float), cudaMemcpyHostToDevice);
    cudaMemcpy(d_w_form, h_w_form, HIDDEN2_DIM * FORMATION_DIM * sizeof(float), cudaMemcpyHostToDevice);
    cudaMemcpy(d_b_form, h_b_form, FORMATION_DIM * sizeof(float), cudaMemcpyHostToDevice);

    int threads = 64;
    int blocks = (batch_size + threads - 1) / threads;
    batch_train_kernel<<<blocks, threads>>>(
        d_states, d_actions, d_formations, d_rewards, d_next_states, d_dones,
        d_w1, d_b1, d_w_h2, d_b_h2, d_w2, d_b2, d_w_form, d_b_form,
        lr, gamma, batch_size
    );

    cudaDeviceSynchronize();

    cudaMemcpy(h_w1, d_w1, STATE_DIM * HIDDEN1_DIM * sizeof(float), cudaMemcpyDeviceToHost);
    cudaMemcpy(h_b1, d_b1, HIDDEN1_DIM * sizeof(float), cudaMemcpyDeviceToHost);
    cudaMemcpy(h_w_h2, d_w_h2, HIDDEN1_DIM * HIDDEN2_DIM * sizeof(float), cudaMemcpyDeviceToHost);
    cudaMemcpy(h_b_h2, d_b_h2, HIDDEN2_DIM * sizeof(float), cudaMemcpyDeviceToHost);
    cudaMemcpy(h_w2, d_w2, HIDDEN2_DIM * ACTION_DIM * sizeof(float), cudaMemcpyDeviceToHost);
    cudaMemcpy(h_b2, d_b2, ACTION_DIM * sizeof(float), cudaMemcpyDeviceToHost);
    cudaMemcpy(h_w_form, d_w_form, HIDDEN2_DIM * FORMATION_DIM * sizeof(float), cudaMemcpyDeviceToHost);
    cudaMemcpy(h_b_form, d_b_form, FORMATION_DIM * sizeof(float), cudaMemcpyDeviceToHost);

    cudaFree(d_states);
    cudaFree(d_actions);
    cudaFree(d_formations);
    cudaFree(d_rewards);
    cudaFree(d_next_states);
    cudaFree(d_dones);
    cudaFree(d_w1);
    cudaFree(d_b1);
    cudaFree(d_w_h2);
    cudaFree(d_b_h2);
    cudaFree(d_w2);
    cudaFree(d_b2);
    cudaFree(d_w_form);
    cudaFree(d_b_form);
}

} // extern "C"

