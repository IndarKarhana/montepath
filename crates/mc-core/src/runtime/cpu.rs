use std::f64::consts::PI;
use std::thread;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EuropeanCallMethod {
    Auto,
    TerminalDistribution,
    StepwisePaths,
}

impl Default for EuropeanCallMethod {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MonteCarloTechnique {
    Standard,
    Antithetic,
    ControlVariate,
}

impl Default for MonteCarloTechnique {
    fn default() -> Self {
        Self::Standard
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct EuropeanCallConfig {
    pub s0: f64,
    pub k: f64,
    pub r: f64,
    pub sigma: f64,
    pub t: f64,
    pub n_paths: usize,
    pub n_steps: usize,
    pub seed: u64,
    pub n_threads: usize,
    pub technique: MonteCarloTechnique,
}

impl Default for EuropeanCallConfig {
    fn default() -> Self {
        Self {
            s0: 100.0,
            k: 100.0,
            r: 0.03,
            sigma: 0.2,
            t: 1.0,
            n_paths: 100_000,
            n_steps: 252,
            seed: 42,
            n_threads: 0,
            technique: MonteCarloTechnique::Standard,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct EuropeanCallResult {
    pub price: f64,
    pub stderr: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EuropeanCallPricer {
    config: EuropeanCallConfig,
    method: EuropeanCallMethod,
}

impl Default for EuropeanCallPricer {
    fn default() -> Self {
        Self::new()
    }
}

impl EuropeanCallPricer {
    pub fn new() -> Self {
        Self {
            config: EuropeanCallConfig::default(),
            method: EuropeanCallMethod::Auto,
        }
    }

    pub fn from_config(config: EuropeanCallConfig) -> Self {
        Self {
            config,
            method: EuropeanCallMethod::Auto,
        }
    }

    pub fn s0(mut self, value: f64) -> Self {
        self.config.s0 = value;
        self
    }

    pub fn strike(mut self, value: f64) -> Self {
        self.config.k = value;
        self
    }

    pub fn rate(mut self, value: f64) -> Self {
        self.config.r = value;
        self
    }

    pub fn volatility(mut self, value: f64) -> Self {
        self.config.sigma = value;
        self
    }

    pub fn maturity(mut self, value: f64) -> Self {
        self.config.t = value;
        self
    }

    pub fn paths(mut self, value: usize) -> Self {
        self.config.n_paths = value;
        self
    }

    pub fn steps(mut self, value: usize) -> Self {
        self.config.n_steps = value;
        self
    }

    pub fn seed(mut self, value: u64) -> Self {
        self.config.seed = value;
        self
    }

    pub fn threads(mut self, value: usize) -> Self {
        self.config.n_threads = value;
        self
    }

    pub fn method(mut self, value: EuropeanCallMethod) -> Self {
        self.method = value;
        self
    }

    pub fn technique(mut self, value: MonteCarloTechnique) -> Self {
        self.config.technique = value;
        self
    }

    pub fn terminal(mut self) -> Self {
        self.method = EuropeanCallMethod::TerminalDistribution;
        self
    }

    pub fn stepwise(mut self) -> Self {
        self.method = EuropeanCallMethod::StepwisePaths;
        self
    }

    pub fn antithetic(mut self) -> Self {
        self.config.technique = MonteCarloTechnique::Antithetic;
        self
    }

    pub fn control_variate(mut self) -> Self {
        self.config.technique = MonteCarloTechnique::ControlVariate;
        self
    }

    pub fn standard(mut self) -> Self {
        self.config.technique = MonteCarloTechnique::Standard;
        self
    }

    pub fn config(&self) -> &EuropeanCallConfig {
        &self.config
    }

    pub fn methodology(&self) -> EuropeanCallMethod {
        self.method
    }

    pub fn price(&self) -> EuropeanCallResult {
        european_call_price_mc_cpu_with_method(&self.config, self.method)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MonteCarloRng {
    state: u64,
    cached_normal: Option<f64>,
}

impl MonteCarloRng {
    pub fn new(seed: u64) -> Self {
        let non_zero_seed = if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        };

        Self {
            state: non_zero_seed,
            cached_normal: None,
        }
    }

    fn next_u64(&mut self) -> u64 {
        // xorshift64* for a small deterministic PRNG with low overhead.
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn next_f64_open01(&mut self) -> f64 {
        // Use top 53 bits to produce a uniform in (0, 1).
        let raw = self.next_u64() >> 11;
        let value = (raw as f64) * (1.0 / ((1u64 << 53) as f64));
        value.max(f64::MIN_POSITIVE)
    }

    pub fn standard_normal(&mut self) -> f64 {
        if let Some(cached) = self.cached_normal.take() {
            return cached;
        }

        // Box-Muller transform. Cache one sample to halve transcendental calls.
        let u1 = self.next_f64_open01();
        let u2 = self.next_f64_open01();
        let radius = (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * PI * u2;
        let z0 = radius * theta.cos();
        let z1 = radius * theta.sin();
        self.cached_normal = Some(z1);
        z0
    }
}

pub fn european_call_price_mc_cpu(cfg: &EuropeanCallConfig) -> EuropeanCallResult {
    european_call_price_mc_cpu_with_method(cfg, EuropeanCallMethod::Auto)
}

pub fn european_call_price_mc_cpu_with_method(
    cfg: &EuropeanCallConfig,
    method: EuropeanCallMethod,
) -> EuropeanCallResult {
    assert!(cfg.n_paths > 0, "n_paths must be > 0");
    assert!(cfg.n_steps > 0, "n_steps must be > 0");

    match method {
        EuropeanCallMethod::Auto | EuropeanCallMethod::TerminalDistribution => {
            european_call_price_mc_cpu_terminal(cfg)
        }
        EuropeanCallMethod::StepwisePaths => european_call_price_mc_cpu_stepwise(cfg),
    }
}

pub fn european_call_price_mc_cpu_terminal(cfg: &EuropeanCallConfig) -> EuropeanCallResult {
    match cfg.technique {
        MonteCarloTechnique::Antithetic => return simulate_terminal_antithetic(cfg),
        MonteCarloTechnique::ControlVariate => return simulate_terminal_control_variate(cfg),
        MonteCarloTechnique::Standard => {}
    }

    // For European calls under GBM, we can sample terminal distribution directly:
    // S_T = S_0 * exp((r - 0.5*sigma^2)T + sigma*sqrt(T)*Z)
    // This is equivalent in distribution to step-by-step simulation and is much faster.
    let drift_t = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * cfg.t;
    let vol_t = cfg.sigma * cfg.t.sqrt();
    let discount = (-cfg.r * cfg.t).exp();

    let thread_count = resolved_thread_count(cfg.n_threads);
    let (payoff_sum, payoff_sq_sum) = if thread_count <= 1 || cfg.n_paths < thread_count * 2_000 {
        simulate_terminal_chunk(
            cfg.seed,
            cfg.n_paths,
            cfg.s0,
            cfg.k,
            drift_t,
            vol_t,
            discount,
            MonteCarloTechnique::Standard,
        )
    } else {
        simulate_terminal_parallel(cfg, thread_count, drift_t, vol_t, discount)
    };

    summarize_payoffs(cfg.n_paths, payoff_sum, payoff_sq_sum)
}

pub fn european_call_price_mc_cpu_stepwise(cfg: &EuropeanCallConfig) -> EuropeanCallResult {
    match cfg.technique {
        MonteCarloTechnique::Antithetic => return simulate_stepwise_antithetic(cfg),
        MonteCarloTechnique::ControlVariate => return simulate_stepwise_control_variate(cfg),
        MonteCarloTechnique::Standard => {}
    }

    let dt = cfg.t / cfg.n_steps as f64;
    let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
    let vol_dt = cfg.sigma * dt.sqrt();
    let discount = (-cfg.r * cfg.t).exp();

    let thread_count = resolved_thread_count(cfg.n_threads);
    let (payoff_sum, payoff_sq_sum) = if thread_count <= 1 || cfg.n_paths < thread_count * 2_000 {
        simulate_stepwise_chunk(
            cfg.seed,
            cfg.n_paths,
            cfg.n_steps,
            cfg.s0,
            cfg.k,
            drift_dt,
            vol_dt,
            discount,
            MonteCarloTechnique::Standard,
        )
    } else {
        simulate_stepwise_parallel(cfg, thread_count, drift_dt, vol_dt, discount)
    };

    summarize_payoffs(cfg.n_paths, payoff_sum, payoff_sq_sum)
}

#[allow(dead_code)]
pub(crate) fn generate_stepwise_standard_normals_f32(
    seed: u64,
    n_paths: usize,
    n_steps: usize,
) -> Vec<f32> {
    let mut rng = MonteCarloRng::new(seed);
    let mut normals = Vec::with_capacity(n_paths.saturating_mul(n_steps));

    for _ in 0..n_paths.saturating_mul(n_steps) {
        normals.push(rng.standard_normal() as f32);
    }

    normals
}

#[allow(dead_code)]
pub(crate) fn generate_stepwise_stateless_normals_f32(
    seed: u64,
    n_paths: usize,
    n_steps: usize,
) -> Vec<f32> {
    let seed_u32 = seed as u32;
    let mut normals = Vec::with_capacity(n_paths.saturating_mul(n_steps));

    for path_idx in 0..n_paths {
        for step_idx in 0..n_steps {
            normals.push(stateless_standard_normal_f32(
                seed_u32,
                path_idx as u32,
                step_idx as u32,
            ));
        }
    }

    normals
}

#[allow(dead_code)]
pub(crate) fn european_call_price_mc_stepwise_from_f32_normals(
    cfg: &EuropeanCallConfig,
    normals: &[f32],
) -> EuropeanCallResult {
    let expected = cfg.n_paths.saturating_mul(cfg.n_steps);
    assert_eq!(
        normals.len(),
        expected,
        "stepwise normal buffer must contain n_paths * n_steps values"
    );

    let log_s0 = cfg.s0.ln() as f32;
    let strike = cfg.k as f32;
    let dt = (cfg.t / cfg.n_steps as f64) as f32;
    let drift_dt = ((cfg.r - 0.5 * cfg.sigma * cfg.sigma) as f32) * dt;
    let vol_dt = (cfg.sigma as f32) * dt.sqrt();
    let discount = ((-cfg.r * cfg.t).exp()) as f32;

    let mut payoff_sum = 0.0f64;
    let mut payoff_sq_sum = 0.0f64;

    for path_idx in 0..cfg.n_paths {
        let mut log_s_t = log_s0;
        let base_offset = path_idx * cfg.n_steps;
        for step_idx in 0..cfg.n_steps {
            let z = normals[base_offset + step_idx];
            log_s_t += drift_dt + vol_dt * z;
        }

        let s_t = log_s_t.exp();
        let payoff = ((s_t - strike).max(0.0) * discount) as f64;
        payoff_sum += payoff;
        payoff_sq_sum += payoff * payoff;
    }

    summarize_payoffs(cfg.n_paths, payoff_sum, payoff_sq_sum)
}

#[allow(dead_code)]
pub(crate) fn stateless_standard_normal_f32(seed: u32, path_idx: u32, step_idx: u32) -> f32 {
    let u1 = stateless_open01_f32(seed, path_idx, step_idx, 0);
    let u2 = stateless_open01_f32(seed, path_idx, step_idx, 1);
    let radius = (-2.0f32 * u1.ln()).sqrt();
    let theta = 2.0f32 * (std::f32::consts::PI) * u2;
    radius * theta.cos()
}

#[allow(dead_code)]
fn stateless_open01_f32(seed: u32, path_idx: u32, step_idx: u32, lane: u32) -> f32 {
    let mixed = seed
        ^ path_idx.wrapping_mul(747_796_405)
        ^ step_idx.wrapping_mul(2_891_336_453)
        ^ lane.wrapping_mul(277_803_737);
    let hashed = hash_u32(mixed);
    (((hashed as f64) + 1.0) / 4_294_967_297.0).max(f32::MIN_POSITIVE as f64) as f32
}

#[allow(dead_code)]
fn hash_u32(mut x: u32) -> u32 {
    x = x.wrapping_add(0x9E37_79B9);
    x ^= x >> 16;
    x = x.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 13;
    x = x.wrapping_mul(0xC2B2_AE35);
    x ^ (x >> 16)
}

fn simulate_terminal_parallel(
    cfg: &EuropeanCallConfig,
    thread_count: usize,
    drift_t: f64,
    vol_t: f64,
    discount: f64,
) -> (f64, f64) {
    let base_chunk = cfg.n_paths / thread_count;
    let remainder = cfg.n_paths % thread_count;

    let mut handles = Vec::with_capacity(thread_count);
    for idx in 0..thread_count {
        let n_paths_chunk = base_chunk + usize::from(idx < remainder);
        let seed = derive_chunk_seed(cfg.seed, idx as u64);
        let s0 = cfg.s0;
        let k = cfg.k;
        let technique = cfg.technique;
        handles.push(thread::spawn(move || {
            simulate_terminal_chunk(
                seed,
                n_paths_chunk,
                s0,
                k,
                drift_t,
                vol_t,
                discount,
                technique,
            )
        }));
    }

    // Join in spawn order so reduction order is deterministic across runs.
    let mut payoff_sum = 0.0;
    let mut payoff_sq_sum = 0.0;
    for handle in handles {
        let (chunk_sum, chunk_sq_sum) = handle
            .join()
            .expect("CPU Monte Carlo worker thread panicked");
        payoff_sum += chunk_sum;
        payoff_sq_sum += chunk_sq_sum;
    }

    (payoff_sum, payoff_sq_sum)
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ControlVariateMoments {
    pub(crate) sample_count: usize,
    pub(crate) payoff_sum: f64,
    pub(crate) payoff_sq_sum: f64,
    pub(crate) control_sum: f64,
    pub(crate) control_sq_sum: f64,
    pub(crate) payoff_control_cross_sum: f64,
}

impl ControlVariateMoments {
    pub(crate) fn record(&mut self, payoff: f64, control: f64) {
        self.sample_count += 1;
        self.payoff_sum += payoff;
        self.payoff_sq_sum += payoff * payoff;
        self.control_sum += control;
        self.control_sq_sum += control * control;
        self.payoff_control_cross_sum += payoff * control;
    }

    pub(crate) fn merge(&mut self, other: Self) {
        self.sample_count += other.sample_count;
        self.payoff_sum += other.payoff_sum;
        self.payoff_sq_sum += other.payoff_sq_sum;
        self.control_sum += other.control_sum;
        self.control_sq_sum += other.control_sq_sum;
        self.payoff_control_cross_sum += other.payoff_control_cross_sum;
    }
}

fn simulate_stepwise_parallel(
    cfg: &EuropeanCallConfig,
    thread_count: usize,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
) -> (f64, f64) {
    let base_chunk = cfg.n_paths / thread_count;
    let remainder = cfg.n_paths % thread_count;

    let mut handles = Vec::with_capacity(thread_count);
    for idx in 0..thread_count {
        let n_paths_chunk = base_chunk + usize::from(idx < remainder);
        let seed = derive_chunk_seed(cfg.seed, idx as u64);
        let s0 = cfg.s0;
        let k = cfg.k;
        let n_steps = cfg.n_steps;
        let technique = cfg.technique;
        handles.push(thread::spawn(move || {
            simulate_stepwise_chunk(
                seed,
                n_paths_chunk,
                n_steps,
                s0,
                k,
                drift_dt,
                vol_dt,
                discount,
                technique,
            )
        }));
    }

    let mut payoff_sum = 0.0;
    let mut payoff_sq_sum = 0.0;
    for handle in handles {
        let (chunk_sum, chunk_sq_sum) = handle
            .join()
            .expect("CPU Monte Carlo worker thread panicked");
        payoff_sum += chunk_sum;
        payoff_sq_sum += chunk_sq_sum;
    }

    (payoff_sum, payoff_sq_sum)
}

fn simulate_terminal_control_variate_parallel(
    cfg: &EuropeanCallConfig,
    thread_count: usize,
    drift_t: f64,
    vol_t: f64,
    discount: f64,
) -> ControlVariateMoments {
    let base_chunk = cfg.n_paths / thread_count;
    let remainder = cfg.n_paths % thread_count;

    let mut handles = Vec::with_capacity(thread_count);
    for idx in 0..thread_count {
        let n_paths_chunk = base_chunk + usize::from(idx < remainder);
        let seed = derive_chunk_seed(cfg.seed, idx as u64);
        let s0 = cfg.s0;
        let k = cfg.k;
        handles.push(thread::spawn(move || {
            simulate_terminal_control_variate_chunk(
                seed,
                n_paths_chunk,
                s0,
                k,
                drift_t,
                vol_t,
                discount,
            )
        }));
    }

    let mut moments = ControlVariateMoments::default();
    for handle in handles {
        let chunk = handle
            .join()
            .expect("CPU Monte Carlo worker thread panicked");
        moments.merge(chunk);
    }

    moments
}

fn simulate_stepwise_control_variate_parallel(
    cfg: &EuropeanCallConfig,
    thread_count: usize,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
) -> ControlVariateMoments {
    let base_chunk = cfg.n_paths / thread_count;
    let remainder = cfg.n_paths % thread_count;

    let mut handles = Vec::with_capacity(thread_count);
    for idx in 0..thread_count {
        let n_paths_chunk = base_chunk + usize::from(idx < remainder);
        let seed = derive_chunk_seed(cfg.seed, idx as u64);
        let s0 = cfg.s0;
        let k = cfg.k;
        let n_steps = cfg.n_steps;
        handles.push(thread::spawn(move || {
            simulate_stepwise_control_variate_chunk(
                seed,
                n_paths_chunk,
                n_steps,
                s0,
                k,
                drift_dt,
                vol_dt,
                discount,
            )
        }));
    }

    let mut moments = ControlVariateMoments::default();
    for handle in handles {
        let chunk = handle
            .join()
            .expect("CPU Monte Carlo worker thread panicked");
        moments.merge(chunk);
    }

    moments
}

fn simulate_terminal_chunk(
    seed: u64,
    n_paths: usize,
    s0: f64,
    k: f64,
    drift_t: f64,
    vol_t: f64,
    discount: f64,
    technique: MonteCarloTechnique,
) -> (f64, f64) {
    let mut rng = MonteCarloRng::new(seed);
    let mut payoff_sum = 0.0;
    let mut payoff_sq_sum = 0.0;

    match technique {
        MonteCarloTechnique::Standard => {
            for _ in 0..n_paths {
                let z = rng.standard_normal();
                let payoff = european_call_payoff_from_shock(s0, k, drift_t, vol_t, z, discount);
                payoff_sum += payoff;
                payoff_sq_sum += payoff * payoff;
            }
        }
        MonteCarloTechnique::Antithetic => {
            let pair_count = n_paths / 2;
            for _ in 0..pair_count {
                let z = rng.standard_normal();
                let payoff_a = european_call_payoff_from_shock(s0, k, drift_t, vol_t, z, discount);
                let payoff_b = european_call_payoff_from_shock(s0, k, drift_t, vol_t, -z, discount);
                payoff_sum += payoff_a + payoff_b;
                payoff_sq_sum += payoff_a * payoff_a + payoff_b * payoff_b;
            }

            if n_paths % 2 != 0 {
                let z = rng.standard_normal();
                let payoff = european_call_payoff_from_shock(s0, k, drift_t, vol_t, z, discount);
                payoff_sum += payoff;
                payoff_sq_sum += payoff * payoff;
            }
        }
        MonteCarloTechnique::ControlVariate => {
            unreachable!("control variate terminal path uses dedicated accumulator kernel");
        }
    }

    (payoff_sum, payoff_sq_sum)
}

fn simulate_stepwise_chunk(
    seed: u64,
    n_paths: usize,
    n_steps: usize,
    s0: f64,
    k: f64,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
    technique: MonteCarloTechnique,
) -> (f64, f64) {
    let mut rng = MonteCarloRng::new(seed);
    let mut payoff_sum = 0.0;
    let mut payoff_sq_sum = 0.0;

    match technique {
        MonteCarloTechnique::Standard => {
            for _ in 0..n_paths {
                let mut log_s_t = s0.ln();
                for _ in 0..n_steps {
                    let z = rng.standard_normal();
                    log_s_t += drift_dt + vol_dt * z;
                }

                let payoff = (log_s_t.exp() - k).max(0.0) * discount;
                payoff_sum += payoff;
                payoff_sq_sum += payoff * payoff;
            }
        }
        MonteCarloTechnique::Antithetic => {
            let pair_count = n_paths / 2;
            for _ in 0..pair_count {
                let mut log_a = s0.ln();
                let mut log_b = s0.ln();
                for _ in 0..n_steps {
                    let z = rng.standard_normal();
                    log_a += drift_dt + vol_dt * z;
                    log_b += drift_dt - vol_dt * z;
                }

                let payoff_a = (log_a.exp() - k).max(0.0) * discount;
                let payoff_b = (log_b.exp() - k).max(0.0) * discount;
                payoff_sum += payoff_a + payoff_b;
                payoff_sq_sum += payoff_a * payoff_a + payoff_b * payoff_b;
            }

            if n_paths % 2 != 0 {
                let mut log_s_t = s0.ln();
                for _ in 0..n_steps {
                    let z = rng.standard_normal();
                    log_s_t += drift_dt + vol_dt * z;
                }

                let payoff = (log_s_t.exp() - k).max(0.0) * discount;
                payoff_sum += payoff;
                payoff_sq_sum += payoff * payoff;
            }
        }
        MonteCarloTechnique::ControlVariate => {
            unreachable!("control variate stepwise path uses dedicated accumulator kernel");
        }
    }

    (payoff_sum, payoff_sq_sum)
}

fn simulate_terminal_control_variate_chunk(
    seed: u64,
    n_paths: usize,
    s0: f64,
    k: f64,
    drift_t: f64,
    vol_t: f64,
    discount: f64,
) -> ControlVariateMoments {
    let mut rng = MonteCarloRng::new(seed);
    let mut moments = ControlVariateMoments::default();

    for _ in 0..n_paths {
        let z = rng.standard_normal();
        let s_t = s0 * (drift_t + vol_t * z).exp();
        let control = discount * s_t;
        let payoff = (s_t - k).max(0.0) * discount;
        moments.record(payoff, control);
    }

    moments
}

fn simulate_stepwise_control_variate_chunk(
    seed: u64,
    n_paths: usize,
    n_steps: usize,
    s0: f64,
    k: f64,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
) -> ControlVariateMoments {
    let mut rng = MonteCarloRng::new(seed);
    let mut moments = ControlVariateMoments::default();

    for _ in 0..n_paths {
        let mut log_s_t = s0.ln();
        for _ in 0..n_steps {
            let z = rng.standard_normal();
            log_s_t += drift_dt + vol_dt * z;
        }

        let s_t = log_s_t.exp();
        let control = discount * s_t;
        let payoff = (s_t - k).max(0.0) * discount;
        moments.record(payoff, control);
    }

    moments
}

pub(crate) fn summarize_payoffs(
    n_paths: usize,
    payoff_sum: f64,
    payoff_sq_sum: f64,
) -> EuropeanCallResult {
    let n = n_paths as f64;
    let price = payoff_sum / n;
    let variance = (payoff_sq_sum / n) - (price * price);
    let stderr = variance.max(0.0).sqrt() / n.sqrt();

    EuropeanCallResult { price, stderr }
}

pub(crate) fn summarize_block_estimates(
    block_count: usize,
    block_sum: f64,
    block_sq_sum: f64,
) -> EuropeanCallResult {
    let n = block_count as f64;
    let price = block_sum / n;
    let variance = (block_sq_sum / n) - (price * price);
    let stderr = variance.max(0.0).sqrt() / n.sqrt();

    EuropeanCallResult { price, stderr }
}

pub(crate) fn summarize_control_variate(
    moments: ControlVariateMoments,
    control_expectation: f64,
) -> EuropeanCallResult {
    let n = moments.sample_count as f64;
    let payoff_mean = moments.payoff_sum / n;
    let control_mean = moments.control_sum / n;
    let control_var = (moments.control_sq_sum / n) - (control_mean * control_mean);

    if control_var <= f64::EPSILON {
        return summarize_payoffs(
            moments.sample_count,
            moments.payoff_sum,
            moments.payoff_sq_sum,
        );
    }

    let payoff_control_cov = (moments.payoff_control_cross_sum / n) - (payoff_mean * control_mean);
    let beta = payoff_control_cov / control_var;
    let adjusted_mean = payoff_mean - beta * (control_mean - control_expectation);
    let adjusted_sq_mean = (moments.payoff_sq_sum / n)
        - (2.0 * beta)
            * ((moments.payoff_control_cross_sum / n) - control_expectation * payoff_mean)
        + (beta * beta)
            * ((moments.control_sq_sum / n) - (2.0 * control_expectation * control_mean)
                + (control_expectation * control_expectation));
    let adjusted_var = (adjusted_sq_mean - (adjusted_mean * adjusted_mean)).max(0.0);
    let stderr = adjusted_var.sqrt() / n.sqrt();

    EuropeanCallResult {
        price: adjusted_mean,
        stderr,
    }
}

fn european_call_payoff_from_shock(
    s0: f64,
    k: f64,
    drift_t: f64,
    vol_t: f64,
    z: f64,
    discount: f64,
) -> f64 {
    let s_t = s0 * (drift_t + vol_t * z).exp();
    (s_t - k).max(0.0) * discount
}

fn simulate_terminal_antithetic(cfg: &EuropeanCallConfig) -> EuropeanCallResult {
    let drift_t = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * cfg.t;
    let vol_t = cfg.sigma * cfg.t.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let pair_count = cfg.n_paths.div_ceil(2);

    let mut rng = MonteCarloRng::new(cfg.seed);
    let mut block_sum = 0.0;
    let mut block_sq_sum = 0.0;

    for _ in 0..pair_count {
        let z = rng.standard_normal();
        let payoff_a = european_call_payoff_from_shock(cfg.s0, cfg.k, drift_t, vol_t, z, discount);
        let payoff_b = european_call_payoff_from_shock(cfg.s0, cfg.k, drift_t, vol_t, -z, discount);
        let block_estimate = 0.5 * (payoff_a + payoff_b);
        block_sum += block_estimate;
        block_sq_sum += block_estimate * block_estimate;
    }

    summarize_block_estimates(pair_count, block_sum, block_sq_sum)
}

fn simulate_stepwise_antithetic(cfg: &EuropeanCallConfig) -> EuropeanCallResult {
    let dt = cfg.t / cfg.n_steps as f64;
    let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
    let vol_dt = cfg.sigma * dt.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let pair_count = cfg.n_paths.div_ceil(2);

    let mut rng = MonteCarloRng::new(cfg.seed);
    let mut block_sum = 0.0;
    let mut block_sq_sum = 0.0;

    for _ in 0..pair_count {
        let mut log_a = cfg.s0.ln();
        let mut log_b = cfg.s0.ln();
        for _ in 0..cfg.n_steps {
            let z = rng.standard_normal();
            log_a += drift_dt + vol_dt * z;
            log_b += drift_dt - vol_dt * z;
        }

        let payoff_a = (log_a.exp() - cfg.k).max(0.0) * discount;
        let payoff_b = (log_b.exp() - cfg.k).max(0.0) * discount;
        let block_estimate = 0.5 * (payoff_a + payoff_b);
        block_sum += block_estimate;
        block_sq_sum += block_estimate * block_estimate;
    }

    summarize_block_estimates(pair_count, block_sum, block_sq_sum)
}

fn simulate_terminal_control_variate(cfg: &EuropeanCallConfig) -> EuropeanCallResult {
    let drift_t = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * cfg.t;
    let vol_t = cfg.sigma * cfg.t.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let thread_count = resolved_thread_count(cfg.n_threads);

    let moments = if thread_count <= 1 || cfg.n_paths < thread_count * 2_000 {
        simulate_terminal_control_variate_chunk(
            cfg.seed,
            cfg.n_paths,
            cfg.s0,
            cfg.k,
            drift_t,
            vol_t,
            discount,
        )
    } else {
        simulate_terminal_control_variate_parallel(cfg, thread_count, drift_t, vol_t, discount)
    };

    summarize_control_variate(moments, cfg.s0)
}

fn simulate_stepwise_control_variate(cfg: &EuropeanCallConfig) -> EuropeanCallResult {
    let dt = cfg.t / cfg.n_steps as f64;
    let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
    let vol_dt = cfg.sigma * dt.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let thread_count = resolved_thread_count(cfg.n_threads);

    let moments = if thread_count <= 1 || cfg.n_paths < thread_count * 2_000 {
        simulate_stepwise_control_variate_chunk(
            cfg.seed,
            cfg.n_paths,
            cfg.n_steps,
            cfg.s0,
            cfg.k,
            drift_dt,
            vol_dt,
            discount,
        )
    } else {
        simulate_stepwise_control_variate_parallel(cfg, thread_count, drift_dt, vol_dt, discount)
    };

    summarize_control_variate(moments, cfg.s0)
}

fn resolved_thread_count(requested_threads: usize) -> usize {
    if requested_threads > 0 {
        return requested_threads;
    }

    thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

fn derive_chunk_seed(base_seed: u64, chunk_index: u64) -> u64 {
    splitmix64(base_seed.wrapping_add(chunk_index.wrapping_mul(0x9E37_79B9_7F4A_7C15)))
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
