#include "../src/algo.h"
#include "../src/utils.h"

int main(int argc, char ** argv){

    declare_link_stream(lks);


    FILE * gfile = fopen("tests_files/test_graph", "r"); 
    FILE * afile = fopen("tests_files/test_attack", "r"); 
    err_code failure = parse_link_stream(&lks, gfile, afile);
    def_err_handler(failure, "main", failure);

    fclose(gfile);
    fclose(afile);

    
    write_graph(stdout, &lks.sparse_graph);
    printf("---\n");
    write_dlt(stdout, &lks.deleted_links);
    
     
    declare_dma(dma); 
    printf("lks.deleted_links.size=%u\n", lks.deleted_links.size);
    init_dma(&dma, lks.deleted_links.size, 3,3);
    
    failure = temporal_floyd_warshall(&lks, &dma );
    def_err_handler(failure, "main", failure);
  
     printf("---\n");
    fprint_dma(stdout, &dma);
      free_dma(&dma); 
    free_link_stream(&lks);

    return 0 ; 
}
