#include "../src/sparse_graph.h"

int main(int argc, char ** argv){

    /*graph alloc / print / free tests */
    declare_sparse_graph(sg);
    init_graph(&sg, 4, 3);

    for(uint32_t i = 0 ; i < 4 ; i++){
        sg.elems[i] = i; 
        sg.col_index[i] = i ;
    }
    sg.size_elems_arr = 4; 
    sg.size_row_arr = 3 ;
    
    sg.row_index[0] = 0 ; 
    sg.row_index[1] = 2; 
    sg.row_index[2] = 3; 

    int64_t ret;
    declare_link(l); 
    l.col = 15; 
    l.row = 1; 
    get_link(&ret, &sg, l);
    

    bool ret_exist;
    link_exists(&ret_exist,&sg, l);

    printf("ret=%li ex=%u\n", ret, ret_exist);
    
    write_graph(stdout, &sg);
    free_graph(&sg);



    return 0; 
}
