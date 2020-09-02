use serde::{Deserialize, Serialize};
use std::time::Duration;

pub type StudyId = String;
pub type TrialId = u32;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Span {
    start: f64,
    end: f64,
}

impl Span {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDef {
    pub name: String,
    #[serde(flatten)]
    pub range: ParamRange,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ParamRange {
    Numerical {
        min: f64,
        max: f64,
        #[serde(default, skip_serializing_if = "Scale::is_default")]
        scale: Scale,
    },
    Categorical {
        choices: Vec<String>,
    },
}

impl ParamRange {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueRange {
    #[serde(default = "neg_infinity", skip_serializing_if = "is_neg_infinity")]
    pub start: f64,

    #[serde(default = "infinity", skip_serializing_if = "is_infinity")]
    pub end: f64,
}

impl ValueRange {
    fn is_default(&self) -> bool {
        self.start == std::f64::NEG_INFINITY && self.end == std::f64::INFINITY
    }
}

impl Default for ValueRange {
    fn default() -> Self {
        Self {
            start: std::f64::NEG_INFINITY,
            end: std::f64::INFINITY,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Record {
    Study(StudyRecord),
    Eval(EvalRecord),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StudyRecord {
    pub id: StudyId,
    pub name: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRecord {
    pub study: StudyId,
    pub trial: TrialId,
    pub spans: Vec<Span>,

    #[serde(with = "nullable_f64_vec")]
    pub params: Vec<f64>,

    #[serde(with = "nullable_f64_vec")]
    pub values: Vec<f64>,

    pub state: EvalState,
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
