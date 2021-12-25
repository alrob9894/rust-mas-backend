// #[derive]
struct StatisticAgent<'a>{
    id: u32,
    context: &'a zmq::Context,
    server: zmq::Socket,
}

impl<'a> StatisticAgent<'a>{
    fn new(
        id:u32,
        context: &zmq::Context,
    ) -> StatisticAgent {
        StatisticAgent {
            id,
            context,
            server: context.socket(zmq::REP).unwrap(),
        }
    }
}

//     //append the SIR-statistic of the current iteration
//     let current_stats = agents_statistics(&agents);
//     plot_data.0[(iteration-1) as usize] = current_stats.0;
//     plot_data.1[(iteration-1) as usize] = current_stats.1;
//     plot_data.2[(iteration-1) as usize] = current_stats.2;
// }


// draw_chart(plot_data);
//


// fn draw_chart((s, i, r): ([u32; ITERATIONS as usize], [u32; ITERATIONS as usize], [u32; ITERATIONS as usize])) {
//     if std::fs::metadata("./images").is_err() {
//         std::fs::create_dir("./images");
//     }
//     let drawing_area = BitMapBackend::new("./images/2.1.png", (600, 400))
//         .into_drawing_area();
//
//     drawing_area.fill(&WHITE).unwrap();
//
//     let mut chart = ChartBuilder::on(&drawing_area)
//         .set_label_area_size(LabelAreaPosition::Left, 40)
//         .set_label_area_size(LabelAreaPosition::Bottom, 40)
//         .build_cartesian_2d(0..ITERATIONS as i32, 0..AGENT_NUMBER as i32)
//         .unwrap();
//
//     chart.draw_series(
//         LineSeries::new(
//             (0..).zip(s.iter()).map(|(x, &y)| {
//                 (x as i32, y as i32)
//             }),
//             &GREEN)
//     ).unwrap();
//
//     chart.draw_series(
//         LineSeries::new(
//             (0..).zip(i.iter()).map(|(x, &y)| {
//                 (x as i32, y as i32)
//             }),
//             &RED)
//     ).unwrap();
//
//     chart.draw_series(
//         LineSeries::new(
//             (0..).zip(r.iter()).map(|(x, &y)| {
//                 (x as i32, y as i32)
//             }),
//             &BLUE)
//     ).unwrap();
//
//     chart.configure_mesh().draw().unwrap();
// }
//
// // infected agents counter
// fn agents_statistics(agents: &HashMap<u32, Agent>) -> (u32, u32, u32) {
//     let mut agent_stats = (0, 0, 0);
//     for (_, agent) in agents {
//         match agent.compartment {
//             Compartment::Susceptible => {agent_stats.0 += 1;},
//             Compartment::Infected => {agent_stats.1 += 1;},
//             Compartment::Removed => {agent_stats.2 += 1;},
//         }
//     }
//     agent_stats
// }