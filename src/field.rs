use std::f32::consts::{PI, TAU};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Spectrum {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl Spectrum {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    pub const fn new(red: f32, green: f32, blue: f32) -> Self {
        Self { red, green, blue }
    }

    pub fn total(self) -> f64 {
        (self.red + self.green + self.blue) as f64
    }

    pub fn peak(self) -> f32 {
        self.red.max(self.green).max(self.blue)
    }

    fn magnitude(self) -> f32 {
        self.red.abs() + self.green.abs() + self.blue.abs()
    }

    fn map(self, f: impl Fn(f32) -> f32) -> Self {
        Self::new(f(self.red), f(self.green), f(self.blue))
    }

    fn zip(self, other: Self, f: impl Fn(f32, f32) -> f32) -> Self {
        Self::new(
            f(self.red, other.red),
            f(self.green, other.green),
            f(self.blue, other.blue),
        )
    }

    fn clamp_nonnegative(self) -> Self {
        self.map(|value| value.max(0.0))
    }
}

impl std::ops::Add for Spectrum {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        self.zip(rhs, |a, b| a + b)
    }
}

impl std::ops::AddAssign for Spectrum {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::Sub for Spectrum {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self.zip(rhs, |a, b| a - b)
    }
}

impl std::ops::SubAssign for Spectrum {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl std::ops::Mul<f32> for Spectrum {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        self.map(|value| value * rhs)
    }
}

impl std::ops::Mul for Spectrum {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        self.zip(rhs, |a, b| a * b)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub width: usize,
    pub height: usize,
    pub seed: u64,
    pub diffusion: f32,
    pub gradient: f32,
    pub formation: f32,
    pub erosion: f32,
    pub radiation: f32,
    pub dissipation: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: 192,
            height: 112,
            seed: 0xA701_5EED,
            diffusion: 0.105,
            gradient: 0.16,
            formation: 0.022,
            erosion: 0.0018,
            radiation: 0.19,
            dissipation: 0.0012,
        }
    }
}

impl Config {
    pub fn validate(self) -> Result<Self, String> {
        if self.width < 24 || self.height < 24 {
            return Err("the field must be at least 24 by 24 sites".into());
        }
        if self.width > 2048 || self.height > 2048 {
            return Err("the field cannot exceed 2048 by 2048 sites".into());
        }
        for (name, value) in [
            ("diffusion", self.diffusion),
            ("gradient", self.gradient),
            ("formation", self.formation),
            ("erosion", self.erosion),
            ("radiation", self.radiation),
            ("dissipation", self.dissipation),
        ] {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(format!("{name} must be finite and between zero and one"));
            }
        }
        if self.diffusion > 0.20 {
            return Err("diffusion above 0.20 is unstable for this local law".into());
        }
        if self.diffusion * 4.0 + self.gradient > 0.95 {
            return Err("combined transport would move too much energy in one moment".into());
        }
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Site {
    energy: Spectrum,
    trace: Spectrum,
    coupling: Spectrum,
    permeability: f32,
    activity: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Measurements {
    pub age: u64,
    pub introduced: f64,
    pub resident: f64,
    pub radiated: f64,
    pub dissipated: f64,
    pub accounting_error: f64,
    pub mean_permeability: f64,
    pub organized_fraction: f64,
    pub luminous_sites: usize,
}

pub struct World {
    config: Config,
    sites: Vec<Site>,
    age: u64,
    introduced: f64,
    radiated: f64,
    dissipated: f64,
}

impl World {
    pub fn new(config: Config) -> Result<Self, String> {
        let config = config.validate()?;
        let mut random = SplitMix64(config.seed);
        let mut sites = Vec::with_capacity(config.width * config.height);
        for y in 0..config.height {
            for x in 0..config.width {
                sites.push(Site {
                    permeability: 0.055 + random.unit_f32() * 0.025,
                    coupling: visible_coupling(x, y, config.width, config.height),
                    ..Site::default()
                });
            }
        }
        Ok(Self {
            config,
            sites,
            age: 0,
            introduced: 0.0,
            radiated: 0.0,
            dissipated: 0.0,
        })
    }

    pub fn width(&self) -> usize {
        self.config.width
    }

    pub fn height(&self) -> usize {
        self.config.height
    }

    pub fn age(&self) -> u64 {
        self.age
    }

    /// Advances the field one moment. The loop order implements simultaneous
    /// local change; it is not an ordering visible within the substrate.
    pub fn evolve(&mut self) {
        self.stimulate_boundary();
        let count = self.sites.len();
        let mut change = vec![Spectrum::ZERO; count];
        let mut local_flow = vec![0.0_f32; count];

        for y in 0..self.config.height {
            for x in 0..self.config.width {
                let here = self.index(x, y);
                if x + 1 < self.config.width {
                    let there = self.index(x + 1, y);
                    self.exchange(here, there, true, &mut change, &mut local_flow);
                }
                if y + 1 < self.config.height {
                    let there = self.index(x, y + 1);
                    self.exchange(here, there, false, &mut change, &mut local_flow);
                }
            }
        }

        for (index, site) in self.sites.iter_mut().enumerate() {
            site.energy = (site.energy + change[index]).clamp_nonnegative();
            let emitted = (site.energy * site.coupling) * self.config.radiation;
            site.energy -= emitted;
            site.trace += emitted;
            self.radiated += emitted.total();

            let lost = site.energy * self.config.dissipation;
            site.energy -= lost;
            self.dissipated += lost.total();

            let activity = (local_flow[index] * 2.5).min(1.0);
            site.activity = site.activity * 0.94 + activity * 0.06;
            let growth = self.config.formation * site.activity * (1.0 - site.permeability);
            let decay = self.config.erosion * (site.permeability - 0.05).max(0.0);
            site.permeability = (site.permeability + growth - decay).clamp(0.025, 1.0);
        }
        self.age += 1;
    }

    pub fn evolve_for(&mut self, moments: u64) {
        for _ in 0..moments {
            self.evolve();
        }
    }

    pub fn measurements(&self) -> Measurements {
        let resident = self
            .sites
            .iter()
            .map(|site| site.energy.total())
            .sum::<f64>();
        let mean_permeability = self
            .sites
            .iter()
            .map(|site| site.permeability as f64)
            .sum::<f64>()
            / self.sites.len() as f64;
        let organized = self
            .sites
            .iter()
            .filter(|site| site.permeability > 0.14)
            .count();
        let luminous_sites = self
            .sites
            .iter()
            .filter(|site| site.trace.peak() > 0.01)
            .count();
        let accounted = resident + self.radiated + self.dissipated;
        Measurements {
            age: self.age,
            introduced: self.introduced,
            resident,
            radiated: self.radiated,
            dissipated: self.dissipated,
            accounting_error: self.introduced - accounted,
            mean_permeability,
            organized_fraction: organized as f64 / self.sites.len() as f64,
            luminous_sites,
        }
    }

    pub(crate) fn rgb8(&self) -> Vec<[u8; 3]> {
        let peak_trace = self
            .sites
            .iter()
            .map(|site| site.trace.peak())
            .fold(0.000_001_f32, f32::max);
        let exposure = 3.2 / peak_trace.max(0.1).ln_1p();
        self.sites
            .iter()
            .map(|site| {
                let glow = site.trace.map(|value| (value.ln_1p() * exposure).min(1.0));
                let live = site.energy.map(|value| (value * 0.30).min(1.0));
                let structure = ((site.permeability - 0.05) * 0.45).clamp(0.0, 0.28);
                [
                    linear_to_u8(glow.red + live.red + structure * 0.38),
                    linear_to_u8(glow.green + live.green + structure * 0.55),
                    linear_to_u8(glow.blue + live.blue + structure),
                ]
            })
            .collect()
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.config.width + x
    }

    fn stimulate_boundary(&mut self) {
        let height = self.config.height as f32;
        let pulse = self.age as f32 * 0.037;
        let sources = [
            (0.23 + pulse.sin() * 0.025, Spectrum::new(0.92, 0.08, 0.03)),
            (
                0.50 + (pulse * 0.73 + 1.2).sin() * 0.035,
                Spectrum::new(0.05, 0.88, 0.12),
            ),
            (
                0.77 + (pulse * 0.51 + 2.4).sin() * 0.025,
                Spectrum::new(0.04, 0.13, 0.96),
            ),
        ];
        for (position, spectrum) in sources {
            let center = (position * height).round() as isize;
            for offset in -2_isize..=2 {
                let y = (center + offset).clamp(0, self.config.height as isize - 1) as usize;
                let envelope = (3 - offset.unsigned_abs()) as f32 / 3.0;
                let stimulus = spectrum * (0.16 * envelope);
                let index = self.index(1, y);
                self.sites[index].energy += stimulus;
                self.introduced += stimulus.total();
            }
        }
    }

    fn exchange(
        &self,
        left: usize,
        right: usize,
        follows_gradient: bool,
        change: &mut [Spectrum],
        local_flow: &mut [f32],
    ) {
        let a = self.sites[left];
        let b = self.sites[right];
        let conductance = 0.32 + 0.68 * (a.permeability * b.permeability).sqrt();
        let relaxation = (a.energy - b.energy) * (self.config.diffusion * conductance);
        let drift = if follows_gradient {
            a.energy * (self.config.gradient * conductance)
        } else {
            Spectrum::ZERO
        };
        let flow = relaxation + drift;
        change[left] -= flow;
        change[right] += flow;
        let magnitude = flow.magnitude();
        local_flow[left] += magnitude;
        local_flow[right] += magnitude;
    }
}

fn visible_coupling(x: usize, y: usize, width: usize, height: usize) -> Spectrum {
    let nx = x as f32 / (width - 1) as f32;
    let ny = y as f32 / (height - 1) as f32;
    let dx = (nx - 0.67) / 0.34;
    let dy = (ny - 0.50) / 0.45;
    let radius = (dx * dx + dy * dy).sqrt();
    let angle = dy.atan2(dx);
    let folded_ring = 0.52 + 0.095 * (angle * 5.0 + radius * 7.0).sin();
    let ring = soft_band((radius - folded_ring).abs(), 0.055);
    let inner = soft_band((radius - 0.19 - 0.035 * (angle * 3.0).cos()).abs(), 0.045);
    let vein = soft_band((dy - 0.19 * (nx * TAU * 2.3).sin()).abs(), 0.035)
        * soft_band((nx - 0.58).abs(), 0.42);
    let aperture = (ring.max(inner * 0.8).max(vein * 0.42) * edge_gate(nx)).clamp(0.0, 1.0);
    if aperture <= 0.001 {
        return Spectrum::ZERO;
    }
    let hue = (angle / TAU + 1.0).fract();
    let red = (0.5 + 0.5 * (TAU * hue).cos()).powi(2);
    let green = (0.5 + 0.5 * (TAU * (hue - 1.0 / 3.0)).cos()).powi(2);
    let blue = (0.5 + 0.5 * (TAU * (hue - 2.0 / 3.0)).cos()).powi(2);
    let floor = 0.16 * aperture;
    Spectrum::new(
        floor + aperture * red,
        floor + aperture * green,
        floor + aperture * blue,
    )
}

fn soft_band(distance: f32, width: f32) -> f32 {
    let normalized = (distance / width).clamp(0.0, 1.0);
    0.5 + 0.5 * (PI * normalized).cos()
}

fn edge_gate(nx: f32) -> f32 {
    ((nx - 0.18) / 0.12).clamp(0.0, 1.0)
}

fn linear_to_u8(value: f32) -> u8 {
    let gamma_corrected = value.clamp(0.0, 1.0).powf(1.0 / 2.2);
    (gamma_corrected * 255.0).round() as u8
}

struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^ (value >> 31)
    }

    fn unit_f32(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1_u32 << 24) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_dimensions_are_rejected() {
        let result = World::new(Config {
            width: 8,
            ..Config::default()
        });
        assert!(result.is_err());
    }

    #[test]
    fn energy_accounting_remains_closed() {
        let mut world = World::new(Config {
            width: 48,
            height: 32,
            seed: 7,
            ..Config::default()
        })
        .unwrap();
        world.evolve_for(500);
        let measured = world.measurements();
        let relative_error = measured.accounting_error.abs() / measured.introduced;
        assert!(
            relative_error < 0.000_03,
            "relative error was {relative_error}"
        );
        assert!(measured.radiated > 0.0);
        assert!(measured.resident > 0.0);
    }

    #[test]
    fn identical_initial_conditions_have_identical_futures() {
        let config = Config {
            width: 40,
            height: 28,
            seed: 91,
            ..Config::default()
        };
        let mut first = World::new(config).unwrap();
        let mut second = World::new(config).unwrap();
        first.evolve_for(100);
        second.evolve_for(100);
        assert_eq!(first.rgb8(), second.rgb8());
        assert_eq!(
            first.measurements().introduced,
            second.measurements().introduced
        );
    }

    #[test]
    fn repeated_flow_changes_the_material() {
        let mut world = World::new(Config {
            width: 40,
            height: 28,
            seed: 19,
            ..Config::default()
        })
        .unwrap();
        let before = world.measurements().mean_permeability;
        world.evolve_for(700);
        let after = world.measurements().mean_permeability;
        assert!(after > before * 1.02, "before {before}, after {after}");
        assert!(world.measurements().organized_fraction > 0.0);
    }
}
