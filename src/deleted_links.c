#include "deleted_links.h"
#include "utils.h"

static err_code init_uarr32(UARR_32 * arr, uint32_t size){
    def_err_handler(!arr, "init_uarr32", ERR_NULL);

    arr->elems = calloc(size, sizeof(uint32_t)); 
    def_err_handler(!arr->elems, "init_uarr32", ERR_ALLOC);

    arr->size = size ; 

    return ERR_OK;
}//not tested

static err_code add_uarr32(UARR_32 * arr, uint32_t index, uint32_t elem){
    def_err_handler(!arr, "add_uarr32", ERR_NULL);

    if(index < arr->size){
        arr->elems[index] = elem ;
    }

    return ERR_OK;
}//not tested

static void free_uarr32(UARR_32 *  arr){
    if(arr){
        if(arr->elems) free(arr->elems); 
        arr->elems = NULL; 
        arr->size = 0 ;
    }
}//not tested

static int less_than( const void * nb1, const void * nb2){
    //not doing the nb1 - nb2 bc uint32_t might overflow
    int ret ;

    uint32_t val1 = (uint32_t) * ((uint32_t *)  nb1) ;
    uint32_t val2 = (uint32_t) * ((uint32_t *)  nb2) ;

    if( val1 == val2){
        ret = 0 ; 
    }else if(val1 < val2){
        ret =-1 ;
    }else{
        ret = 1 ;
    }
    return ret ;
}

static err_code parse_uarr32(UARR_32 * arr, char * source){
    
    char * start ;
    char * end ; 
    
    start = source ;
    skip_char(' ', &start);
    end  = start; 

    uint32_t size = strtol(start, &end, DEC_BASE);
    def_err_handler((start == end), "parse_uarr32", ERR_FORMAT);
    
    err_code failure = init_uarr32(arr, size);
    def_err_handler(failure, "parse_uarr32", failure);

    failure = fill_arr(end, arr->elems, size); 
    def_err_handler(failure, "parse_uarr32", failure);

    qsort((void*)arr->elems, arr->size, sizeof(uint32_t), less_than);

    return ERR_OK;
}//not tested

static err_code fprint_uarr32(FILE * dest, UARR_32 * arr){
    def_err_handler(!(arr && dest), "fprint_uarr32", ERR_NULL);

    fprintf(dest, "%u ", arr->size);
    for(uint32_t i = 0 ; i < arr->size ; i++){
        fprintf(dest, "%u ", arr->elems[i]);
    }
    fprintf(dest, "\n");

    return ERR_OK;
}//not tested

err_code init_dlt(DELETED_LINKS_TAB * dlt, uint32_t size ){
    def_err_handler(!dlt, "init_dlt", ERR_NULL);

    dlt->elems = calloc(size, sizeof(UARR_32));
    def_err_handler(!dlt->elems, "init_dlt", ERR_NULL);
    
    dlt->size = size ;

    return ERR_OK; 
}//not tested 

void free_dlt(DELETED_LINKS_TAB * dlt, uint32_t size){

    if(dlt){
        if(dlt->elems){
            for(uint32_t i = 0 ; i < size ; i ++){
                free_uarr32(&dlt->elems[i]);
            }
            free(dlt->elems);
        }
        dlt->elems = NULL ; 
        dlt->size = 0 ;
    }
}//not tested 

err_code write_dlt(FILE * flux, DELETED_LINKS_TAB * dlt){
    def_err_handler(!(flux && dlt), "write_dlt", ERR_NULL);

    fprintf(flux, "%u\n", dlt->size);
    for(uint32_t i = 0 ; i < dlt->size ; i++){
        fprint_uarr32(flux, &dlt->elems[i]);
    }
    fprintf(flux,"\n");

    return ERR_OK;
}//not tested

err_code read_dlt(FILE * flux, DELETED_LINKS_TAB * dlt){
    def_err_handler(!(flux && dlt), "write_dlt", ERR_NULL);

    char buff[BUFF_SIZE] ;
    def_err_handler(!fgets(buff,BUFF_SIZE,flux),"read_dlt",ERR_FORMAT);

    char * start ;
    char * end ;
    start = buff; 
    skip_char(' ', &start) ; 
    end = start ; 

    uint32_t size = strtol(start, &end, DEC_BASE);
    def_err_handler((start == end), "read_dlt", ERR_FORMAT);

    err_code failure = init_dlt(dlt, size);
    def_err_handler(failure, "read_dlt", failure);

    for(uint32_t i = 0 ; i < size ; i++){
        def_err_handler(!fgets(buff,BUFF_SIZE,flux),"read_dlt",ERR_FORMAT);
        failure = parse_uarr32(&dlt->elems[i], buff);
        def_err_handler(failure, "read_dlt", failure);
    }

 

    return ERR_OK;
}//not tested


static err_code dicho_search(bool * ret, UARR_32 * arr, uint32_t elem){
    def_err_handler(!arr, "dicho_search", ERR_NULL);

    uint32_t left = 0 ; 
    uint32_t right = arr->size ;
    *ret = false ;

    while(left != right){
        uint32_t cur_index = (left + right) / 2 ;

        if(arr->elems[cur_index] < elem){
            left = cur_index + 1; 
        }else if(arr->elems[cur_index] > elem){
            right = cur_index - 1 ;
        }else{
            *ret = true ; 
            left = right ;
        }
    }
    return ERR_OK;
}

err_code is_deleted(bool * ret, DELETED_LINKS_TAB * dlt, uint32_t link, uint32_t time){
    def_err_handler(!(ret && dlt), "is_deleted", ERR_NULL);
   
    err_code failure = dicho_search(ret, &dlt->elems[link], time);
    def_err_handler(failure, "is_deleted", failure);

    return ERR_OK;
}//not tested
