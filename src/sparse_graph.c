#include "sparse_graph.h"

err_code init_graph(SPARSE_GRAPH * sg, uint32_t sea, uint32_t sra ){

    if(!sg) return ERR_NULL ;

    sg->elems = calloc(sea, sizeof(uint32_t));
    if(! sg->elems) return ERR_ALLOC; 

    sg->col_index = calloc(sea, sizeof(uint32_t));
    if(! sg->col_index) return ERR_ALLOC; 

    sg->row_index = calloc(sra, sizeof(uint32_t)); 
    if(!sg->row_index) return ERR_ALLOC;

    return ERR_OK;
}//not tested 

void free_graph(SPARSE_GRAPH * sg){
    if(sg){
        if(sg->elems) free(sg->elems); 
        if(sg->col_index) free(sg->col_index); 
        if(sg->row_index) free(sg->row_index);
        sg->elems = sg->col_index = sg->row_index = NULL ; 
        sg->size_row_arr = sg->size_elems_arr = 0 ; 
    }
}//not tested 
