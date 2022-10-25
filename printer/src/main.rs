mod cli;

use model::{input::read_input_from_stdin, map::Map, object::Object};

fn main() {
    let (task, solution) = read_input_from_stdin().unwrap();
    let solution = solution.unwrap_or_default();

    let mut objects = Vec::with_capacity(task.objects.len() + solution.0.len());
    objects.extend(task.objects.into_iter().map(Object::from));
    objects.extend(solution.0.into_iter().map(Object::from));

    let map = Map::new(task.width, task.height, objects);

    println!("{}", map);
}
