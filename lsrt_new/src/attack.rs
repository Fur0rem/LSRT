use std::{fs::File, io::{BufRead, BufReader}};

use crate::graph::{Graph, Time, Node};
use sscanf::scanf;

pub trait Attack : std::fmt::Debug {
    // efficiency + reachables
    fn efficiency(&self, graph: &Graph, max_time: Time) -> (f64, usize);
    /*
    err_code sum_dma(double * ret_sum, uint64_t * ret_reachables,  DISTANCE_MATRIX_ARRAY * dma){
    def_err_handler(!(ret_sum && dma && ret_reachables), "sum_dma", ERR_NULL);

    double sum = 0; 
    uint64_t reachables = 0 ; 
    for(uint32_t i = 0 ; i < dma->nb_matrixes ; i++){
        for(uint32_t j = 0 ; j < dma->matrixes[i].cols * dma->matrixes[i].rows ; j++){
            
            if( (dma->matrixes[i].values[j] != UINT8_MAX) && (dma->matrixes[i].values[j]) && i!=j ){
                sum += (double)((double)1/(double)(dma->matrixes[i].values[j])) ;
                reachables++;
            }
        }
    }
    sum -= dma->matrixes[0].rows ;
    sum /=  dma->nb_matrixes *(((dma->matrixes[0].rows) * (dma->matrixes[0].rows - 1) ));
    //sum normalized by time * (|N| * |N-1|) with G=(V,N)

    reachables -= dma->matrixes[0].rows ;
    //reachables /= (((dma->matrixes[0].rows) * (dma->matrixes[0].rows - 1) ));
   
    *ret_sum = sum ; 
    *ret_reachables = reachables ;
    return ERR_OK ;
    }//not tested ; should be okay though 
    */
}

#[derive(Debug)]
pub struct DynamicAttack {
    blocked_links: Vec<Vec<((Node, Node), Vec<Time>)>>,
    delta: Time
}

#[derive(Debug)]
pub struct StaticAttack {
    blocked_links: Vec<Vec<Node>>,
    delta: Time,
}

impl dyn Attack {
    pub fn from_file(file: &str, graph: &Graph) -> Box<dyn Attack> {
        let file = File::open(file).unwrap();
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();

        if line.trim() == "S" {
            return Box::new(StaticAttack::from_file(&mut reader, graph));
        }
        else if line.trim() == "D" {
            return Box::new(DynamicAttack::from_file(&mut reader, graph));
        }

        panic!("Invalid attack file");
    }
}

impl StaticAttack {
    pub fn from_file(reader: &mut BufReader<File>, graph: &Graph) -> Self {
        // Format : first line contains number of blocked links then delta (useless)
        // Then, the list of blocked links separated by a space
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        println!("Line : {}", line);
        //let (nb_blocks, _delta) = scanf!(line, "{} {} ", usize, usize).unwrap();
        let iter = line.split_whitespace();
        let mut iter = iter.map(|x| x.parse().unwrap());
        let nb_blocks = iter.next().unwrap();
        let delta = iter.next().unwrap();

        // while there is new line, read it and append to the result
        let mut blocked_links = Vec::with_capacity(graph.nb_nodes);
        for _ in 0..graph.nb_nodes {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let blocks = line.split_whitespace().map(|x| x.parse().unwrap()).collect();
            blocked_links.push(blocks);
        }

        return Self {
            blocked_links,
            delta
        };
    }
}


// TODO : that
impl DynamicAttack {
    pub fn from_file(reader: &mut BufReader<File>, graph: &Graph) -> Self {
        return Self {
            blocked_links: Vec::new(),
            delta: 0
        };
    }
}
impl Attack for DynamicAttack {
    fn efficiency(&self, graph: &Graph, max_time: Time) -> (f64, usize) {
        return (0.0, 0);
    }
}

impl Attack for StaticAttack {
    // Create a graph without the links that got blocked
    fn efficiency(&self, graph: &Graph, max_time: Time) -> (f64, usize) {
        let mut new_graph = graph.clone();
        for i in 0..self.blocked_links.len() {
            for j in 0..self.blocked_links[i].len() {
                new_graph.remove_link(i as Node, self.blocked_links[i][j]);
            }
        }

        let (distance_matrix, shortest_paths) = new_graph.all_pairs_shortest_paths(max_time);
        println!("{:?}", distance_matrix);
        let mut sum = 0.0;
        let mut reachables = 0;
        for t in 0..max_time {
            for i in 0..distance_matrix.distances.len() {
                for j in 0..distance_matrix.distances[i].len() {

                    let value = distance_matrix.distances[i][j];
                    if value > max_time {
                        continue;
                    }
                    let value = value + t;
                    if value > max_time {
                        continue;
                    }
                    sum += 1.0 / (value as f64);
                    reachables += 1;
                }
            }
        }

        let efficiency = sum / ((max_time as f64) * (graph.nb_nodes * (graph.nb_nodes - 1)) as f64);
        return (efficiency, reachables);
    }
}