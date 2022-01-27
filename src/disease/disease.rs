use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Disease{
    pub time_symptomatic: u8,
    pub infection_risk: f64 ,
    pub severe_course_chance: f64,
}

impl Disease{
    pub fn new (
        time_symptomatic: u8,
        infection_risk: f64,
        severe_course_chance: f64,
    ) -> Disease { Disease {
        time_symptomatic,
        infection_risk,
        severe_course_chance
    }}
}