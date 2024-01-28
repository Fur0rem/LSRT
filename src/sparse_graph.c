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
}//tested ; works

void free_graph(SPARSE_GRAPH * sg){
    if(sg){
        if(sg->elems) free(sg->elems); 
        if(sg->col_index) free(sg->col_index); 
        if(sg->row_index) free(sg->row_index);
        sg->elems = sg->col_index = sg->row_index = NULL ; 
        sg->size_row_arr = sg->size_elems_arr = 0 ; 
    }
}//tested ; works

err_code get_link( int64_t * ret, SPARSE_GRAPH * sg , LINK l){
    if(!ret || !sg) return ERR_NULL;
    if(sg->size_row_arr - 1 < l.row ) {*ret = -1; return ERR_OK;}
    if(sg->size_elems_arr - 1 < l.col )  {*ret = -1; return ERR_OK;}

    int64_t row_start = sg->row_index[l.row];
    int64_t row_end ;
    if(sg->size_row_arr - 1 == l.row ){
        row_end = sg->size_elems_arr ;
    }else{
        row_end = sg->row_index[l.row + 1 ];
    }
    
    for(uint32_t i = row_start ; i < row_end ; i++){
        if(sg->col_index[i] == l.col ){
            *ret = sg->elems[i] ;
            return ERR_OK ;
        }
    }
    *ret = -1 ;
    return ERR_OK ;
}//tested ; seems ok

err_code link_exists(bool * ret, SPARSE_GRAPH * sg , LINK l){

    int64_t tmp;
    err_code failure = get_link( &tmp, sg, l);
    if(failure) return failure;

    *ret = (tmp != -1);
    return ERR_OK;
}//tested ; ok if get_link is ok

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
}//not tested ; wrong (see get link for fix)

extern err_code write_graph(FILE * flux_dest, SPARSE_GRAPH * graph_source){
    if(! (flux_dest && graph_source)) return ERR_NULL;

    fprintf(flux_dest, "%u %u\n", graph_source->size_elems_arr, graph_source->size_row_arr);
    
    for(uint32_t i = 0 ; i < graph_source->size_elems_arr; i++){
        fprintf(flux_dest, "%u ", graph_source->elems[i]);
    }
    fprintf(flux_dest, "\n");
    for(uint32_t i = 0 ; i < graph_source->size_elems_arr; i++){
        fprintf(flux_dest, "%u ", graph_source->col_index[i]);
    }
    fprintf(flux_dest, "\n");
    for(uint32_t i = 0 ; i < graph_source->size_row_arr; i++){
        fprintf(flux_dest, "%u ", graph_source->row_index[i]);
    }
    fprintf(flux_dest, "\n");
    return ERR_OK;
}//tested; works

static void skip_char(char c, char ** buff){
    while(**buff == c && **buff != '\0') (*buff)++;
}//not tested static helper ; doesn't check for shit.

static void fill_arr(char * str_source, uint32_t * arr_dest, uint32_t arr_size){

    uint32_t cpt = 0; 

    char * start ,*end ; 
    start = end = str_source ;

    do{ 
        start = end ; 

        uint32_t parsed_num = strtol(start, &end, 10); //parses a number
        if(start != end){
            arr_dest[cpt] = parsed_num ;
        }

        skip_char(' ', &start); //goes to the next non space (next number)
        
        cpt++;
    }while(cpt < arr_size && start != end);
}//not tested; static helper ; doesn't check for shit
//prolly wrong

err_code read_graph(FILE * flux_source, SPARSE_GRAPH * graph_dest){
    if(! (flux_source && graph_dest)) return ERR_NULL;

    char buff[256]; 
    fgets(buff, 256, flux_source);
    skip_char(' ', (char**)&buff);

    char * end = (char*)&buff;
    char * start = (char*)&buff;
    
    uint32_t size_elems_arr = 0;
    size_elems_arr = strtol(start, &end,10);
    if(start == end ){
        return ERR_VAL;
    }
    start = end ; 

    uint32_t size_row_arr = 0;
    size_row_arr = strtol(start, &end,10);
    if(start == end ){
        return ERR_VAL;
    }
    start = end ;

    err_code failure = init_graph(graph_dest, size_elems_arr, size_row_arr); 
    if(failure) return failure;

    if(!fgets(buff, 256, flux_source)) return ERR_FORMAT;
    fill_arr(buff, graph_dest->elems, size_elems_arr); 

    if(!fgets(buff, 256, flux_source)) return ERR_FORMAT;
    fill_arr(buff, graph_dest->col_index, size_elems_arr); 

    if(!fgets(buff, 256, flux_source)) return ERR_FORMAT;
    fill_arr(buff, graph_dest->row_index, size_row_arr);

    return ERR_OK; 
}//not tested; 100% wrong 

