#include "../src/sparse_graph.h"

int main(int argc, char ** argv){

    declare_sparse_graph(sg);

    init_graph(&sg, 4, 3);

    for(uint32_t i = 0 ; i < 4 ; i++){
        sg.elems[i] = i+1; 
        sg.col_index[i] = i ;
    }

    for(uint32_t i = 0 ; i < sg.size_row_arr ;i++){
        sg.row_index[i] = i ;
    }
    
    write_graph(stdout, &sg);
    
    free_graph(&sg);

    return 0; 
}
