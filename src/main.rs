mod agents;
mod messages;
mod environment;
mod disease;
// mod messages {
// include!(concat!(env!("OUT_DIR"), "/messages/environment"));
// only this absolute direct path is enabling auto-completion
// include!("/Users/r_kessler/Documents/uni/vorlesungen/masterprojekt/rust/target/debug/build/rust_mas-8b260aab452a624c/out/messages/environment");
// }

use crate::agents::controller_agent::ControllerAgent;
use crate::agents::statistic_agent;
use crate::environment::world;

#[macro_use]
extern crate rocket;
use std::thread;

use std::time::SystemTime;
use rocket::{Build, Request, Response, Rocket};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::form::Form;
use rocket::http::Header;
use rocket::serde::{json::Json};
use crate::agents::statistic_agent::{RoundResult};
use crate::agents::authority_agent::AuthorityAgent;

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Attaching CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[derive(FromForm)]
struct JSONForm<> {
    agent_number: u32,
    avg_friends: u8,
    contact_prob: f64,
    //icu_beds: u16,
    infection_risk: f64,
    mm_checked: bool,
    mm_incidence_limit: u32,
    patients_zero: u8,
    sdr_checked: bool,
    sdr_incidence_limit: u32,
    severe_course_chance: f64,
    sim_duration: u16,
    time_symptomatic: u8,
}

#[post("/", data = "<form_data>")]
fn run_sim(form_data: Form<JSONForm<>>) -> Json<Vec<RoundResult>> {

    let result = start_simulation(
        form_data.agent_number,
        form_data.avg_friends,
        form_data.contact_prob,
        form_data.infection_risk,
        form_data.mm_checked,
        form_data.mm_incidence_limit,
        form_data.patients_zero,
        form_data.sdr_checked,
        form_data.sdr_incidence_limit,
        form_data.severe_course_chance,
        form_data.sim_duration,
        form_data.time_symptomatic,
    );
    result
}

#[options("/")]
fn opt() {}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![opt])
        .mount("/", routes![run_sim])
        .attach(CORS)
}


#[allow(unused_variables)]
fn start_simulation(
    agent_number: u32,
    avg_friends: u8,
    contact_prob: f64,
    infection_risk: f64,
    mm_checked: bool,
    mm_incidence_limit: u32,
    patients_zero: u8,
    sdr_checked: bool,
    sdr_incidence_limit: u32,
    severe_course_chance: f64,
    sim_duration: u16,
    time_symptomatic: u8,
) -> Json<Vec<RoundResult>> {
    let start = SystemTime::now();
    let context = zmq::Context::new();
    let context_stats = context.clone();
    let context_authority = context.clone();
    let context_world = context.clone();

    let controller_handler = thread::spawn(move || {
        let controller = ControllerAgent::new(0, &context.clone(), sim_duration);
        controller.run(agent_number);
    });

    let sim_results = thread::spawn(move || {
        let mut stats_agent = statistic_agent::StatisticAgent::new(0, sim_duration, agent_number, &context_stats.clone());
        stats_agent.run();
        stats_agent.results_to_json()
    });

    let authority_handle = thread::spawn(move || {
        let authority_agent = AuthorityAgent::new(0, mm_checked, mm_incidence_limit, sdr_checked, sdr_incidence_limit, &context_authority.clone());
        authority_agent.run();
    });

    let mut world = world::World::new(agent_number, avg_friends);
    let agent_threads = world.init_world(
        patients_zero, &context_world, infection_risk, severe_course_chance, time_symptomatic);

    for agent_thread in agent_threads {
        let _ = agent_thread.join();
    }
    controller_handler.join().unwrap();
    let time = SystemTime::now().duration_since(start).unwrap();
    println!("Time needed: {:?}", time);

    sim_results.join().unwrap()
}