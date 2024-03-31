use crate::graph::{Graph, Node, Time, Weight};

pub fn print_dma(dma: &[DistanceMatrix]) {
    for t in 0..dma.len() {
        println!("Time : {}", t);
        for i in 0..dma[t as usize].distances.len() {
            for j in 0..dma[t as usize].distances[i].len() {
                let distance_to_str = if dma[t as usize][i as Node][j] == Weight::MAX {
                    "∞".to_string()
                } else {
                    dma[t as usize][i as Node][j as usize].to_string()
                };
                print!("{} ", distance_to_str);
            }
            println!();
        }
    }
}

pub fn sum_dma(dma : &Vec<DistanceMatrix>, max_time: Time) -> (f64, u64) {
    let mut sum = 0.0;
    //let mut sum_int = 0;
    let mut reachables = 0;
    let distances_len = dma[0 as usize].distances.len();
    for t in 0..max_time {
        for i in 0..distances_len {
            for j in 0..distances_len {
                if i == j {
                    continue;
                }
                let distance = dma[t as usize][i as Node][j];
                if distance + t < max_time && distance != 0 {
                    //sum_int += distance as u64;
                    sum += 1.0 / (distance as f64);
                    reachables += 1;
                }
            }
        }
    }
    sum -= distances_len as f64;
    sum /= dma.len() as f64 * (distances_len * (distances_len - 1)) as f64;
    return (sum, reachables);
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistanceMatrix {
    pub distances: Vec<Vec<Weight>>,
}

impl DistanceMatrix {
    pub fn new(graph: &Graph) -> DistanceMatrix {
        let mut distances = vec![vec![u16::MAX; graph.nb_nodes]; graph.nb_nodes];
        for i in 0..graph.nb_nodes {
            distances[i][i] = 1;
        }
        DistanceMatrix { distances }
    }

    pub fn zero(graph: &Graph) -> DistanceMatrix {
        let mut distances = vec![vec![0; graph.nb_nodes]; graph.nb_nodes];
        for i in 0..graph.nb_nodes {
            for j in 0..graph.nb_nodes {
                if i == j {
                    distances[i][j] = 1;
                }
                else {
                    distances[i][j] = 0;
                }
            }
        }
        DistanceMatrix { distances }
    }

    pub fn size(size: usize) -> DistanceMatrix {
        let mut distances = vec![vec![0; size]; size];
        for i in 0..size {
            for j in 0..size {
                if i == j {
                    distances[i][j] = 1;
                }
            }
        }
        DistanceMatrix { distances }
    }

    pub fn print(&self) {
        for row in &self.distances {
            for distance in row {
                if *distance == u16::MAX {
                    print!("inf ");
                } else {
                    print!("{} ", distance);
                }
            }
            println!();
        }
    }
}

// TODO : i can index the first time with u16 but not the other times i have to cast to usize, kinda ugly
impl std::ops::Index<Node> for DistanceMatrix {
    type Output = Vec<Weight>;

    fn index(&self, index: Node) -> &Self::Output {
        return &self.distances[index as usize];
    }
}
impl std::ops::IndexMut<Node> for DistanceMatrix {
    fn index_mut(&mut self, index: Node) -> &mut Self::Output {
        return &mut self.distances[index as usize];
    }
}