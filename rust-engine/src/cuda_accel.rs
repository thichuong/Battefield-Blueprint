use crate::rl::RlBrain;

#[cfg(has_cuda)]
use crate::rl::{ACTION_DIM, DEFAULT_HIDDEN2_SIZE, DEFAULT_HIDDEN_SIZE, FORMATION_DIM, STATE_DIM};

#[cfg(has_cuda)]
extern "C" {
    fn cuda_is_available() -> std::os::raw::c_int;
    fn cuda_get_device_name(buffer: *mut std::os::raw::c_char, max_len: std::os::raw::c_int);
    fn cuda_batch_train(
        states: *const f32,
        actions: *const i32,
        formations: *const i32,
        rewards: *const f32,
        next_states: *const f32,
        dones: *const i32,
        w1: *mut f32,
        b1: *mut f32,
        w_h2: *mut f32,
        b_h2: *mut f32,
        w2: *mut f32,
        b2: *mut f32,
        w_form: *mut f32,
        b_form: *mut f32,
        lr: f32,
        gamma: f32,
        batch_size: i32,
    );

    fn cuda_persistent_init(
        max_batch_size: std::os::raw::c_int,
        w1: *const f32,
        b1: *const f32,
        w_h2: *const f32,
        b_h2: *const f32,
        w2: *const f32,
        b2: *const f32,
        w_form: *const f32,
        b_form: *const f32,
    ) -> *mut std::os::raw::c_void;

    fn cuda_persistent_train_batch(
        ctx: *mut std::os::raw::c_void,
        states: *const f32,
        actions: *const i32,
        formations: *const i32,
        rewards: *const f32,
        next_states: *const f32,
        dones: *const i32,
        lr: f32,
        gamma: f32,
        batch_size: std::os::raw::c_int,
    );

    fn cuda_persistent_sync_weights(
        ctx: *mut std::os::raw::c_void,
        w1: *mut f32,
        b1: *mut f32,
        w_h2: *mut f32,
        b_h2: *mut f32,
        w2: *mut f32,
        b2: *mut f32,
        w_form: *mut f32,
        b_form: *mut f32,
    );

    fn cuda_persistent_free(ctx: *mut std::os::raw::c_void);
}

pub struct CudaDeviceManager;

impl CudaDeviceManager {
    pub fn is_gpu_available() -> bool {
        #[cfg(has_cuda)]
        unsafe {
            cuda_is_available() > 0
        }
        #[cfg(not(has_cuda))]
        {
            false
        }
    }

    pub fn device_name() -> String {
        #[cfg(has_cuda)]
        unsafe {
            let mut buf = [0u8; 256];
            cuda_get_device_name(buf.as_mut_ptr() as *mut std::os::raw::c_char, 256);
            std::ffi::CStr::from_ptr(buf.as_ptr() as *const std::os::raw::c_char)
                .to_string_lossy()
                .into_owned()
        }
        #[cfg(not(has_cuda))]
        {
            "CPU Fallback".to_string()
        }
    }
}

#[derive(Clone, Debug)]
pub struct TransitionSample {
    pub state: Vec<f32>,
    pub action: usize,
    pub formation: usize,
    pub reward: f32,
    pub next_state: Vec<f32>,
    pub done: bool,
}

pub struct GpuBatchTrainer {
    pub buffer: Vec<TransitionSample>,
    pub batch_size: usize,
    pub use_gpu: bool,
}

impl GpuBatchTrainer {
    pub fn new(batch_size: usize) -> Self {
        let use_gpu = CudaDeviceManager::is_gpu_available();
        Self {
            buffer: Vec::with_capacity(batch_size * 2),
            batch_size,
            use_gpu,
        }
    }

    pub fn push(&mut self, sample: TransitionSample) {
        self.buffer.push(sample);
    }

    pub fn can_train(&self) -> bool {
        self.buffer.len() >= self.batch_size
    }

    pub fn train_batch(&mut self, brain: &mut RlBrain) {
        if self.buffer.is_empty() {
            return;
        }

        let count = self.buffer.len().min(self.batch_size);
        let batch: Vec<TransitionSample> = self.buffer.drain(0..count).collect();

        if self.use_gpu {
            #[cfg(has_cuda)]
            {
                self.train_gpu(&batch, brain);
                return;
            }
        }

        // CPU Fallback
        for s in batch {
            brain.train_step_with_formation(
                &s.state,
                crate::rl::BotStrategicAction::from_index(s.action),
                crate::components::FormationType::from_index(s.formation),
                s.reward,
                &s.next_state,
                s.done,
            );
        }
    }

    #[cfg(has_cuda)]
    fn train_gpu(&self, batch: &[TransitionSample], brain: &mut RlBrain) {
        let b_len = batch.len();
        let mut flat_states = Vec::with_capacity(b_len * STATE_DIM);
        let mut actions = Vec::with_capacity(b_len);
        let mut formations = Vec::with_capacity(b_len);
        let mut rewards = Vec::with_capacity(b_len);
        let mut flat_next_states = Vec::with_capacity(b_len * STATE_DIM);
        let mut dones = Vec::with_capacity(b_len);

        for item in batch {
            flat_states.extend_from_slice(&item.state);
            actions.push(item.action as i32);
            formations.push(item.formation as i32);
            rewards.push(item.reward);
            flat_next_states.extend_from_slice(&item.next_state);
            dones.push(if item.done { 1 } else { 0 });
        }

        // Flatten weights
        let mut flat_w1 = Vec::with_capacity(STATE_DIM * DEFAULT_HIDDEN_SIZE);
        for row in &brain.w1 {
            flat_w1.extend_from_slice(row);
        }
        let mut flat_b1 = brain.b1.clone();

        let mut flat_w_h2 = Vec::with_capacity(DEFAULT_HIDDEN_SIZE * DEFAULT_HIDDEN2_SIZE);
        for row in &brain.w_h2 {
            flat_w_h2.extend_from_slice(row);
        }
        let mut flat_b_h2 = brain.b_h2.clone();

        let mut flat_w2 = Vec::with_capacity(DEFAULT_HIDDEN2_SIZE * ACTION_DIM);
        for row in &brain.w2 {
            flat_w2.extend_from_slice(row);
        }
        let mut flat_b2 = brain.b2.clone();

        let mut flat_w_form = Vec::with_capacity(DEFAULT_HIDDEN2_SIZE * FORMATION_DIM);
        for row in &brain.w_form {
            flat_w_form.extend_from_slice(row);
        }
        let mut flat_b_form = brain.b_form.clone();

        unsafe {
            cuda_batch_train(
                flat_states.as_ptr(),
                actions.as_ptr(),
                formations.as_ptr(),
                rewards.as_ptr(),
                flat_next_states.as_ptr(),
                dones.as_ptr(),
                flat_w1.as_mut_ptr(),
                flat_b1.as_mut_ptr(),
                flat_w_h2.as_mut_ptr(),
                flat_b_h2.as_mut_ptr(),
                flat_w2.as_mut_ptr(),
                flat_b2.as_mut_ptr(),
                flat_w_form.as_mut_ptr(),
                flat_b_form.as_mut_ptr(),
                brain.learning_rate,
                brain.gamma,
                b_len as i32,
            );
        }

        // Unflatten weights back to brain with NaN/Inf safeguard and [-5.0, 5.0] weight clamping
        for i in 0..STATE_DIM {
            for j in 0..DEFAULT_HIDDEN_SIZE {
                let v = flat_w1[i * DEFAULT_HIDDEN_SIZE + j];
                brain.w1[i][j] = if v.is_finite() { v.clamp(-5.0, 5.0) } else { 0.0 };
            }
        }
        for j in 0..DEFAULT_HIDDEN_SIZE {
            let v = flat_b1[j];
            brain.b1[j] = if v.is_finite() { v.clamp(-5.0, 5.0) } else { 0.0 };
        }

        for j in 0..DEFAULT_HIDDEN_SIZE {
            for k in 0..DEFAULT_HIDDEN2_SIZE {
                let v = flat_w_h2[j * DEFAULT_HIDDEN2_SIZE + k];
                brain.w_h2[j][k] = if v.is_finite() { v.clamp(-5.0, 5.0) } else { 0.0 };
            }
        }
        for k in 0..DEFAULT_HIDDEN2_SIZE {
            let v = flat_b_h2[k];
            brain.b_h2[k] = if v.is_finite() { v.clamp(-5.0, 5.0) } else { 0.0 };
        }

        for k in 0..DEFAULT_HIDDEN2_SIZE {
            for o in 0..ACTION_DIM {
                let v = flat_w2[k * ACTION_DIM + o];
                brain.w2[k][o] = if v.is_finite() { v.clamp(-5.0, 5.0) } else { 0.0 };
            }
        }
        for o in 0..ACTION_DIM {
            let v = flat_b2[o];
            brain.b2[o] = if v.is_finite() { v.clamp(-5.0, 5.0) } else { 0.0 };
        }

        for k in 0..DEFAULT_HIDDEN2_SIZE {
            for f in 0..FORMATION_DIM {
                let v = flat_w_form[k * FORMATION_DIM + f];
                brain.w_form[k][f] = if v.is_finite() { v.clamp(-5.0, 5.0) } else { 0.0 };
            }
        }
        for f in 0..FORMATION_DIM {
            let v = flat_b_form[f];
            brain.b_form[f] = if v.is_finite() { v.clamp(-5.0, 5.0) } else { 0.0 };
        }
    }
}

/// Bộ huấn luyện GPU giữ trọng số và bộ đệm thường trú trong VRAM
/// Không cấp phát hay giải phóng bộ nhớ qua từng batch (Zero cudaMalloc / Zero cudaFree)
pub struct PersistentGpuTrainer {
    #[cfg(has_cuda)]
    ctx: *mut std::os::raw::c_void,
    pub max_batch_size: usize,
    pub use_gpu: bool,
    pub fallback_brain: Option<RlBrain>,
}

unsafe impl Send for PersistentGpuTrainer {}
unsafe impl Sync for PersistentGpuTrainer {}

impl PersistentGpuTrainer {
    pub fn new(brain: &RlBrain, max_batch_size: usize) -> Self {
        let use_gpu = CudaDeviceManager::is_gpu_available();
        if use_gpu {
            #[cfg(has_cuda)]
            {
                let mut flat_w1 = Vec::with_capacity(STATE_DIM * DEFAULT_HIDDEN_SIZE);
                for row in &brain.w1 {
                    flat_w1.extend_from_slice(row);
                }
                let flat_b1 = brain.b1.clone();

                let mut flat_w_h2 = Vec::with_capacity(DEFAULT_HIDDEN_SIZE * DEFAULT_HIDDEN2_SIZE);
                for row in &brain.w_h2 {
                    flat_w_h2.extend_from_slice(row);
                }
                let flat_b_h2 = brain.b_h2.clone();

                let mut flat_w2 = Vec::with_capacity(DEFAULT_HIDDEN2_SIZE * ACTION_DIM);
                for row in &brain.w2 {
                    flat_w2.extend_from_slice(row);
                }
                let flat_b2 = brain.b2.clone();

                let mut flat_w_form = Vec::with_capacity(DEFAULT_HIDDEN2_SIZE * FORMATION_DIM);
                for row in &brain.w_form {
                    flat_w_form.extend_from_slice(row);
                }
                let flat_b_form = brain.b_form.clone();

                unsafe {
                    let ctx = cuda_persistent_init(
                        max_batch_size as i32,
                        flat_w1.as_ptr(),
                        flat_b1.as_ptr(),
                        flat_w_h2.as_ptr(),
                        flat_b_h2.as_ptr(),
                        flat_w2.as_ptr(),
                        flat_b2.as_ptr(),
                        flat_w_form.as_ptr(),
                        flat_b_form.as_ptr(),
                    );
                    if !ctx.is_null() {
                        return Self {
                            ctx,
                            max_batch_size,
                            use_gpu: true,
                            fallback_brain: None,
                        };
                    }
                }
            }
        }

        Self {
            #[cfg(has_cuda)]
            ctx: std::ptr::null_mut(),
            max_batch_size,
            use_gpu: false,
            fallback_brain: Some(brain.clone()),
        }
    }

    pub fn train_batch(&mut self, batch: &[TransitionSample], lr: f32, gamma: f32) {
        if batch.is_empty() {
            return;
        }

        if self.use_gpu {
            #[cfg(has_cuda)]
            {
                if !self.ctx.is_null() {
                    let b_len = batch.len().min(self.max_batch_size);
                    let mut flat_states = Vec::with_capacity(b_len * STATE_DIM);
                    let mut actions = Vec::with_capacity(b_len);
                    let mut formations = Vec::with_capacity(b_len);
                    let mut rewards = Vec::with_capacity(b_len);
                    let mut flat_next_states = Vec::with_capacity(b_len * STATE_DIM);
                    let mut dones = Vec::with_capacity(b_len);

                    for item in &batch[0..b_len] {
                        flat_states.extend_from_slice(&item.state);
                        actions.push(item.action as i32);
                        formations.push(item.formation as i32);
                        rewards.push(item.reward);
                        flat_next_states.extend_from_slice(&item.next_state);
                        dones.push(if item.done { 1 } else { 0 });
                    }

                    unsafe {
                        cuda_persistent_train_batch(
                            self.ctx,
                            flat_states.as_ptr(),
                            actions.as_ptr(),
                            formations.as_ptr(),
                            rewards.as_ptr(),
                            flat_next_states.as_ptr(),
                            dones.as_ptr(),
                            lr,
                            gamma,
                            b_len as i32,
                        );
                    }
                    return;
                }
            }
        }

        // CPU Fallback
        if let Some(brain) = self.fallback_brain.as_mut() {
            brain.learning_rate = lr;
            brain.gamma = gamma;
            for s in batch {
                brain.train_step_with_formation(
                    &s.state,
                    crate::rl::BotStrategicAction::from_index(s.action),
                    crate::components::FormationType::from_index(s.formation),
                    s.reward,
                    &s.next_state,
                    s.done,
                );
            }
        }
    }

    pub fn sync_to_brain(&self, brain: &mut RlBrain) {
        if self.use_gpu {
            #[cfg(has_cuda)]
            {
                if !self.ctx.is_null() {
                    let mut flat_w1 = vec![0.0f32; STATE_DIM * DEFAULT_HIDDEN_SIZE];
                    let mut flat_b1 = vec![0.0f32; DEFAULT_HIDDEN_SIZE];
                    let mut flat_w_h2 = vec![0.0f32; DEFAULT_HIDDEN_SIZE * DEFAULT_HIDDEN2_SIZE];
                    let mut flat_b_h2 = vec![0.0f32; DEFAULT_HIDDEN2_SIZE];
                    let mut flat_w2 = vec![0.0f32; DEFAULT_HIDDEN2_SIZE * ACTION_DIM];
                    let mut flat_b2 = vec![0.0f32; ACTION_DIM];
                    let mut flat_w_form = vec![0.0f32; DEFAULT_HIDDEN2_SIZE * FORMATION_DIM];
                    let mut flat_b_form = vec![0.0f32; FORMATION_DIM];

                    unsafe {
                        cuda_persistent_sync_weights(
                            self.ctx,
                            flat_w1.as_mut_ptr(),
                            flat_b1.as_mut_ptr(),
                            flat_w_h2.as_mut_ptr(),
                            flat_b_h2.as_mut_ptr(),
                            flat_w2.as_mut_ptr(),
                            flat_b2.as_mut_ptr(),
                            flat_w_form.as_mut_ptr(),
                            flat_b_form.as_mut_ptr(),
                        );
                    }

                    // Copy back with gentle weight decay (L2 regularization) and safe clamping
                    let decay = 0.999f32;
                    for i in 0..STATE_DIM {
                        for j in 0..DEFAULT_HIDDEN_SIZE {
                            let v = flat_w1[i * DEFAULT_HIDDEN_SIZE + j] * decay;
                            brain.w1[i][j] = if v.is_finite() { v.clamp(-3.0, 3.0) } else { 0.0 };
                        }
                    }
                    for j in 0..DEFAULT_HIDDEN_SIZE {
                        let v = flat_b1[j] * decay;
                        brain.b1[j] = if v.is_finite() { v.clamp(-3.0, 3.0) } else { 0.0 };
                    }

                    for j in 0..DEFAULT_HIDDEN_SIZE {
                        for k in 0..DEFAULT_HIDDEN2_SIZE {
                            let v = flat_w_h2[j * DEFAULT_HIDDEN2_SIZE + k] * decay;
                            brain.w_h2[j][k] = if v.is_finite() { v.clamp(-3.0, 3.0) } else { 0.0 };
                        }
                    }
                    for k in 0..DEFAULT_HIDDEN2_SIZE {
                        let v = flat_b_h2[k] * decay;
                        brain.b_h2[k] = if v.is_finite() { v.clamp(-3.0, 3.0) } else { 0.0 };
                    }

                    for k in 0..DEFAULT_HIDDEN2_SIZE {
                        for o in 0..ACTION_DIM {
                            let v = flat_w2[k * ACTION_DIM + o] * decay;
                            brain.w2[k][o] = if v.is_finite() { v.clamp(-3.0, 3.0) } else { 0.0 };
                        }
                    }
                    for o in 0..ACTION_DIM {
                        let v = flat_b2[o] * decay;
                        brain.b2[o] = if v.is_finite() { v.clamp(-3.0, 3.0) } else { 0.0 };
                    }

                    for k in 0..DEFAULT_HIDDEN2_SIZE {
                        for f in 0..FORMATION_DIM {
                            let v = flat_w_form[k * FORMATION_DIM + f] * decay;
                            brain.w_form[k][f] = if v.is_finite() { v.clamp(-3.0, 3.0) } else { 0.0 };
                        }
                    }
                    for f in 0..FORMATION_DIM {
                        let v = flat_b_form[f] * decay;
                        brain.b_form[f] = if v.is_finite() { v.clamp(-3.0, 3.0) } else { 0.0 };
                    }
                    return;
                }
            }
        }

        if let Some(fallback) = &self.fallback_brain {
            *brain = fallback.clone();
        }
    }
}

impl Drop for PersistentGpuTrainer {
    fn drop(&mut self) {
        #[cfg(has_cuda)]
        {
            if !self.ctx.is_null() {
                unsafe {
                    cuda_persistent_free(self.ctx);
                }
                self.ctx = std::ptr::null_mut();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cuda_detection_and_training() {
        let is_avail = CudaDeviceManager::is_gpu_available();
        println!("CUDA is available: {}", is_avail);
        let dev_name = CudaDeviceManager::device_name();
        println!("CUDA Device: {}", dev_name);
        assert!(
            is_avail,
            "CUDA should be available on this NVIDIA RTX machine"
        );

        let mut trainer = GpuBatchTrainer::new(4);
        assert!(trainer.use_gpu);

        let mut brain = RlBrain::default();
        let p_count = brain.total_parameters();
        println!("Brain parameters: {}", p_count);
        assert_eq!(p_count, 170005);

        for _ in 0..4 {
            trainer.push(TransitionSample {
                state: vec![0.5; STATE_DIM],
                action: 1,
                formation: 0,
                reward: 10.0,
                next_state: vec![0.6; STATE_DIM],
                done: false,
            });
        }

        assert!(trainer.can_train());
        trainer.train_batch(&mut brain);
        println!("GPU Batch Training executed successfully on {}", dev_name);
    }

    #[test]
    fn test_cuda_persistent_trainer() {
        let is_avail = CudaDeviceManager::is_gpu_available();
        if !is_avail {
            return;
        }

        let mut brain = RlBrain::default();
        let mut trainer = PersistentGpuTrainer::new(&brain, 128);
        assert!(trainer.use_gpu);

        let samples: Vec<TransitionSample> = (0..32)
            .map(|i| TransitionSample {
                state: vec![0.1 * (i as f32); STATE_DIM],
                action: i % ACTION_DIM,
                formation: i % FORMATION_DIM,
                reward: 5.0,
                next_state: vec![0.12 * (i as f32); STATE_DIM],
                done: i % 10 == 0,
            })
            .collect();

        // Train multiple batches persistently in VRAM
        for _ in 0..5 {
            trainer.train_batch(&samples, brain.learning_rate, brain.gamma);
        }

        // Sync weights back to brain
        trainer.sync_to_brain(&mut brain);
        assert!(brain.is_valid(), "Brain must be valid after persistent VRAM training");
        println!("Persistent GPU VRAM Trainer executed successfully!");
    }
}
