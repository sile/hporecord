use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;

pub mod io;

pub type StudyId = String;
pub type TrialId = u32;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Span {
    pub start: f64,
    pub end: f64,
}

impl Span {
    pub fn new(start: f64, end: f64) -> Self {
        Self { start, end }
    }
}

impl Span {
    // TODO: remove
    pub fn start(self) -> Duration {
        Duration::from_secs_f64(self.start)
    }

    pub fn end(self) -> Duration {
        Duration::from_secs_f64(self.end)
    }

    pub fn duration(self) -> Duration {
        self.end() - self.start()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanDef {
    pub name: String,
}

impl SpanDef {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParamDef {
    pub name: String,
    #[serde(flatten)]
    pub range: ParamRange,
}

impl ParamDef {
    pub fn continuous(name: impl Into<String>, min: f64, max: f64) -> Self {
        Self {
            name: name.into(),
            range: ParamRange::continuous(min, max),
        }
    }

    pub fn log_continuous(name: impl Into<String>, min: f64, max: f64) -> Self {
        Self {
            name: name.into(),
            range: ParamRange::log_continuous(min, max),
        }
    }

    pub fn discrete(name: impl Into<String>, min: f64, max: f64, step: f64) -> Self {
        Self {
            name: name.into(),
            range: ParamRange::discrete(min, max, step),
        }
    }

    pub fn categorical(name: impl Into<String>, choices: Vec<String>) -> Self {
        Self {
            name: name.into(),
            range: ParamRange::categorical(choices),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Scale {
    Linear,
    Log,
}

impl Scale {
    fn is_default(&self) -> bool {
        *self == Self::Linear
    }
}

impl Default for Scale {
    fn default() -> Self {
        Self::Linear
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ParamRange {
    Numerical {
        min: f64,
        max: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        step: Option<f64>,
        #[serde(default, skip_serializing_if = "Scale::is_default")]
        scale: Scale,
    },
    Categorical {
        choices: Vec<String>,
    },
}

impl ParamRange {
    fn continuous(min: f64, max: f64) -> Self {
        Self::Numerical {
            min,
            max,
            step: None,
            scale: Scale::Linear,
        }
    }

    fn log_continuous(min: f64, max: f64) -> Self {
        Self::Numerical {
            min,
            max,
            step: None,
            scale: Scale::Log,
        }
    }

    fn discrete(min: f64, max: f64, step: f64) -> Self {
        Self::Numerical {
            min,
            max,
            step: Some(step),
            scale: Scale::Linear,
        }
    }

    fn categorical(choices: Vec<String>) -> Self {
        Self::Categorical { choices }
    }

    pub fn min(&self) -> f64 {
        match self {
            Self::Numerical { min, .. } => *min,
            Self::Categorical { .. } => 0.0,
        }
    }

    pub fn max(&self) -> f64 {
        match self {
            Self::Numerical { max, .. } => *max,
            Self::Categorical { choices } => choices.len() as f64,
        }
    }

    pub fn scale(&self) -> Scale {
        match self {
            Self::Numerical { scale, .. } => *scale,
            Self::Categorical { .. } => Scale::Linear,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueDef {
    pub name: String,
    #[serde(default, skip_serializing_if = "ValueRange::is_default")]
    pub range: ValueRange,
    pub direction: Direction,
}

impl ValueDef {
    pub fn new(name: impl Into<String>, direction: Direction) -> Self {
        Self {
            name: name.into(),
            range: ValueRange::default(),
            direction,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueRange {
    #[serde(default = "neg_infinity", skip_serializing_if = "is_neg_infinity")]
    pub min: f64,

    #[serde(default = "infinity", skip_serializing_if = "is_infinity")]
    pub max: f64,
}

impl ValueRange {
    fn is_default(&self) -> bool {
        self.min == std::f64::NEG_INFINITY && self.max == std::f64::INFINITY
    }
}

impl Default for ValueRange {
    fn default() -> Self {
        Self {
            min: std::f64::NEG_INFINITY,
            max: std::f64::INFINITY,
        }
    }
}

fn neg_infinity() -> f64 {
    std::f64::NEG_INFINITY
}

fn is_neg_infinity(v: &f64) -> bool {
    *v == std::f64::NEG_INFINITY
}

fn infinity() -> f64 {
    std::f64::INFINITY
}

fn is_infinity(v: &f64) -> bool {
    *v == std::f64::INFINITY
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Direction {
    Minimize,
    Maximize,
}

impl Direction {
    pub fn better(self, x: f64, y: f64) -> f64 {
        if self == Direction::Minimize {
            x.min(y)
        } else {
            x.max(y)
        }
    }

    pub const fn is_minimize(self) -> bool {
        matches!(self, Self::Minimize)
    }

    pub const fn is_maximize(self) -> bool {
        matches!(self, Self::Maximize)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Record {
    Study(StudyRecord),
    Eval(EvalRecord),
    // TODO: StudyEnd
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StudyRecord {
    pub id: StudyId,
    #[serde(default)]
    pub attrs: BTreeMap<String, String>,
    pub spans: Vec<SpanDef>,
    pub params: Vec<ParamDef>,
    pub values: Vec<ValueDef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum EvalState {
    Complete,
    Interim,
    Failed,
    Infeasible,
}

impl EvalState {
    pub const fn is_complete(self) -> bool {
        matches!(self, Self::Complete)
    }

    pub const fn is_interm(self) -> bool {
        matches!(self, Self::Interim)
    }

    pub const fn is_failed(self) -> bool {
        matches!(self, Self::Failed)
    }

    pub const fn is_infeasible(self) -> bool {
        matches!(self, Self::Infeasible)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRecord {
    pub study: StudyId,
    pub trial: TrialId,
    pub state: EvalState,
    pub spans: Vec<Span>,

    #[serde(with = "nullable_f64_vec")]
    pub params: Vec<f64>,

    #[serde(with = "nullable_f64_vec")]
    pub values: Vec<f64>,
}

mod nullable_f64_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::f64::NAN;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: Vec<Option<f64>> = Deserialize::deserialize(deserializer)?;
        Ok(v.into_iter()
            .map(|v| if let Some(v) = v { v } else { NAN })
            .collect())
    }

    pub fn serialize<S>(v: &[f64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let v = v
            .iter()
            .map(|v| if v.is_finite() { Some(*v) } else { None })
            .collect::<Vec<_>>();
        v.serialize(serializer)
    }
}
