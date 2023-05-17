use std::fs::File;
use std::io::*;
use std::env;
use rand::Rng;

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

        for i in 0..100 {
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

            solution = self.local_search(&solution);
            if eval_best_solution > self.evaluate(&mut solution) {
                best_solution = solution;
                eval_best_solution = self.evaluate(&mut best_solution);
            }
            current = rng.gen_range(0..self.cities.len());
            solution = vec![current];
            visited = vec![false; self.cities.len()];
            visited[current] = true;
            println!("{}", i);
        }
        best_solution
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 { 
        println!("Nenhum arquivo especificado");
        return;    
    }
    let mut instance = Instance::new();
    let tspp_file_name = args[1].as_str();
    let matrix_file_name = match args.len() {
        3..=usize::MAX => Some(args[2].as_str()),
        _ => None,
    };

    instance.set_data(tspp_file_name, matrix_file_name);

    let mut solution = instance.grasp();
    println!("{:?}", solution);
    println!("{:?}", instance.evaluate(&mut solution));
}
