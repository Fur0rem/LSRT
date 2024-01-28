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

err_code get_link( int64_t * ret, SPARSE_GRAPH * sg , LINK l){
    if(!ret || !sg) return ERR_NULL;
    if(sg->size_row_arr +1  < l.row ) return ERR_VAL ;

    int64_t row_start = sg->row_index[l.row];
    int64_t row_end = sg->row_index[l.row + 1 ];
    
    for(uint32_t i = row_start ; i < row_end ; i++){
        if(sg->col_index[i] == l.col ){
            *ret = sg->elems[i] ;
            return ERR_OK ;
        }
    }

    *ret = -1 ;
    return ERR_OK ;
}//not tested 

err_code link_exists(bool * ret, SPARSE_GRAPH * sg , LINK l){

    int64_t tmp;
    err_code failure = get_link( &tmp, sg, l);
    if(failure) return failure;

    *ret = (tmp != -1);
    return ERR_OK;
}//not tested ; 

err_code set_link(SPARSE_GRAPH * sg, LINK l, uint32_t weight){
    if(!sg) return ERR_NULL;
    if(sg->size_row_arr +1  < l.row ) return ERR_VAL ;

    int64_t row_start = sg->row_index[l.row];
    int64_t row_end = sg->row_index[l.row + 1 ];
    
    for(uint32_t i = row_start ; i < row_end ; i++){
        if(sg->col_index[i] == l.col ){
            sg->elems[i] = weight ;
            return ERR_OK ;
        }
    }

    return ERR_OK;
}//not tested 



extern err_code write_graph(FILE * flux_dest, SPARSE_GRAPH * graph_source){

    return ERR_OK;
}//not done

extern err_code read_graph(FILE * flux_source, SPARSE_GRAPH * graph_dest){

    return ERR_OK; 
}//not done