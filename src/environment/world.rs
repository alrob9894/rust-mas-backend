use rand::Rng;
use std::thread;
use crate::agents::agent::Compartment;
use crate::agents::agent;
use crate::environment::social_network::SocialNetwork;
use crate::environment::social_network;
use crate::disease::disease::Disease;
use std::thread::JoinHandle;

#[allow(dead_code)]
pub struct World {
    agent_number: u32,
    avg_friends: u8,
    pub patients_zero: Vec<u32>,
    sn: social_network::SocialNetwork,

}

impl World {
    pub fn new(
        agent_number: u32,
        avg_friends: u8,
    ) -> World {
        World {
            agent_number,
            avg_friends,
            patients_zero: Vec::new(),
            sn: SocialNetwork::new(agent_number, avg_friends),
        }
    }

    pub fn init_world(&mut self, num_patients_zero: u8, context: &zmq::Context, infection_risk: f64, severe_course_chance: f64, time_symptomatic: u8) -> Vec<JoinHandle<()>> {
        let mut agent_threads: Vec<JoinHandle<()>> = vec![];
        self.patients_zero = determine_patients_zero(self.agent_number as f32, num_patients_zero);
        let disease = Disease::new(time_symptomatic, infection_risk, severe_course_chance);
        for id in 1..=self.agent_number {
            let context = context.clone();
            let neighbors = self.sn.network.remove(&id).unwrap();
            let p_zero = self.patients_zero.clone();
            agent_threads.push(thread::Builder::new().name(format!("{}{}", "agent:", id).to_string()).spawn(move || {
                let mut agent = agent::Agent::new(id, neighbors, &context);
                if p_zero.contains(&id) {
                    agent.compartment = Compartment::INFECTED;
                    agent.disease = Some(disease.clone());
                    agent.days_infected = 1;
                } else {
                    agent.compartment = Compartment::SUSCEPTIBLE;
                    agent.disease = None;
                }
                agent.run();
            }).unwrap());
        }
        agent_threads
    }
}

// set-up the initial patient-zero
fn determine_patients_zero(num_of_agents: f32, num_patients_zero: u8) -> Vec<u32> {
    let mut patients_zero = Vec::new();
    let mut rng = rand::thread_rng();
    while (patients_zero.len() as u8) <= num_patients_zero {
        let patient_zero = rng.gen_range(1..=num_of_agents as u32);
        if !(patients_zero.contains(&patient_zero)) {
            patients_zero.push(patient_zero);
        }
    }
    patients_zero
}

