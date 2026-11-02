use crate::components::FormationType;
use serde::{Deserialize, Serialize};

pub const STATE_DIM: usize = 64;
pub const ACTION_DIM: usize = 15;
pub const FORMATION_DIM: usize = 6;
pub const DEFAULT_HIDDEN_SIZE: usize = 512;
pub const DEFAULT_HIDDEN2_SIZE: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiGameMode {
    Duel1v1,
    Team2v2,
    Team3v3,
    Team4v4,
    ThreeWay2v2v2,
    FreeForAll8,
}

impl AiGameMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Duel1v1 => "1v1 Duel",
            Self::Team2v2 => "2v2 Battle",
            Self::Team3v3 => "3v3 War",
            Self::Team4v4 => "4v4 Grand War",
            Self::ThreeWay2v2v2 => "2v2v2 3-Way",
            Self::FreeForAll8 => "8-FFA Battle Royale",
        }
    }

    pub fn model_filename(&self) -> &'static str {
        match self {
            Self::Duel1v1 => "rl_model_1v1.json",
            Self::Team2v2 => "rl_model_2v2.json",
            Self::Team3v3 => "rl_model_3v3.json",
            Self::Team4v4 => "rl_model_4v4.json",
            Self::ThreeWay2v2v2 => "rl_model_2v2v2.json",
            Self::FreeForAll8 => "rl_model_ffa8.json",
        }
    }

    pub fn detect_from_slots(slots: &[crate::components::PlayerSlotConfig; 8]) -> Self {
        let active_slots: Vec<&crate::components::PlayerSlotConfig> =
            slots.iter().filter(|s| s.active).collect();
        let num_active = active_slots.len();

        let mut factions = std::collections::HashSet::new();
        for s in &active_slots {
            factions.insert(s.faction);
        }
        let num_factions = factions.len();

        if num_factions >= 6 && num_active >= 6 {
            Self::FreeForAll8
        } else if num_factions == 3 {
            Self::ThreeWay2v2v2
        } else {
            match num_active {
                0..=2 => Self::Duel1v1,
                3..=4 => Self::Team2v2,
                5..=6 => Self::Team3v3,
                _ => Self::Team4v4,
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BotStrategicAction {
    ExpandEco = 0,
    BuildInfrastructure = 1,
    BuildDefenseGrid = 2,
    TechScienceBranch = 3,
    TechMagicBranch = 4,
    TechMonsterBranch = 5,
    MilitarySwarmRush = 6,
    DefendAndCounter = 7,
    HarassEconomy = 8,
    AllOutAssault = 9,
    RegroupAndConsolidate = 10,
    TechLogisticsBranch = 11,
    SiegeArtilleryPush = 12,
    CoordinateAllySupport = 13,
    FocusWeakestEnemy = 14,
}

impl BotStrategicAction {
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::ExpandEco,
            1 => Self::BuildInfrastructure,
            2 => Self::BuildDefenseGrid,
            3 => Self::TechScienceBranch,
            4 => Self::TechMagicBranch,
            5 => Self::TechMonsterBranch,
            6 => Self::MilitarySwarmRush,
            7 => Self::DefendAndCounter,
            8 => Self::HarassEconomy,
            9 => Self::AllOutAssault,
            10 => Self::RegroupAndConsolidate,
            11 => Self::TechLogisticsBranch,
            12 => Self::SiegeArtilleryPush,
            13 => Self::CoordinateAllySupport,
            _ => Self::FocusWeakestEnemy,
        }
    }

    pub fn to_index(&self) -> usize {
        *self as usize
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::ExpandEco => "Expand Base & Eco",
            Self::BuildInfrastructure => "Build Power & Supply Depot",
            Self::BuildDefenseGrid => "Fortify Turret Defense Grid",
            Self::TechScienceBranch => "Tech & Produce: Science Branch",
            Self::TechMagicBranch => "Tech & Produce: Magic Branch",
            Self::TechMonsterBranch => "Tech & Produce: Monster Branch",
            Self::MilitarySwarmRush => "Mass Production: Swarm Rush",
            Self::DefendAndCounter => "Tactical: Defend & Counter-Attack",
            Self::HarassEconomy => "Tactical: Harass Player Minerals",
            Self::AllOutAssault => "Tactical: All-out Base Assault",
            Self::RegroupAndConsolidate => "Tactical: Regroup & Consolidate",
            Self::TechLogisticsBranch => "Tech & Produce: Logistics Branch",
            Self::SiegeArtilleryPush => "Tactical: Siege Artillery Push",
            Self::CoordinateAllySupport => "Tactical: Coordinate Ally Support",
            Self::FocusWeakestEnemy => "Tactical: Focus Weakest Enemy",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RlBrain {
    pub input_size: usize,
    pub hidden_size: usize,
    #[serde(default = "default_hidden2_size")]
    pub hidden2_size: usize,
    pub output_size: usize,

    pub w1: Vec<Vec<f32>>, // [input_size][hidden_size]
    pub b1: Vec<f32>,      // [hidden_size]

    #[serde(default)]
    pub w_h2: Vec<Vec<f32>>, // [hidden_size][hidden2_size]
    #[serde(default)]
    pub b_h2: Vec<f32>, // [hidden2_size]

    pub w2: Vec<Vec<f32>>, // [hidden2_size][output_size]
    pub b2: Vec<f32>,      // [output_size]

    #[serde(default)]
    pub w_form: Vec<Vec<f32>>, // [hidden2_size][FORMATION_DIM]
    #[serde(default)]
    pub b_form: Vec<f32>, // [FORMATION_DIM]

    #[serde(default = "default_learning_rate")]
    pub learning_rate: f32,
    #[serde(default = "default_gamma")]
    pub gamma: f32,
    #[serde(default = "default_epsilon")]
    pub epsilon: f32,
}

fn default_hidden2_size() -> usize {
    DEFAULT_HIDDEN2_SIZE
}
fn default_learning_rate() -> f32 {
    0.015
}
fn default_gamma() -> f32 {
    0.95
}
fn default_epsilon() -> f32 {
    0.05
}

impl Default for RlBrain {
    fn default() -> Self {
        if let Some(loaded) = Self::load_default_weights() {
            return loaded;
        }
        Self::new_random(
            STATE_DIM,
            DEFAULT_HIDDEN_SIZE,
            DEFAULT_HIDDEN2_SIZE,
            ACTION_DIM,
        )
    }
}

impl RlBrain {
    pub fn new_random(
        input_size: usize,
        hidden_size: usize,
        hidden2_size: usize,
        output_size: usize,
    ) -> Self {
        let mut rng = fastrand::Rng::with_seed(1337);
        let scale1 = (2.0 / (input_size as f32)).sqrt();
        let scale_h2 = (2.0 / (hidden_size as f32)).sqrt();
        let scale2 = (2.0 / (hidden2_size as f32)).sqrt();

        // Layer 1: input -> hidden
        let mut w1 = Vec::with_capacity(input_size);
        for _ in 0..input_size {
            let mut row = Vec::with_capacity(hidden_size);
            for _ in 0..hidden_size {
                row.push((rng.f32() * 2.0 - 1.0) * scale1);
            }
            w1.push(row);
        }
        let b1 = vec![0.0; hidden_size];

        // Layer 2: hidden -> hidden2
        let mut w_h2 = Vec::with_capacity(hidden_size);
        for _ in 0..hidden_size {
            let mut row = Vec::with_capacity(hidden2_size);
            for _ in 0..hidden2_size {
                row.push((rng.f32() * 2.0 - 1.0) * scale_h2);
            }
            w_h2.push(row);
        }
        let b_h2 = vec![0.0; hidden2_size];

        // Action Head: hidden2 -> action
        let mut w2 = Vec::with_capacity(hidden2_size);
        for _ in 0..hidden2_size {
            let mut row = Vec::with_capacity(output_size);
            for _ in 0..output_size {
                row.push((rng.f32() * 2.0 - 1.0) * scale2);
            }
            w2.push(row);
        }
        let b2 = vec![0.0; output_size];

        // Formation Head: hidden2 -> formation
        let scale_form = (2.0 / (hidden2_size as f32)).sqrt();
        let mut w_form = Vec::with_capacity(hidden2_size);
        for _ in 0..hidden2_size {
            let mut row = Vec::with_capacity(FORMATION_DIM);
            for _ in 0..FORMATION_DIM {
                row.push((rng.f32() * 2.0 - 1.0) * scale_form);
            }
            w_form.push(row);
        }
        let b_form = vec![0.0; FORMATION_DIM];

        Self {
            input_size,
            hidden_size,
            hidden2_size,
            output_size,
            w1,
            b1,
            w_h2,
            b_h2,
            w2,
            b2,
            w_form,
            b_form,
            learning_rate: 0.015,
            gamma: 0.95,
            epsilon: 0.05,
        }
    }

    /// Tổng số tham số (Weights + Biases) của mạng
    pub fn total_parameters(&self) -> usize {
        let mut count = 0;
        for row in &self.w1 {
            count += row.len();
        }
        count += self.b1.len();

        for row in &self.w_h2 {
            count += row.len();
        }
        count += self.b_h2.len();

        for row in &self.w2 {
            count += row.len();
        }
        count += self.b2.len();

        for row in &self.w_form {
            count += row.len();
        }
        count += self.b_form.len();

        count
    }

    /// Đảm bảo kích thước ma trận phù hợp với STATE_DIM (64), DEFAULT_HIDDEN_SIZE (512), DEFAULT_HIDDEN2_SIZE (256)
    pub fn ensure_dimensions(self) -> Self {
        if self.input_size == STATE_DIM
            && self.output_size == ACTION_DIM
            && self.hidden_size == DEFAULT_HIDDEN_SIZE
            && self.hidden2_size == DEFAULT_HIDDEN2_SIZE
            && self.w1.len() == STATE_DIM
            && self.w1[0].len() == DEFAULT_HIDDEN_SIZE
            && self.w_h2.len() == DEFAULT_HIDDEN_SIZE
            && self.w_h2[0].len() == DEFAULT_HIDDEN2_SIZE
            && self.w2.len() == DEFAULT_HIDDEN2_SIZE
            && self.w2[0].len() == ACTION_DIM
            && self.w_form.len() == DEFAULT_HIDDEN2_SIZE
            && self.w_form[0].len() == FORMATION_DIM
        {
            return self;
        }

        // Tạo bộ khung mới chuẩn kích thước
        let mut target = Self::new_random(
            STATE_DIM,
            DEFAULT_HIDDEN_SIZE,
            DEFAULT_HIDDEN2_SIZE,
            ACTION_DIM,
        );
        target.learning_rate = self.learning_rate;
        target.gamma = self.gamma;
        target.epsilon = self.epsilon;

        // Sao chép trọng số cũ sang ma trận mới (nếu có)
        let copy_inputs = self.input_size.min(STATE_DIM);
        let copy_hidden = self.hidden_size.min(DEFAULT_HIDDEN_SIZE);
        let copy_hidden2 = if self.hidden2_size > 0 {
            self.hidden2_size.min(DEFAULT_HIDDEN2_SIZE)
        } else {
            0
        };
        let _copy_actions = self.output_size.min(ACTION_DIM);

        for i in 0..copy_inputs {
            for j in 0..copy_hidden {
                if i < self.w1.len() && j < self.w1[i].len() {
                    target.w1[i][j] = self.w1[i][j];
                }
            }
        }
        for j in 0..copy_hidden {
            if j < self.b1.len() {
                target.b1[j] = self.b1[j];
            }
        }

        if !self.w_h2.is_empty() {
            for j in 0..copy_hidden {
                for k in 0..copy_hidden2 {
                    if j < self.w_h2.len() && k < self.w_h2[j].len() {
                        target.w_h2[j][k] = self.w_h2[j][k];
                    }
                }
            }
            for k in 0..copy_hidden2 {
                if k < self.b_h2.len() {
                    target.b_h2[k] = self.b_h2[k];
                }
            }
        }

        target
    }

    pub fn load_for_mode(mode: AiGameMode) -> Self {
        let model_str = match mode {
            AiGameMode::Duel1v1 => include_str!("../assets/rl_model_1v1.json"),
            AiGameMode::Team2v2 => include_str!("../assets/rl_model_2v2.json"),
            AiGameMode::Team3v3 => include_str!("../assets/rl_model_3v3.json"),
            AiGameMode::Team4v4 => include_str!("../assets/rl_model_4v4.json"),
            AiGameMode::ThreeWay2v2v2 => include_str!("../assets/rl_model_2v2v2.json"),
            AiGameMode::FreeForAll8 => include_str!("../assets/rl_model_ffa8.json"),
        };

        if let Ok(brain) = serde_json::from_str::<Self>(model_str) {
            brain.ensure_dimensions()
        } else {
            Self::default()
        }
    }

    pub fn load_default_weights() -> Option<Self> {
        let model_str = include_str!("../assets/rl_model.json");
        if let Ok(brain) = serde_json::from_str::<Self>(model_str) {
            Some(brain.ensure_dimensions())
        } else {
            None
        }
    }

    /// Forward pass qua 2 tầng ẩn: Layer 1 (512) -> Layer 2 (256) -> Heads
    pub fn forward_all(&self, state: &[f32]) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
        // Layer 1: state [input_size] -> hidden1 [hidden_size]
        let mut h1 = vec![0.0; self.hidden_size];
        for (j, h1_val) in h1.iter_mut().enumerate() {
            let mut sum = self.b1[j];
            for i in 0..self.input_size {
                if i < state.len() {
                    sum += state[i] * self.w1[i][j];
                }
            }
            // LeakyReLU: max(0.01 * sum, sum)
            *h1_val = if sum >= 0.0 { sum } else { 0.01 * sum };
        }

        // Layer 2: hidden1 [hidden_size] -> hidden2 [hidden2_size]
        let mut h2 = vec![0.0; self.hidden2_size];
        for (k, h2_val) in h2.iter_mut().enumerate() {
            let mut sum = self.b_h2[k];
            for (j, &h1_val) in h1.iter().enumerate() {
                if j < self.w_h2.len() && k < self.w_h2[j].len() {
                    sum += h1_val * self.w_h2[j][k];
                }
            }
            *h2_val = if sum >= 0.0 { sum } else { 0.01 * sum };
        }

        // Action Head: hidden2 [hidden2_size] -> action_q [output_size]
        let mut action_q = vec![0.0; self.output_size];
        for (o, action_val) in action_q.iter_mut().enumerate() {
            let mut sum = self.b2[o];
            for (k, &h2_val) in h2.iter().enumerate() {
                if k < self.w2.len() && o < self.w2[k].len() {
                    sum += h2_val * self.w2[k][o];
                }
            }
            *action_val = sum;
        }

        // Formation Head: hidden2 [hidden2_size] -> form_q [FORMATION_DIM]
        let mut form_q = vec![0.0; FORMATION_DIM];
        for (f, form_val) in form_q.iter_mut().enumerate() {
            let mut sum = self.b_form[f];
            for (k, &h2_val) in h2.iter().enumerate() {
                if k < self.w_form.len() && f < self.w_form[k].len() {
                    sum += h2_val * self.w_form[k][f];
                }
            }
            *form_val = sum;
        }

        (h1, h2, action_q, form_q)
    }

    pub fn forward(&self, state: &[f32]) -> (Vec<f32>, Vec<f32>) {
        let (_, h2, action_q, _) = self.forward_all(state);
        (h2, action_q)
    }

    pub fn select_formation(&self, state: &[f32], explore: bool) -> FormationType {
        if explore && fastrand::f32() < self.epsilon {
            let random_idx = fastrand::usize(0..FORMATION_DIM);
            return FormationType::from_index(random_idx);
        }

        let (_, _, _, form_q) = self.forward_all(state);
        let mut best_idx = 0;
        let mut best_val = form_q[0];
        for (i, &v) in form_q.iter().enumerate().skip(1) {
            if v > best_val {
                best_val = v;
                best_idx = i;
            }
        }
        FormationType::from_index(best_idx)
    }

    pub fn select_action(&self, state: &[f32], explore: bool) -> BotStrategicAction {
        if explore && fastrand::f32() < self.epsilon {
            let random_idx = fastrand::usize(0..self.output_size);
            return BotStrategicAction::from_index(random_idx);
        }

        let (_, _, q_values, _) = self.forward_all(state);
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
        self.train_step_with_formation(
            state,
            action,
            FormationType::Line,
            reward,
            next_state,
            done,
        );
    }

    pub fn train_step_with_formation(
        &mut self,
        state: &[f32],
        action: BotStrategicAction,
        formation: FormationType,
        reward: f32,
        next_state: &[f32],
        done: bool,
    ) {
        let (h1, h2, q_values, form_q) = self.forward_all(state);
        let action_idx = action.to_index();
        let form_idx = formation.to_index();

        let (target_act, target_form) = if done {
            (reward, reward)
        } else {
            let (_, _, next_q_values, next_form_q) = self.forward_all(next_state);
            let max_next_q = next_q_values
                .iter()
                .copied()
                .fold(f32::NEG_INFINITY, f32::max);
            let max_next_form = next_form_q
                .iter()
                .copied()
                .fold(f32::NEG_INFINITY, f32::max);
            (
                reward + self.gamma * max_next_q,
                reward + self.gamma * max_next_form,
            )
        };

        let td_error_act = target_act - q_values[action_idx];
        let grad_out_act = td_error_act.clamp(-1.0, 1.0);

        let td_error_form = target_form - form_q[form_idx];
        let grad_out_form = td_error_form.clamp(-1.0, 1.0);

        // 1. Backprop từ Heads vào Layer 2 (h2)
        let mut grad_h2 = vec![0.0; self.hidden2_size];
        for k in 0..self.hidden2_size {
            let g_act = self.w2[k][action_idx] * grad_out_act;
            self.w2[k][action_idx] += self.learning_rate * grad_out_act * h2[k];

            let g_form = self.w_form[k][form_idx] * grad_out_form;
            self.w_form[k][form_idx] += self.learning_rate * grad_out_form * h2[k];

            grad_h2[k] = g_act + g_form * 0.7;
        }
        self.b2[action_idx] += self.learning_rate * grad_out_act;
        self.b_form[form_idx] += self.learning_rate * grad_out_form;

        // 2. Backprop từ Layer 2 (h2) qua LeakyReLU vào Layer 1 (h1)
        let mut grad_h1 = vec![0.0; self.hidden_size];
        for k in 0..self.hidden2_size {
            let act_derivative = if h2[k] >= 0.0 { 1.0 } else { 0.01 };
            let delta_k = grad_h2[k] * act_derivative;

            for j in 0..self.hidden_size {
                grad_h1[j] += self.w_h2[j][k] * delta_k;
                self.w_h2[j][k] += self.learning_rate * delta_k * h1[j];
            }
            self.b_h2[k] += self.learning_rate * delta_k;
        }

        // 3. Backprop từ Layer 1 (h1) qua LeakyReLU vào Input (w1, b1)
        for j in 0..self.hidden_size {
            let act_derivative = if h1[j] >= 0.0 { 1.0 } else { 0.01 };
            let delta_j = grad_h1[j] * act_derivative;

            for i in 0..self.input_size {
                if i < state.len() {
                    self.w1[i][j] += self.learning_rate * delta_j * state[i];
                }
            }
            self.b1[j] += self.learning_rate * delta_j;
        }
    }
}
