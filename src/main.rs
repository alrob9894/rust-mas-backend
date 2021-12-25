use std::collections::HashMap;
use std::thread;
use crate::agent::{Compartment};
use rand::Rng;
use std::time::{SystemTime, Duration};
use crate::model_controller_agent::ControllerAgent;
use rocket::{Rocket, Build, Response, Request};
use std::thread::sleep;
use rocket::tokio::time::{sleep as sleepT, Duration as DurationT};
use rocket::response::stream::TextStream;
use rocket::http::{Status, Method, Header};
use rocket::futures::io::Cursor;
use rocket::fairing::{Fairing, Info, Kind};

#[macro_use]
extern crate rocket;

mod agent;
mod disease;
mod authority_agent;
mod statistic_agent;
mod model_controller_agent;
mod messages;

const ITERATIONS: u32 = 100;
const AGENT_NUMBER: u32 = 100;
const AVERAGE_NUM_FRIENDS: u8 = 10;
//Chance Meeting another agent
const CONTACT_PROB: f64 = 0.2;
const INFECTION_RISK: f64 = 0.1;
//Percentage of Patient-Zero
const PATIENT_ZERO_RATE: f32 = 0.002;
const TIME_SYMPTOMATIC: u8 = 14;

#[get("/")]
fn index() -> &'static str {
    "Hello World"
}

#[get("/world")]
fn world() -> &'static str {
    "hello, world!"
}

#[get("/delay/<seconds>")]
async fn delay(seconds: u64) -> String {
    sleepT(DurationT::from_secs(seconds)).await;
    format!("Waited for {} seconds", seconds)
    // format!("Das könnte etwas dauern ... \n")
}

#[get("/infinite-hellos")]
fn hello() -> TextStream![&'static str] {
    TextStream! {
        let mut interval = rocket::tokio::time::interval(DurationT::from_secs(1));
        for n in 1..11 {
            yield "hello \n";
            interval.tick().await;
        }
    }
}
// #[post]

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Attaching CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![index])
        .mount("/hi", routes![world])
        // .mount("/hi", routes![index])
        .mount("/hi", routes![delay])
        .mount("/", routes![hello])
    // .attach(req_fairing)
        .attach(CORS)
}


// #[allow(unused_variables)]
// fn main() {
//     let start = SystemTime::now();
//
//
//
//
//     let controller_handle = thread::spawn(|| {
//         let controller = ControllerAgent::new(0, ITERATIONS as u16);
//         controller.run_sim();
//     });
//
//     {
//         let mut sn = init_sn(AGENT_NUMBER, AVERAGE_NUM_FRIENDS);
//         let p_zeros= determine_patients_zero(AGENT_NUMBER as f32, PATIENT_ZERO_RATE);
//         for id in 1..=AGENT_NUMBER {
//             let neighbors = sn.remove(&id).unwrap();
//             let mut compartment = Compartment::Susceptible;
//             if p_zeros.contains(&id) { compartment = Compartment::Infected}
//             let agent_handle = thread::spawn(move || {
//                 let mut agent = agent::Agent::new(id, neighbors);
//                 agent.set_compartment(compartment);
//                 agent.run_agent();
//             });
//         }
//     }
//
//     controller_handle.join().unwrap();
//     let time = SystemTime::now().duration_since(start).unwrap();
//     println!("Time needed: {:?}", time);
// }


// k (number of connected neighbors) is assumed to be an even integer
fn init_sn(n: u32, k: u8) -> HashMap<u32, Vec<u32>> {
    let mut ring_mesh = create_ring(n, k);
    let sn = ring2sn(n, k, ring_mesh);
    sn
}


// creating a ring-mesh as foundation for the social network
fn create_ring(n: u32, k: u8) -> HashMap<u32, Vec<u32>> {
    let mut neighbor_map = HashMap::new();
    for agent_id in 1..=n {
        let mut neighbor_vec = Vec::new();
        for neighbor in 1..=k {
            neighbor_vec.push(determine_neighbor_id(n as i32, agent_id as i32, neighbor as i32));
        }
        neighbor_map.insert(agent_id, neighbor_vec);
    }
    neighbor_map
}

#[allow(unused_variables)]
// transform the ring-mesh into a social network using the Watts-Strogatz algorithm
fn ring2sn(n: u32, k: u8, mut ring: HashMap<u32, Vec<u32>>) -> HashMap<u32, Vec<u32>> {
    let mut rng = rand::thread_rng();

    for (agent_id, neighbor_vec) in ring.clone() {
        for contact_distance in 1..=AVERAGE_NUM_FRIENDS {
            // only delete edge with 50% chance and when its a right-side neighbor
            if rng.gen_bool(0.5) && contact_distance % 2 == 0 {
                // delete the old contact bridge
                let contact_del = determine_neighbor_id(n as i32, agent_id as i32, contact_distance as i32);
                ring.get_mut(&agent_id).unwrap().retain(|&contact| contact == contact_del);
                ring.get_mut(&(contact_distance as u32)).unwrap().retain(|&contact| contact == agent_id);
                // create a new contact bridge to a randomly selected node
                let mut contact_new = rng.gen_range(1..=AGENT_NUMBER);
                while contact_new != contact_del && contact_new != agent_id && !(neighbor_vec.contains(&contact_new)) {
                    contact_new = rng.gen_range(1..=AGENT_NUMBER);
                }
                ring.get_mut(&agent_id).unwrap().append(&mut vec![contact_new]);
                ring.get_mut(&contact_new).unwrap().append(&mut vec![agent_id]);
            }
        }
    }
    ring
}


// function to determine the ID of an neighbor based on the relative distance of one specific node
fn determine_neighbor_id(n: i32, id: i32, neighbor: i32) -> u32 {
    match neighbor % 2 {
        // even neighbor_distance represents clockwise neighbors
        0 =>
            if (id + neighbor / 2) >= 1 && (id + neighbor / 2) <= n { (id + neighbor / 2) as u32 } else { ((id + neighbor / 2) % n) as u32 },
        // uneven neighbor_distance represents counterclockwise neighbors
        1 =>
            if (id - (neighbor / 2 + 1)) >= 1 && (id - (neighbor / 2 + 1)) <= n { (id - (neighbor / 2 + 1)) as u32 } else { (n - (id - (neighbor / 2 + 1)) * (-1)) as u32 }
        _ => 0,
    }
}


// set-up the initial patient-zero
fn determine_patients_zero(num_of_agents: f32, patient_zero_rate: f32) -> Vec<u32> {
    let mut patients_zero = Vec::new();
    let mut rng = rand::thread_rng();
    let num_p_zeros = (num_of_agents * patient_zero_rate).ceil();
    while (patients_zero.len() as f32) <= num_p_zeros {
        let patient_zero = rng.gen_range(1..=AGENT_NUMBER);
        if !(patients_zero.contains(&patient_zero)) {
            patients_zero.push(patient_zero);
        }
    }
    patients_zero
}
