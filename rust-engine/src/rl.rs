use serde::{Deserialize, Serialize};

pub const STATE_DIM: usize = 16;
pub const ACTION_DIM: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BotStrategicAction {
    AutoEcoExpand = 0,
    AutoDefenseFortify = 1,
    AutoTechScience = 2,
    AutoTechMagic = 3,
    AutoTechMonster = 4,
    AutoAggressiveRush = 5,
    ManualDefendAndRepair = 6,
    ManualStageAndHarass = 7,
    ManualAllOutAttack = 8,
    ManualRetreatAndRegroup = 9,
}

impl BotStrategicAction {
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::AutoEcoExpand,
            1 => Self::AutoDefenseFortify,
            2 => Self::AutoTechScience,
            3 => Self::AutoTechMagic,
            4 => Self::AutoTechMonster,
            5 => Self::AutoAggressiveRush,
            6 => Self::ManualDefendAndRepair,
            7 => Self::ManualStageAndHarass,
            8 => Self::ManualAllOutAttack,
            _ => Self::ManualRetreatAndRegroup,
        }
    }

    pub fn to_index(&self) -> usize {
        *self as usize
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::AutoEcoExpand => "Auto Eco & Expansion",
            Self::AutoDefenseFortify => "Auto Defense & Fortification",
            Self::AutoTechScience => "Auto Tech: Science Branch",
            Self::AutoTechMagic => "Auto Tech: Magic Branch",
            Self::AutoTechMonster => "Auto Tech: Monster Branch",
            Self::AutoAggressiveRush => "Auto Rush: Combat Units",
            Self::ManualDefendAndRepair => "Direct: Defend & Repair Base",
            Self::ManualStageAndHarass => "Direct: Stage & Harass",
            Self::ManualAllOutAttack => "Direct: All-out Attack Wave",
            Self::ManualRetreatAndRegroup => "Direct: Retreat & Regroup",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RlBrain {
    pub input_size: usize,
    pub hidden_size: usize,
    pub output_size: usize,
    pub w1: Vec<Vec<f32>>, // [input_size][hidden_size]
    pub b1: Vec<f32>,      // [hidden_size]
    pub w2: Vec<Vec<f32>>, // [hidden_size][output_size]
    pub b2: Vec<f32>,      // [output_size]
    #[serde(default = "default_learning_rate")]
    pub learning_rate: f32,
    #[serde(default = "default_gamma")]
    pub gamma: f32,
    #[serde(default = "default_epsilon")]
    pub epsilon: f32,
}

fn default_learning_rate() -> f32 { 0.01 }
fn default_gamma() -> f32 { 0.95 }
fn default_epsilon() -> f32 { 0.05 }

impl Default for RlBrain {
    fn default() -> Self {
        if let Some(loaded) = Self::load_default_weights() {
            loaded
        } else {
            Self::new_random(STATE_DIM, 32, ACTION_DIM)
        }
    }
}

impl RlBrain {
    pub fn new_random(input_size: usize, hidden_size: usize, output_size: usize) -> Self {
        let mut rng = fastrand::Rng::with_seed(42);
        let scale1 = (2.0 / (input_size as f32)).sqrt();
        let scale2 = (2.0 / (hidden_size as f32)).sqrt();

        let mut w1 = Vec::with_capacity(input_size);
        for _ in 0..input_size {
            let mut row = Vec::with_capacity(hidden_size);
            for _ in 0..hidden_size {
                row.push((rng.f32() * 2.0 - 1.0) * scale1);
            }
            w1.push(row);
        }

        let b1 = vec![0.0; hidden_size];

        let mut w2 = Vec::with_capacity(hidden_size);
        for _ in 0..hidden_size {
            let mut row = Vec::with_capacity(output_size);
            for _ in 0..output_size {
                row.push((rng.f32() * 2.0 - 1.0) * scale2);
            }
            w2.push(row);
        }

        let b2 = vec![0.0; output_size];

        Self {
            input_size,
            hidden_size,
            output_size,
            w1,
            b1,
            w2,
            b2,
            learning_rate: 0.01,
            gamma: 0.95,
            epsilon: 0.05,
        }
    }

    pub fn load_default_weights() -> Option<Self> {
        let model_str = include_str!("../assets/rl_model.json");
        serde_json::from_str(model_str).ok()
    }

    pub fn forward(&self, state: &[f32]) -> (Vec<f32>, Vec<f32>) {
        let mut hidden = vec![0.0; self.hidden_size];
        for j in 0..self.hidden_size {
            let mut sum = self.b1[j];
            for i in 0..self.input_size {
                if i < state.len() {
                    sum += state[i] * self.w1[i][j];
                }
            }
            hidden[j] = sum.max(0.0); // ReLU
        }

        let mut outputs = vec![0.0; self.output_size];
        for k in 0..self.output_size {
            let mut sum = self.b2[k];
            for j in 0..self.hidden_size {
                sum += hidden[j] * self.w2[j][k];
            }
            outputs[k] = sum;
        }

        (hidden, outputs)
    }

    pub fn select_action(&self, state: &[f32], explore: bool) -> BotStrategicAction {
        if explore && fastrand::f32() < self.epsilon {
            let random_idx = fastrand::usize(0..self.output_size);
            return BotStrategicAction::from_index(random_idx);
        }

        let (_, q_values) = self.forward(state);
        let mut best_idx = 0;
        let mut best_val = q_values[0];
        for (i, &v) in q_values.iter().enumerate().skip(1) {
            if v > best_val {
                best_val = v;
                best_idx = i;
            }
        }
        BotStrategicAction::from_index(best_idx)
    }

    pub fn train_step(
        &mut self,
        state: &[f32],
        action: BotStrategicAction,
        reward: f32,
        next_state: &[f32],
        done: bool,
    ) {
        let (hidden, q_values) = self.forward(state);
        let action_idx = action.to_index();
        let target = if done {
            reward
        } else {
            let (_, next_q_values) = self.forward(next_state);
            let max_next_q = next_q_values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            reward + self.gamma * max_next_q
        };

        let td_error = target - q_values[action_idx];
        let grad_out = td_error.clamp(-1.0, 1.0);

        // Backprop to w2, b2
        let mut grad_hidden = vec![0.0; self.hidden_size];
        for j in 0..self.hidden_size {
            grad_hidden[j] = self.w2[j][action_idx] * grad_out;
            self.w2[j][action_idx] += self.learning_rate * grad_out * hidden[j];
        }
        self.b2[action_idx] += self.learning_rate * grad_out;

        // Backprop to w1, b1 (ReLU derivative)
        for j in 0..self.hidden_size {
            if hidden[j] > 0.0 {
                let delta_h = grad_hidden[j];
                for i in 0..self.input_size {
                    if i < state.len() {
                        self.w1[i][j] += self.learning_rate * delta_h * state[i];
                    }
                }
                self.b1[j] += self.learning_rate * delta_h;
            }
        }
    }
}

