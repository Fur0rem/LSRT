#include "../src/sparse_graph.h"
#include <time.h>

int main(int argc, char ** argv){

    /*graph alloc / print / free tests */
    declare_sparse_graph(spg);
    init_graph(&spg, 30, 50);

    time_t t; 
    time(&t); 
    srand(t);

    for(uint32_t i = 0 ; i < 30 ; i++){
        spg.elems[i] = rand()%100; 
        spg.col_index[i] = i ;
    }
    spg.size_elems_arr = 30; 
    spg.size_row_arr = 50;
    
    for(uint32_t i = 0 ; i < 50 ; i ++){
        spg.row_index[i] = i ; 
    }

   // int64_t ret;
   // declare_link(lnk); 
    //lnk.col = 30; 
    //lnk.row = 50; 
    //get_link(&ret, &spg, lnk);
    

    //bool ret_exist;
    //link_exists(&ret_exist,&spg, lnk);

    //printf("ret=%li ex=%u\n", ret, ret_exist);
    
    FILE * dest = fopen_check("g", "w");
    write_graph(dest, &spg);
    fclose(dest);

    declare_sparse_graph(spg2);
    FILE * source = fopen_check("g","r");
    read_graph(source,&spg2);
    printf("----------\n");
     write_graph(stdout, &spg2);

    fclose(source);
    free_graph(&spg2);

    free_graph(&spg);



    return 0; 
}
