use crate::components::FormationType;
use serde::{Deserialize, Serialize};

pub const STATE_DIM: usize = 40;
pub const ACTION_DIM: usize = 15;
pub const FORMATION_DIM: usize = 6;
pub const DEFAULT_HIDDEN_SIZE: usize = 96;

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
    pub output_size: usize,
    pub w1: Vec<Vec<f32>>, // [input_size][hidden_size]
    pub b1: Vec<f32>,      // [hidden_size]
    pub w2: Vec<Vec<f32>>, // [hidden_size][output_size]
    pub b2: Vec<f32>,      // [output_size]
    #[serde(default)]
    pub w_form: Vec<Vec<f32>>, // [hidden_size][FORMATION_DIM]
    #[serde(default)]
    pub b_form: Vec<f32>,      // [FORMATION_DIM]
    #[serde(default = "default_learning_rate")]
    pub learning_rate: f32,
    #[serde(default = "default_gamma")]
    pub gamma: f32,
    #[serde(default = "default_epsilon")]
    pub epsilon: f32,
}

fn default_learning_rate() -> f32 { 0.015 }
fn default_gamma() -> f32 { 0.95 }
fn default_epsilon() -> f32 { 0.05 }

impl Default for RlBrain {
    fn default() -> Self {
        if let Some(loaded) = Self::load_default_weights() {
            return loaded;
        }
        Self::new_random(STATE_DIM, DEFAULT_HIDDEN_SIZE, ACTION_DIM)
    }
}

impl RlBrain {
    pub fn new_random(input_size: usize, hidden_size: usize, output_size: usize) -> Self {
        let mut rng = fastrand::Rng::with_seed(1337);
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

        let scale_form = (2.0 / (hidden_size as f32)).sqrt();
        let mut w_form = Vec::with_capacity(hidden_size);
        for _ in 0..hidden_size {
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
            output_size,
            w1,
            b1,
            w2,
            b2,
            w_form,
            b_form,
            learning_rate: 0.015,
            gamma: 0.95,
            epsilon: 0.05,
        }
    }

    /// Đảm bảo kích thước ma trận phù hợp với STATE_DIM (40) và ACTION_DIM (15)
    pub fn ensure_dimensions(self) -> Self {
        if self.input_size == STATE_DIM
            && self.output_size == ACTION_DIM
            && self.hidden_size == DEFAULT_HIDDEN_SIZE
            && self.w1.len() == STATE_DIM
            && self.w2.len() == DEFAULT_HIDDEN_SIZE
            && self.w2[0].len() == ACTION_DIM
            && self.w_form.len() == DEFAULT_HIDDEN_SIZE
            && self.w_form[0].len() == FORMATION_DIM
        {
            return self;
        }

        // Tạo bộ khung mới chuẩn kích thước
        let mut target = Self::new_random(STATE_DIM, DEFAULT_HIDDEN_SIZE, ACTION_DIM);
        target.learning_rate = self.learning_rate;
        target.gamma = self.gamma;
        target.epsilon = self.epsilon;

        // Sao chép trọng số cũ sang ma trận mới
        let copy_inputs = self.input_size.min(STATE_DIM);
        let copy_hidden = self.hidden_size.min(DEFAULT_HIDDEN_SIZE);
        let copy_actions = self.output_size.min(ACTION_DIM);

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
            for k in 0..copy_actions {
                if j < self.w2.len() && k < self.w2[j].len() {
                    target.w2[j][k] = self.w2[j][k];
                }
            }
            for f in 0..FORMATION_DIM {
                if j < self.w_form.len() && f < self.w_form[j].len() {
                    target.w_form[j][f] = self.w_form[j][f];
                }
            }
        }
        for k in 0..copy_actions {
            if k < self.b2.len() {
                target.b2[k] = self.b2[k];
            }
        }
        for f in 0..FORMATION_DIM {
            if f < self.b_form.len() {
                target.b_form[f] = self.b_form[f];
            }
        }

        target
    }

    pub fn load_for_mode(mode: AiGameMode) -> Self {
        // Cố gắng nạp đúng model theo mode từ assets
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

    pub fn forward(&self, state: &[f32]) -> (Vec<f32>, Vec<f32>) {
        let mut hidden = vec![0.0; self.hidden_size];
        for j in 0..self.hidden_size {
            let mut sum = self.b1[j];
            for i in 0..self.input_size {
                if i < state.len() {
                    sum += state[i] * self.w1[i][j];
                }
            }
            hidden[j] = sum.max(0.0); // LeakyReLU / ReLU
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

    pub fn forward_all(&self, state: &[f32]) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        let (hidden, action_q) = self.forward(state);

        let mut form_q = vec![0.0; FORMATION_DIM];
        if !self.w_form.is_empty() && !self.b_form.is_empty() {
            for k in 0..FORMATION_DIM {
                let mut sum = self.b_form[k];
                for j in 0..self.hidden_size {
                    if j < self.w_form.len() && k < self.w_form[j].len() {
                        sum += hidden[j] * self.w_form[j][k];
                    }
                }
                form_q[k] = sum;
            }
        }

        (hidden, action_q, form_q)
    }

    pub fn select_formation(&self, state: &[f32], explore: bool) -> FormationType {
        if explore && fastrand::f32() < self.epsilon {
            let random_idx = fastrand::usize(0..FORMATION_DIM);
            return FormationType::from_index(random_idx);
        }

        let (_, _, form_q) = self.forward_all(state);
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
        self.train_step_with_formation(state, action, FormationType::Line, reward, next_state, done);
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
        let (hidden, q_values, form_q) = self.forward_all(state);
        let action_idx = action.to_index();
        let form_idx = formation.to_index();

        let (target_act, target_form) = if done {
            (reward, reward)
        } else {
            let (_, next_q_values, next_form_q) = self.forward_all(next_state);
            let max_next_q = next_q_values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let max_next_form = next_form_q.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            (reward + self.gamma * max_next_q, reward + self.gamma * max_next_form)
        };

        let td_error_act = target_act - q_values[action_idx];
        let grad_out_act = td_error_act.clamp(-1.0, 1.0);

        let td_error_form = target_form - form_q[form_idx];
        let grad_out_form = td_error_form.clamp(-1.0, 1.0);

        // Backprop w2, b2 (Action Head) & w_form, b_form (Formation Head)
        let mut grad_hidden = vec![0.0; self.hidden_size];
        for j in 0..self.hidden_size {
            let g_act = self.w2[j][action_idx] * grad_out_act;
            self.w2[j][action_idx] += self.learning_rate * grad_out_act * hidden[j];

            let g_form = if !self.w_form.is_empty() {
                let g = self.w_form[j][form_idx] * grad_out_form;
                self.w_form[j][form_idx] += self.learning_rate * grad_out_form * hidden[j];
                g
            } else {
                0.0
            };

            grad_hidden[j] = g_act + g_form * 0.7; // Cân bằng gradient
        }
        self.b2[action_idx] += self.learning_rate * grad_out_act;
        if !self.b_form.is_empty() {
            self.b_form[form_idx] += self.learning_rate * grad_out_form;
        }

        // Backprop w1, b1 (ReLU derivative)
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
