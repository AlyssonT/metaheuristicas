use std::fs::File;
use std::io::*;
use std::env;
use rand::Rng;
use rand::seq::SliceRandom;
use std::time::Instant;

fn read_file(file_name: &str) -> String {
    let mut file = File::open(file_name)
        .expect("Falha ao abrir o arquivo");

    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Falha ao ler o conteudo do arquivo");

    content
}

#[derive(Debug)]
struct City {
    x: f64,
    y: f64,
}

impl City {
    pub fn calculate_distance(c1: &City, c2: &City) -> i32 {
        ((c2.x-c1.x)*(c2.x-c1.x) + (c2.y-c1.y)*(c2.y-c1.y)).sqrt().round() as i32
    } 
}

struct Instance {
    matrix: Vec<Vec<i32>>,
    cities: Vec<City>,
    distances: Vec<Vec<i32>>,
}

impl Instance {
    fn new() -> Self {
        Self {
            matrix: Vec::new(),
            cities: Vec::new(),
            distances: Vec::new(),
        }
    }

    fn set_data(&mut self, tspp_file_name: &str, matrix_file_name: Option<&str>) {
        let content = read_file(tspp_file_name);
        let matrix_content = match matrix_file_name {
            Some(text) => read_file(text),
            None => "".to_string(),
        };
    
        if matrix_content.len() > 0 {
            let matrix_content_lines: Vec<Vec<&str>> = matrix_content.lines()
                .map(|line| line.split_whitespace().collect()).collect();

            for i in 1..matrix_content_lines.len() {
                self.matrix.push(vec![]);
                matrix_content_lines[i].iter()
                    .for_each(|value| self.matrix[i-1].push(value.parse::<i32>().unwrap_or(0)));
            }
        }
    
        let lines: Vec<Vec<&str>> = content.lines()
            .map(|line| line.split_whitespace().collect()).collect();
    
        for i in 1..lines.len() {
            let xf: f64 = lines[i][1].parse().unwrap_or(0.0);
            let yf: f64 = lines[i][2].parse().unwrap_or(0.0);
    
            let city = City {
                x: xf,
                y: yf,
            };
    
            self.cities.push(city);
        }

        let mut distance: i32;
        self.distances = vec![vec![0; self.cities.len()]; self.cities.len()];
        for i in 0..self.cities.len() {
            for j in i+1..self.cities.len() {
                distance = City::calculate_distance(&self.cities[i], &self.cities[j]);
                self.distances[i][j] = distance;
                self.distances[j][i] = distance;
            }
        }
    }

    fn evaluate(&self, solution: &mut Vec<usize>) -> i32 {
        solution.push(solution[0]);
        let mut evaluation = 0;
        for i in 0..(solution.len()-1) {
            evaluation += self.distances[solution[i]][solution[i+1]];
            if self.matrix.len() > 0 {
                if i==0 {
                    evaluation += self.matrix[solution[0]][0];
                }
                if i<solution.len()-2 {
                    evaluation += self.matrix[solution[i+1]][i+1];
                }
            }
        }
        solution.pop();
        evaluation
    }

    fn sequential(&self) -> Vec<usize> {
        (0..self.cities.len()).collect()
    }

    fn greedy(&self) -> Vec<usize> {
        let number_cities = self.cities.len();
        let mut solution = vec![0];
        let mut visited = vec![false; number_cities];
        visited[0] = true;
        let mut current = 0;
        let mut min_distance = i32::MAX;
        let mut current_distance: i32;
        let mut next_city: usize = 0;
        for _i in 0..(number_cities-1) {
            for j in 0..number_cities {
                if !visited[j] {
                    current_distance = self.distances[current][j];
                    if current_distance < min_distance {
                        min_distance = current_distance;
                        next_city = j;
                    }

                }
            }
            visited[next_city] = true;
            solution.push(next_city);
            current = next_city;
            min_distance = i32::MAX;
        }
        solution
    }

    fn greedy_2_way(&self) -> Vec<usize> {
        let number_cities = self.cities.len();
        let mut solution_front = vec![0];
        let mut solution_back: Vec<usize> = vec![];
        let mut visited = vec![false; number_cities];
        visited[0] = true;
        let mut current = 0;
        let mut current_back = 0;
        let mut min_distance = i32::MAX;
        let mut current_distance: i32;
        let mut next_city: usize = 0;
        let mut pushed_in_front: bool = true;
        for _ in 0..(number_cities-1) {
            for j in 0..number_cities {
                if !visited[j] {
                    current_distance = self.distances[current][j];
                    if current_distance < min_distance {
                        min_distance = current_distance;
                        next_city = j;
                        pushed_in_front = true;
                    }

                }
            }
            for j in 0..number_cities {
                if current == current_back  {break}
                if !visited[j] {
                    current_distance = self.distances[current_back][j];
                    if current_distance < min_distance {
                        min_distance = current_distance;
                        next_city = j;
                        pushed_in_front = false;
                    }

                }
            }
            if pushed_in_front {
                current = next_city;
                solution_front.push(next_city);
            } else {
                current_back = next_city;
                solution_back.push(next_city);
            }
            visited[next_city] = true;
            min_distance = i32::MAX;
        }
        let mut solution = vec![];
        for &city in solution_back.iter().rev() {
            solution.push(city);
        }
        for city in solution_front {
            solution.push(city);
        }
        solution
    }

    fn local_search(&self, init: &Vec<usize>) -> Vec<usize> {
        let mut solution = init.clone();
        let mut solution_temp: Vec<usize> = vec![];
        let mut better_option: Vec<usize> = vec![];
        let mut eval_first = self.evaluate(&mut solution);
        let mut eval_temp: i32;
        let mut eval_better_option: i32 = i32::MAX;
        let has_penalty = self.matrix.len() > 0;
        let size = solution.len();
        loop {
            for i in 1..size-1 {
                for j in i+1..size {
                    if has_penalty {
                        solution_temp = solution.clone();
                        solution_temp[i..=j].reverse();
                        eval_temp = self.evaluate(&mut solution_temp);
                    } else {
                        eval_temp = eval_first;
                        eval_temp -= self.distances[solution[i-1]][solution[i]];
                        eval_temp += self.distances[solution[i-1]][solution[j]];
                        if j==size-1 {
                            eval_temp -= self.distances[solution[j]][solution[0]];
                            eval_temp += self.distances[solution[i]][solution[0]];
                        } else {
                            eval_temp -= self.distances[solution[j]][solution[j+1]];
                            eval_temp += self.distances[solution[i]][solution[j+1]];
                        }
                    }
                    if eval_temp < eval_better_option {
                        if has_penalty {
                            better_option = solution_temp;
                            solution_temp = vec![];
                        }
                        else {
                            better_option = solution.clone();
                            better_option[i..=j].reverse();
                        }
                        
                        eval_better_option = eval_temp;
                    }
                }
            }
            if eval_better_option < eval_first {
                solution = better_option;
                better_option = vec![];
                eval_first = self.evaluate(&mut solution);
                eval_better_option = i32::MAX;
            } else {
                return solution;
            }
        }
    }

    fn grasp(&self) -> Vec<usize> {
        let mut choice: usize;
        let mut visited = vec![false; self.cities.len()];
        let mut rng = rand::thread_rng();
        let mut current: usize = rng.gen_range(0..self.cities.len());
        visited[current] = true;
        let mut solution = vec![current];
        let mut best_solution: Vec<usize> = self.sequential();
        let mut eval_best_solution = self.evaluate(&mut best_solution);
        let mut current_distances: Vec<(usize, &i32)>;

        for i in 0..500 {
            current_distances = self.distances[current].iter().enumerate()
                    .filter(|&c| !visited[c.0])
                    .collect();
            while !current_distances.is_empty() {
                current_distances.sort_by(|a,b| a.1.cmp(b.1));
                choice = rng.gen_range(0..=((current_distances.len() as f64 * 0.2).floor() as usize));
                let next_city = current_distances[choice].0;
                visited[next_city] = true;
                solution.push(next_city);

                current = current_distances[choice].0;
                current_distances = self.distances[current].iter().enumerate()
                    .filter(|&c| !visited[c.0])
                    .collect();
            }

            let mut new_solution = self.local_search(&solution);
            let new_eval = self.evaluate(&mut new_solution);
            if eval_best_solution > new_eval {
                best_solution = new_solution;
                eval_best_solution = new_eval;
            }
            current = rng.gen_range(0..self.cities.len());
            solution.clear();
            solution.push(current);
            visited.fill(false);
            visited[current] = true;
            println!("{}", i);
        }
        best_solution
    }

    fn sa(&self, init: &Vec<usize>, temp: f64, alfa: f64, freeze: f64, max_iter: usize) -> Vec<usize> {
        let mut rng = rand::thread_rng();
        let mut solution = init.clone();
        let mut eval_solution: i32;
        let mut best_solution = solution.clone();
        let mut eval_best: i32 = self.evaluate(&mut best_solution);
        let mut neighbor: Vec<usize>;
        let mut eval_neighbor: i32;
        let mut temperature = temp;
        let mut delta: i32;
        while temperature > freeze {
            
            let mut i: usize; let mut j: usize;
            for _ in 0..max_iter {
                eval_solution = self.evaluate(&mut solution);
                i = rng.gen_range(0..solution.len());
                j = rng.gen_range(0..solution.len());
                while i == j {
                    j = rng.gen_range(0..solution.len());
                }
                if i > j { (i,j) = (j,i); }
                neighbor = solution.clone();
                neighbor[i..=j].reverse();
                eval_neighbor = self.evaluate(&mut neighbor);
                delta = eval_neighbor - eval_solution;
                if delta < 0 {
                    solution = neighbor;
                    if eval_neighbor < eval_best {
                        best_solution = solution.clone();
                        eval_best = self.evaluate(&mut best_solution);
                    }
                } else {
                    let x = rng.gen_range(0.0..=1.0);
                    let e: f64 = 2.7182818284590452353602874713527;
                    if x < (e.powf((-delta as f64) / temperature)) {
                        solution = neighbor;
                    } 
                }
            }
            temperature *= alfa;
        }
        self.local_search(&best_solution)
    }

    fn ils(&self, init: &Vec<usize>) -> Vec<usize> {
        let mut rng = rand::thread_rng();
        let mut i: usize;
        let mut j: usize;
        let len = init.len();
        let pert_strength = (len as f64 * 0.02).ceil() as i32;
        let mut solution = init.clone();
        let mut eval_solution: i32;
        let mut best_solution = solution.clone();
        let mut eval_best_solution = self.evaluate(&mut best_solution);

        for _ in 0..10000 {
            solution = self.local_search(&solution);
            eval_solution = self.evaluate(&mut solution);

            if eval_best_solution > eval_solution {
                best_solution = solution.clone();
                eval_best_solution = eval_solution;
            }
            
            for _ in 0..pert_strength {
                i = rng.gen_range(0..len);
                j = rng.gen_range(0..len);
                while i == j {
                    j = rng.gen_range(0..len);
                }
                (solution[i], solution[j]) = (solution[j], solution[i]);
            }
        }
        best_solution
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut rng = rand::thread_rng();

    if args.len() < 3 { 
        println!("Arquivo ou metodo nao especificados");
        return;    
    }
    let mut instance = Instance::new();
    let tspp_file_name = args[2].as_str();
    let matrix_file_name = match args.len() {
        4..=usize::MAX => Some(args[3].as_str()),
        _ => None,
    };

    instance.set_data(tspp_file_name, matrix_file_name);

    let mut solution: Vec<usize>;
    let mut random_solution = instance.sequential();
    random_solution.shuffle(&mut rng);
    let start = Instant::now();
    match args[1].to_lowercase().as_str() {
        "grasp" => {
            println!("GRASP:");
            solution = instance.grasp();
        },
        "sa" => {
            println!("Simulated Annealing:");
            solution = instance.sa(&random_solution, 10000.0, 0.9999, 0.01, instance.cities.len()*2);
        },
        "ils" => {
            println!("ILS:");
            solution = instance.ils(&random_solution);
        }
        _ => { println!("Nenhum metodo com esse nome!"); return },
    }
    let time_elapsed = start.elapsed();
    println!("{:?}", solution);
    println!("{}", instance.evaluate(&mut solution));
    println!("Tempo de execucao: {:?}", time_elapsed);
}
