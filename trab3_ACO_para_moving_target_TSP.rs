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
            pheromones: vec![vec![vec![1.0;n]; n]; n-1],
            ants,
        }
    }

    fn create_trails(&mut self, instance: &Instance, alfa: f64, beta: f64) {
        self.ants.par_iter_mut().for_each(|ant| {
            let mut visited = vec![false; self.pheromones.len()+1];
            let mut rng = rand::thread_rng();
            let mut which: usize = 0;
            let mut current_time = 0.0;
            let mut dist: f64;
            ant.trail.clear();
            visited[which] = true;
            ant.trail.push(which);
    
            for _ in 0..self.pheromones.len() {
                let step = ant.trail.len()-1;
                let mut v: Vec<_> = self.pheromones[step][which].iter().enumerate()
                    .filter(|&(i, _)| !visited[i])
                    .map(|(i, &value)| (i, value))
                    .collect();
                let mut sum_prob = 0.0;        
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
                which = select_random_index(&v, &mut rng);
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
                self.pheromones[i][ant.trail[i]][ant.trail[i+1]] += 100.0 / (ant.eval as f64);
            }
        }
    }

    fn evaporation(&mut self, evaporation_factor: f64) {
        for i in 0..self.pheromones.len() {
            for j in 0..self.pheromones.len() {
                for k in 0..self.pheromones.len()+1 {
                    self.pheromones[i][j][k] *= 1.0 - evaporation_factor;
                }
            }
        }
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
        evaluation as i64
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
                    (solution[i], solution[j]) = (solution[j], solution[i]);
                    eval_temp = self.evaluate(&solution);
                    if eval_temp < eval_better_option {
                        better_option = solution.clone();
                        eval_better_option = eval_temp;
                    }
                    (solution[i], solution[j]) = (solution[j], solution[i]);
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

    fn ils(&self, init: &Vec<usize>) -> Vec<usize> {
        let mut rng = rand::thread_rng();
        let mut i: usize;
        let mut j: usize;
        let len = init.len();
        let init_pert_strength = 1;
        let max_pert_strength = 4;
        let mut pert_strength = init_pert_strength;
        let mut cont_aux = 1;
        let cont_max = 20 + (init.len() as f64/4.0) as usize;
        let mut solution = init.clone();
        let mut eval_solution: i64;
        let mut best_solution = solution.clone();
        let mut eval_best_solution = self.evaluate(&mut best_solution);

        while pert_strength < max_pert_strength {
            solution = self.local_search(&solution);
            eval_solution = self.evaluate(&mut solution);
            if eval_best_solution > eval_solution {
                best_solution = solution.clone();
                eval_best_solution = eval_solution;
                pert_strength = init_pert_strength;
                cont_aux = 1;
                println!("{:?} {:?}", pert_strength, eval_best_solution);
            } else {
                cont_aux += 1;
                if cont_aux >= cont_max {
                    pert_strength += 1;
                    cont_aux = 0;
                    println!("{}", pert_strength);
                }
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

    fn aco(&self, n_ants: usize, max_gen: usize, alfa: f64, beta: f64, evaporation_factor: f64) -> Vec<usize> {
        let mut colony = Colony::new(self.targets.len(), n_ants);
        let mut best_trail = colony.ants[0].trail.clone();
        let mut eval_best = i64::MAX;
        for _ in 0..max_gen {
            colony.create_trails(&self, alfa, beta);
            colony.evaporation(evaporation_factor);
            colony.reinforcement(&self);
            for ant in &colony.ants { 
                if ant.eval < eval_best {
                    best_trail = ant.trail.clone();
                    eval_best = ant.eval;
                    println!("{}", eval_best);
                }
            }
        }
        best_trail
    }
}

fn main() {
    //let mut rng = rand::thread_rng();
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { 
        println!("Arquivo nao especificado");
        return;
    }
    let mut instance = Instance::new();
    let mttsp_file_name = args[1].as_str();

    instance.set_data(mttsp_file_name);

    let start = Instant::now();
    let mut aco_solution: Vec<usize> = vec![];
    let mut best_eval = i64::MAX;
    let mut better = vec![(1 as usize, 1000 as usize, 1.0, 1.0, 1.0)];
    for i in 0..1 {
        let n_ants = 128000;
        let max_gen = 16;
        let alfa = 1.0;
        let beta = 5.0;
        let evaporation_factor = 0.05;
        aco_solution = instance.aco(n_ants, max_gen, alfa, beta, evaporation_factor);
        let aco_eval = instance.evaluate(&aco_solution);
        if best_eval >= aco_eval {
            best_eval = aco_eval;
            better.push((n_ants, max_gen, alfa, beta, evaporation_factor));
        }
        println!("{}",i);
    }
    println!("{:?} {}", better, best_eval);
    let sa_solution = instance.ils(&aco_solution);
    let time_elapsed = start.elapsed();
    println!("ACO: {:?}", aco_solution);
    println!("{}", instance.evaluate(&aco_solution));
    println!("SA: {:?}", sa_solution);
    println!("{}", instance.evaluate(&sa_solution));
    println!("{:?}", time_elapsed);
}
