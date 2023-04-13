use std::fs::File;
use std::io::*;

#[derive(Debug)]
struct City {
    x: f64,
    y: f64,
}

fn calculate_distance(c1: &City, c2: &City) -> i32 {
    ((c2.x-c1.x)*(c2.x-c1.x) + (c2.y-c1.y)*(c2.y-c1.y)).sqrt().round() as i32
}

fn read_file(file_name: &str) -> String {
    let mut file = File::open(file_name)
        .expect("Falha ao abrir o arquivo");

    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Falha ao ler o conteudo do arquivo");

    content
}

fn main() {
    let file_name = "st70.tspp";
    let matrix_file_name = "st70-mix2.txt";

    let content = read_file(file_name);
    let matrix_content = read_file(matrix_file_name);

    let matrix_content_lines: Vec<Vec<&str>> = matrix_content.lines()
        .map(|line| line.split_whitespace().collect()).collect();

    let mut matrix: Vec<Vec<i32>> = Vec::new();
    for i in 1..matrix_content_lines.len() {
        matrix.push(vec![]);
        matrix_content_lines[i].iter()
            .for_each(|value| matrix[i-1].push(value.parse::<i32>().unwrap_or(0)));
    }

    let lines: Vec<Vec<&str>> = content.lines()
        .map(|line| line.split_whitespace().collect()).collect();

    let mut cities: Vec<City> = Vec::new();

    for i in 1..lines.len() {
        let xf: f64 = lines[i][1].parse().expect("Erro ao converter numero");
        let yf: f64 = lines[i][2].parse().expect("Erro ao converter numero");

        let city = City {
            x: xf,
            y: yf,
        };

        cities.push(city);
    }

    let mut solution: Vec<usize> = (0..cities.len()).collect();
    solution.push(0);
    let mut evaluation = 0;
    for i in 0..(solution.len()-1) {
        evaluation += calculate_distance(&cities[solution[i]], &cities[solution[i+1]]);
        if i==0 {
            evaluation += matrix[solution[0]][0];
        }
        if i<solution.len()-2 {
            evaluation += matrix[solution[i+1]][i+1];
        }
    }

    println!("{}", evaluation);
}
