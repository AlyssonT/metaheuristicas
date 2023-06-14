use std::fs::File;
use std::io::*;
use std::env;
use rand::Rng;
use std::time::Instant;

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

#[derive(Clone, Copy)]
struct Target {
    x: f64,
    y: f64,
    x_speed: f64,
    y_speed: f64,
    speed: f64,
}

struct Ant {
    solution: Vec<usize>,
}

struct Colony {

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
                speed: (x_s*x_s + y_s*y_s).sqrt(),
            };
            self.targets.push(target);
        }
    }

    fn evaluate(&self, solution: &Vec<usize>) -> i32 {
        let mut evaluation = 0.0;
        let mut current_time = 0.0;
        let mut agent_x = self.targets[solution[0]].x;
        let mut agent_y = self.targets[solution[0]].y;
        let mut travelled_distance: f64;

        for i in 1..solution.len() {
            let target = self.targets[solution[i]];
            let interception_point = interception_point(current_time, target.x, target.y,
                agent_x, agent_y, self.agent_speed, target.x_speed, target.y_speed);

            travelled_distance = distance(agent_x, agent_y, interception_point.0, interception_point.1);
            evaluation += travelled_distance;
            agent_x = target.x;
            agent_y = target.y;
            current_time += travelled_distance/self.agent_speed;
        }
        evaluation += distance(agent_x, agent_y, self.targets[solution[0]].x, self.targets[solution[0]].y);
        evaluation as i32
    }

    fn local_search(&self, init: &Vec<usize>) -> Vec<usize> {
        let mut solution = init.clone();
        let mut better_option: Vec<usize> = vec![];
        let mut eval_first = self.evaluate(&mut solution);
        let mut eval_temp: i32;
        let mut eval_better_option: i32 = i32::MAX;
        let size = solution.len();
        loop {
            for i in 1..size-1 {
                for j in i+1..size {
                    solution[i..=j].reverse();
                    eval_temp = self.evaluate(&mut solution);
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
                eval_better_option = i32::MAX;
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
        let max_pert_strength = 2;
        let mut pert_strength = init_pert_strength;
        let mut cont_aux = 1;
        let cont_max = 3000;
        let mut solution = init.clone();
        let mut eval_solution: i32;
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
                    cont_aux = 1;
                    println!("{:?}", pert_strength);
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
}

fn interception_point(ct: f64, target_x: f64, target_y: f64, agent_x: f64, agent_y: f64, agent_speed: f64, tx_speed: f64, ty_speed:f64) -> (f64, f64) {
    let mut dist = f64::INFINITY;
    let mut dt: f64;
    let mut ds: f64;
    let mut next_target_x = target_x + tx_speed*ct;
    let mut next_target_y = target_y + ty_speed*ct;
    let mut prev_target_x: f64;
    let mut prev_target_y: f64;

    while dist > 10e-7 {
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

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 { 
        println!("Arquivo nao especificado");
        return;
    }
    let mut instance = Instance::new();
    let mttsp_file_name = args[1].as_str();

    instance.set_data(mttsp_file_name);

    let solution: Vec<usize> = vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13];
    let start = Instant::now();
    let local_search_solution = instance.ils(&solution);
    let time_elapsed = start.elapsed();
    println!("{}", instance.evaluate(&local_search_solution));
    println!("{:?}", local_search_solution);
    println!("{:?}", time_elapsed);
}
