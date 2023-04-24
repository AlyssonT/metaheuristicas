use std::fs::File;
use std::io::*;
use std::env;

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
}

impl Instance {
    fn new() -> Self {
        Self {
            matrix: Vec::new(),
            cities: Vec::new(),
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
    }

    fn evaluate(&self, solution: &mut Vec<usize>) -> i32 {
        solution.push(solution[0]);
        let mut evaluation = 0;
        for i in 0..(solution.len()-1) {
            evaluation += City::calculate_distance(&self.cities[solution[i]], &self.cities[solution[i+1]]);
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
                    current_distance = City::calculate_distance(&self.cities[current], &self.cities[j]);
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
                    current_distance = City::calculate_distance(&self.cities[current], &self.cities[j]);
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
                    current_distance = City::calculate_distance(&self.cities[current_back], &self.cities[j]);
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
}

fn localSearch(init: &Vec<usize>) -> Vec<usize> {
    let mut solution = init.clone();
    for i in 1..solution.len() {
        for j in i+1..solution.len() {
            
        }
    }
    ()
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut instance = Instance::new();
    let tspp_file_name = args[1].as_str();
    let matrix_file_name = match args.len() {
        3..=usize::MAX => Some(args[2].as_str()),
        _ => None,
    };

    instance.set_data(tspp_file_name, matrix_file_name);

    let mut solution_sequential = instance.sequential();
    let mut solution_greedy = instance.greedy();
    let mut solution_2_way = instance.greedy_2_way();
    let mut solution_LS_init_sequential = localSearch(&solution_sequential);
    let mut solution_LS_init_greedy = localSearch(&solution_greedy);
    let mut solution_LS_init_greedy_2_way = localSearch(&solution_2_way);

    println!("Sequential: {}", instance.evaluate(&mut solution_sequential));
    println!("Greedy: {}", instance.evaluate(&mut solution_greedy));
    println!("Greedy_2_way: {}", instance.evaluate(&mut solution_2_way));
}
