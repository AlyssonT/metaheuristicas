use std::fs::File;
use std::io::*;
use std::env;
use rand::Rng;
use rand::rngs::ThreadRng;
use std::time::Instant;
use rayon::prelude::*;

fn read_file(file_name: &str) -> String {
    let mut file = File::open(file_name)
        .expect("Falha ao abrir o arquivo");

    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Falha ao ler o conteudo do arquivo");

    content
}

fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    (dx * dx + dy * dy).sqrt()
}

fn interception_point(ct: f64, target_x: f64, target_y: f64, agent_x: f64, agent_y: f64, agent_speed: f64, tx_speed: f64, ty_speed:f64) -> (f64, f64) {
    let mut dist = f64::INFINITY;
    let mut dt: f64;
    let mut ds: f64;
    let mut next_target_x = target_x + tx_speed*ct;
    let mut next_target_y = target_y + ty_speed*ct;
    let mut prev_target_x: f64;
    let mut prev_target_y: f64;

    while dist > 10e-4 {
        ds = distance(agent_x, agent_y, next_target_x, next_target_y);
        dt = ds/agent_speed;
        prev_target_x = next_target_x;
        prev_target_y = next_target_y;
        next_target_x = target_x + tx_speed*(ct + dt);
        next_target_y = target_y + ty_speed*(ct + dt);
        dist = distance(prev_target_x, prev_target_y, next_target_x, next_target_y);
    }
    (next_target_x, next_target_y)
}

fn select_random_index(v: &Vec<(usize, f64)>, rng: &mut ThreadRng) -> usize {
    let mut rand_value = rng.gen_range(0.0..=1.0);
    for &(i, value) in v.iter() {
        if rand_value < value {
            return i;
        }
        rand_value -= value;
    }
    return 0
}

#[derive(Clone, Copy)]
struct Target {
    x: f64,
    y: f64,
    x_speed: f64,
    y_speed: f64,
}

struct Ant {
    trail: Vec<usize>,
    eval: i64,
}

impl Ant {
    fn new() -> Self {
        Self {
            trail: Vec::new(),
            eval: 0,
        }
    }
}

struct Colony {
    pheromones: Vec<Vec<Vec<f64>>>,
    ants: Vec<Ant>,
}

impl Colony {
    fn new(n: usize, n_ants: usize) -> Self {
        let mut ants: Vec<Ant> = vec![];
        for _ in 0..n_ants {
            ants.push(Ant::new());
        }
        Self {
            pheromones: vec![vec![vec![1.5; n]; n]; n-1],
            ants,
        }
    }

    fn create_trails(&mut self, instance: &Instance, alfa: f64, beta: f64, q0: f64) {
        self.ants.par_iter_mut().for_each(|ant| {
            let mut visited = vec![false; self.pheromones.len() + 1];
            let mut rng = rand::thread_rng();
            let mut which: usize = 0;
            let mut current_time = 0.0;
            let mut dist: f64;
            ant.trail.clear();
            visited[which] = true;
            ant.trail.push(which);
    
            for i in 0..self.pheromones.len() {
                let mut v: Vec<_> = self.pheromones[i][which].iter().enumerate()
                    .filter(|&(i, _)| !visited[i])
                    .map(|(i, &value)| (i, value))
                    .collect();
                let mut sum_prob = 0.0;
                if v.is_empty() {break;}
                for i in 0..v.len() {
                    let inter_point = interception_point(current_time, instance.targets[v[i].0].x, instance.targets[v[i].0].y,
                        instance.targets[which].x + current_time*instance.targets[which].x_speed, instance.targets[which].y + current_time*instance.targets[which].y_speed, instance.agent_speed,
                        instance.targets[v[i].0].x_speed, instance.targets[v[i].0].y_speed);
                    let distance_travelled = distance(instance.targets[which].x + current_time*instance.targets[which].x_speed,
                        instance.targets[which].y + current_time*instance.targets[which].y_speed, inter_point.0, inter_point.1); 
                    v[i] = (v[i].0, v[i].1.powf(alfa) * (1.0/distance_travelled).powf(beta));
                    sum_prob += v[i].1;
                }
                for i in 0..v.len() {
                    v[i].1 /= sum_prob;
                }
                let prev = (instance.targets[which].x + current_time*instance.targets[which].x_speed,
                    instance.targets[which].y + current_time*instance.targets[which].y_speed);

                let q = rng.gen_range(0.0..1.0);
                if q < q0 {
                    which = select_random_index(&v, &mut rng);
                } else {
                    which = v[rng.gen_range(0..v.len())].0;
                }

                let inter_point2 = interception_point(current_time, instance.targets[which].x, instance.targets[which].y,
                    prev.0, prev.1, instance.agent_speed,
                    instance.targets[which].x_speed, instance.targets[which].y_speed);
                dist = distance(prev.0, prev.1, inter_point2.0,
                    inter_point2.1);
                current_time += dist / instance.agent_speed;
                visited[which] = true;
                ant.trail.push(which);
            }
        });
    }
    
    fn reinforcement(&mut self, instance: &Instance) {
        for ant in &mut self.ants {
            ant.eval = instance.evaluate(&ant.trail);
            for i in 0..(ant.trail.len()-1) {
                self.pheromones[i][ant.trail[i]][ant.trail[i+1]] += 1.0 / (ant.eval as f64);
            }
        }
    }

    fn evaporation(&mut self, evaporation_factor: f64) {
        self.pheromones.par_iter_mut().for_each(|v| {
            for i in 0..v.len() {  
                for j in 0..v.len() {
                    v[i][j] = v[i][j]*(1.0 - evaporation_factor) + f64::EPSILON;
                }
            }
        })
    }
}

struct Instance {
    targets: Vec<Target>,
    agent_speed: f64,
}

impl Instance {
    fn new() -> Self {
        Self {
            targets: Vec::new(),
            agent_speed: 0.0,
        }
    }

    fn set_data(&mut self, mttsp_file_name: &str) {
        let content = read_file(mttsp_file_name);
        let lines: Vec<Vec<&str>> = content.lines()
            .map(|line| line.split_whitespace().collect()).collect();

        self.agent_speed = lines[0][1].parse().unwrap_or(0.0);
    
        for i in 1..lines.len() {
            let xf: f64 = lines[i][1].parse().unwrap_or(0.0);
            let yf: f64 = lines[i][2].parse().unwrap_or(0.0);
            let x_s: f64 = lines[i][3].parse().unwrap_or(0.0);
            let y_s: f64 = lines[i][4].parse().unwrap_or(0.0);
    
            let target = Target {
                x: xf,
                y: yf,
                x_speed: x_s,
                y_speed: y_s,
            };
            self.targets.push(target);
        }
    }

    fn evaluate(&self, solution: &Vec<usize>) -> i64 {
        let mut evaluation = 0.0;
        let mut current_time = 0.0;
        let mut agent_x = self.targets[solution[0]].x;
        let mut agent_y = self.targets[solution[0]].y;
        let mut travelled_distance: f64;
        for i in 1..solution.len() {
            let target = self.targets[solution[i]];
            let interception_point = interception_point(
                current_time, target.x, target.y,
                agent_x, agent_y, self.agent_speed, target.x_speed, target.y_speed
            );
            travelled_distance = distance(agent_x, agent_y, interception_point.0, interception_point.1);
            evaluation += travelled_distance;
            current_time += travelled_distance/self.agent_speed;
            agent_x = target.x + current_time*target.x_speed;
            agent_y = target.y + current_time*target.y_speed;
        }
        let interception_point_origin = interception_point(
            current_time, self.targets[solution[0]].x, self.targets[solution[0]].y,
            agent_x, agent_y, self.agent_speed, self.targets[solution[0]].x_speed, self.targets[solution[0]].y_speed
        );
        evaluation += distance(agent_x, agent_y, interception_point_origin.0, interception_point_origin.1);
        evaluation.round() as i64
    }

    fn local_search(&self, init: &Vec<usize>) -> Vec<usize> {
        let mut solution = init.clone();
        let mut better_option: Vec<usize> = vec![];
        let mut eval_first = self.evaluate(&mut solution);
        let mut eval_temp: i64;
        let mut eval_better_option: i64 = i64::MAX;
        let size = solution.len();
        loop {
            for i in 1..size-1 {
                for j in i+1..size {
                    solution[i..=j].reverse();
                    eval_temp = self.evaluate(&solution);
                    if eval_temp < eval_better_option {
                        better_option = solution.clone();
                        eval_better_option = eval_temp;
                    }
                    solution[i..=j].reverse();
                }
            }
            if eval_better_option < eval_first {
                solution = better_option;
                better_option = vec![];
                eval_first = eval_better_option;
                eval_better_option = i64::MAX;
            } else {
                return solution;
            }
        }
    }

    fn ils(&self, init: &Vec<usize>, after_aco: bool) -> Vec<usize> {
        let mut rng = rand::thread_rng();
        let n_iter = if after_aco {15} else {50};
        let mut i: usize;
        let mut j: usize;
        let len = init.len();
        let pert_strength = if init.len() > 30 {(init.len() as f64 * 0.2).ceil() as usize} else {3};
        let mut solution = init.clone();
        let mut eval_solution: i64;
        let mut best_solution = solution.clone();
        let mut eval_best_solution = self.evaluate(&mut best_solution);

        for k in 0..n_iter {
            println!("{}", k);
            solution = self.local_search(&solution);
            eval_solution = self.evaluate(&mut solution);

            if eval_best_solution > eval_solution {
                best_solution = solution.clone();
                eval_best_solution = eval_solution;
            }
            
            for _ in 0..pert_strength {
                i = rng.gen_range(1..len);
                j = rng.gen_range(1..len);
                while i == j {
                    j = rng.gen_range(1..len);
                }
                let element = solution.remove(i);
                solution.insert(j, element);
            }
        }
        best_solution
    }

    fn aco(&self, n_ants: usize, max_gen: usize, alfa: f64, beta: f64, evaporation_factor: f64, q: f64) -> Vec<usize> {
        let mut colony = Colony::new(self.targets.len(), n_ants);
        let mut best_trail = colony.ants[0].trail.clone();
        let mut eval_best = i64::MAX;
        for i in 0..max_gen {
            colony.create_trails(&self, alfa, beta, q);
            colony.evaporation(evaporation_factor);
            colony.reinforcement(&self);
            for ant in &colony.ants { 
                if ant.eval < eval_best {
                    best_trail = ant.trail.clone();
                    eval_best = ant.eval;
                    println!("{} {}", eval_best, i);
                }
            }
        }
        best_trail
    }
}

fn main() {
    // let mut rng = rand::thread_rng();
    // let args: Vec<String> = env::args().collect();
    // if args.len() < 2 { 
    //     println!("Arquivo nao especificado");
    //     return;
    // }
    // let mut instance = Instance::new();
    // let mttsp_file_name = args[1].as_str();

    // instance.set_data(mttsp_file_name);
    let file_order = vec![
        // "burma14-zero.mttsp",
        // "burma14-max1.mttsp",
        // "burma14-max10.mttsp",
        // "burma14-max100.mttsp",
        // "burma14-same_dir.mttsp",
        // "berlin52-zero.mttsp",
        // "berlin52-max1.mttsp",
        // "berlin52-max10.mttsp",
        // "berlin52-max100.mttsp",
        // "berlin52-same_dir.mttsp",
        "st70-zero.mttsp",
        "st70-max1.mttsp",
        "st70-max10.mttsp",
        "st70-max100.mttsp",
        "st70-same_dir.mttsp",
        "gr120-zero.mttsp",
        "gr120-max1.mttsp",
        "gr120-max10.mttsp",
    ];

    let mut results: Vec<String> = vec![];

    let n_ants = 3000;
    let max_gen = 200;
    let alfa = 1.0;
    let beta = 5.0;
    let q0 = 0.96;
    let evaporation_factor = 0.1;
    
    let mut file = match File::create("RESULTS-zero5.txt") {
        Ok(file) => file,
        Err(err) => panic!("Erro ao criar o arquivo: {}", err),
    };
    for f in file_order {
        let mut instance = Instance::new();
        instance.set_data(f);

        let mut aco_solution: Vec<usize>;
        let mut aco_eval: i64;
        let mut aco_ils_solution: Vec<usize>;
        let mut aco_ils_eval: i64;
        let mut ils_solution: Vec<usize>;
        let mut ils_eval: i64;

        match file.write_all(format!("--------------{}-------------\n", f).as_bytes()) {
            Ok(()) => (),
            Err(_) => panic!("Nao consegui escrever no arquivo"),
        }
        for _ in 0..5 {
            let start = Instant::now();
            aco_solution = instance.aco(n_ants, max_gen, alfa, beta, evaporation_factor, q0);
            let end_aco = start.elapsed();
            
            aco_ils_solution = instance.ils(&aco_solution, true);
            let end_aco_ils = start.elapsed();

            let start_ils = Instant::now();
            ils_solution = instance.ils(&(0..aco_solution.len()).collect(), false);
            let end_ils = start_ils.elapsed();

            aco_eval = instance.evaluate(&aco_solution);
            aco_ils_eval = instance.evaluate(&aco_ils_solution);
            ils_eval = instance.evaluate(&ils_solution);

            // results.push(format!("ACO EVAL: {}, TIME: {:?}\n", aco_eval, end_aco));
            // results.push(format!("ACO+ils EVAL: {}, TIME: {:?}\n", aco_ils_eval, end_aco_ils));
            // results.push(format!("ils EVAL: {}, TIME: {:?}\n\n", ils_eval, end_ils));
            match file.write_all(format!("ACO EVAL: {}, TIME: {:?}\n", aco_eval, end_aco).as_bytes()) {
                Ok(()) => (),
                Err(_) => panic!("Nao consegui escrever no arquivo"),
            }
            match file.write_all(format!("ACO+ils EVAL: {}, TIME: {:?}\n", aco_ils_eval, end_aco_ils).as_bytes()) {
                Ok(()) => (),
                Err(_) => panic!("Nao consegui escrever no arquivo"),
            }
            match file.write_all(format!("ils EVAL: {}, TIME: {:?}\n\n", ils_eval, end_ils).as_bytes()) {
                Ok(()) => (),
                Err(_) => panic!("Nao consegui escrever no arquivo"),
            }
            
            println!("ACO EVAL: {}, TIME: {:?}\n", aco_eval, end_aco);
            println!("ACO+ils EVAL: {}, TIME: {:?}\n", aco_ils_eval, end_aco_ils);
            println!("ils EVAL: {}, TIME: {:?}\n", ils_eval, end_ils);
        }
    }

    

    // for s in results {
        // match file.write_all(s.as_bytes()) {
        //     Ok(()) => (),
        //     Err(_) => panic!("Nao consegui escrever no arquivo"),
        // }
    // }

    // let mut start = Instant::now();
    // let mut aco_solution: Vec<usize> = vec![];
    // let mut best_eval = i64::MAX;
    // let mut better = vec![(1 as usize, 1000 as usize, 1.0, 1.0, 1.0, 1.0)];
    // for i in 0..1 {
    //     let n_ants = 3000;
    //     let max_gen = 200;
    //     let alfa = 1.0;
    //     let beta = 5.0;
    //     let q0 = 0.96;
    //     let evaporation_factor = 0.1;
    //     aco_solution = instance.aco(n_ants, max_gen, alfa, beta, evaporation_factor, q0);
    //     let aco_eval = instance.evaluate(&aco_solution);
    //     if best_eval >= aco_eval {
    //         best_eval = aco_eval;
    //         better.push((n_ants, max_gen, alfa, beta, evaporation_factor, q0));
    //     }
    //     println!("{}",i);
    //     println!("{:?}", better[better.len()-1]);
    // }
    // println!("{:?} {}", better, best_eval);
    // let end_aco = start.elapsed();
    // let sa_solution = instance.sa(&aco_solution, 4.0, 0.9995, 0.05, aco_solution.len()*2);
    // let end_aco_sa = start.elapsed();
    // start = Instant::now();
    // let sa = instance.sa(&(0..aco_solution.len()).collect(), 10000.0, 0.9995, 0.05, aco_solution.len()*2);
    // let end_sa = start.elapsed();
    // println!("ACO: {:?}", aco_solution);
    // println!("{:?}", end_aco);
    // println!("{}", instance.evaluate(&aco_solution));
    // println!("SA + ACO: {:?}", sa_solution);
    // println!("{:?}", end_aco_sa);
    // println!("{}", instance.evaluate(&sa_solution));
    // println!("SA: {:?}", sa_solution);
    // println!("{:?}", end_sa);
    // println!("{}", instance.evaluate(&sa));
}
