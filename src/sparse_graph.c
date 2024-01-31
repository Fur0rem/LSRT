#include "sparse_graph.h"
#include "utils.h"

err_code init_graph(SPARSE_GRAPH* spg, uint32_t sea, uint32_t sra) {

	def_err_handler(!spg, "init_graph", ERR_NULL);

	spg->elems = calloc(sea, sizeof(uint32_t));
	def_err_handler(!spg->elems, "init_graph", ERR_ALLOC);

	spg->col_index = calloc(sea, sizeof(uint32_t));
	def_err_handler(!spg->col_index, "init_graph", ERR_ALLOC);


	spg->row_index = calloc(sra, sizeof(uint32_t));
	def_err_handler(!spg->row_index, "init_graph", ERR_ALLOC);

	spg->size_elems_arr = sea ; 
	spg->size_row_arr = sra ; 

	return ERR_OK;
} // tested ; seems ok

void free_graph(SPARSE_GRAPH* spg) {
	if (spg) {
		if (spg->elems) {
			free(spg->elems);
		}
		if (spg->col_index) {
			free(spg->col_index);
		}
		if (spg->row_index) {
			free(spg->row_index);
		}
		spg->elems = spg->col_index = spg->row_index = NULL;
		spg->size_row_arr = spg->size_elems_arr = 0;
	}
} // tested ; ok 


err_code get_link( int64_t * ret, SPARSE_GRAPH * spg , LINK lnk){
   
	def_err_handler(!(ret && spg), "get_link", ERR_NULL);
	def_err_handler((spg->size_row_arr - 1 < lnk.row), "get link valcheck 1", ERR_VAL);
	def_err_handler((spg->size_elems_arr - 1 < lnk.col ), "get_link valcheck 2", ERR_VAL);

    int64_t row_start = spg->row_index[lnk.row];
    int64_t row_end ;
    if(spg->size_row_arr - 1 == lnk.row ){
        row_end = spg->size_elems_arr ;
    }else{
        row_end = spg->row_index[lnk.row + 1 ];
    }
    
    for(uint32_t i = row_start ; i < row_end ; i++){
        if(spg->col_index[i] == lnk.col ){
            *ret = spg->elems[i] ;
            return ERR_OK ;
        }
    }
    *ret = -1 ;
    return ERR_OK ;
}//tested ; seems ok


err_code link_exists(bool* ret, SPARSE_GRAPH* spg, LINK lnk) {
	def_err_handler(!(ret && spg), "link_exists", ERR_NULL);

	int64_t tmp;
	err_code failure = get_link(&tmp, spg, lnk);
	def_err_handler(failure, "link_exists", failure);

	*ret = (tmp != -1);
	return ERR_OK;
} // tested ok if get_link is ok

err_code set_link(SPARSE_GRAPH* spg, LINK lnk, uint32_t weight) {
	def_err_handler(!( spg), "set_link", ERR_NULL);
    

    int64_t row_start = spg->row_index[lnk.row];
    int64_t row_end ;
    if(spg->size_row_arr - 1 == lnk.row ){
        row_end = spg->size_elems_arr ;
    }else{
        row_end = spg->row_index[lnk.row + 1 ];
    }
    
    for(uint32_t i = row_start ; i < row_end ; i++){
        if(spg->col_index[i] == lnk.col ){
            spg->elems[i] = weight ;
            return ERR_OK ;
        }
    }
    return ERR_OK ;
} // not tested

extern err_code write_graph(FILE* flux_dest, SPARSE_GRAPH* graph_source) {
	def_err_handler(!(flux_dest && graph_source ), "write_graph", ERR_NULL);

	fprintf(flux_dest, "%u %u\n", graph_source->size_elems_arr,
			graph_source->size_row_arr);

	for (uint32_t i = 0; i < graph_source->size_elems_arr; i++) {
		fprintf(flux_dest, "%u ", graph_source->elems[i]);
	}
	fprintf(flux_dest, "\n");
	for (uint32_t i = 0; i < graph_source->size_elems_arr; i++) {
		fprintf(flux_dest, "%u ", graph_source->col_index[i]);
	}
	fprintf(flux_dest, "\n");
	for (uint32_t i = 0; i < graph_source->size_row_arr; i++) {
		fprintf(flux_dest, "%u ", graph_source->row_index[i]);
	}
	fprintf(flux_dest, "\n");
	return ERR_OK;
} // not tested


err_code read_graph(FILE* flux_source, SPARSE_GRAPH* graph_dest) {
	def_err_handler(!(flux_source && graph_dest ), "read_graph", ERR_NULL);

	char buff[BUFF_SIZE];
	fgets(buff, BUFF_SIZE, flux_source);

	char* end = (char*)buff;
	char* start = (char*)buff;

	uint32_t size_elems_arr = 0;
	size_elems_arr = strtol(start, &end, DEC_BASE);
	def_err_handler(start == end , "read_graph 1" ,ERR_VAL);

	start = end;

	uint32_t size_row_arr = 0;
	size_row_arr = strtol(start, &end, DEC_BASE);
	def_err_handler(start == end , "read_graph 2" ,ERR_VAL);


	err_code failure = init_graph(graph_dest, size_elems_arr, size_row_arr);
	def_err_handler(failure, "read_graph" , failure);

	def_err_handler((!fgets(buff, BUFF_SIZE, flux_source)) , "read_graph" , failure);
	failure = fill_arr(buff, graph_dest->elems, size_elems_arr);
	def_err_handler(failure, "read_graph" , failure);

	def_err_handler((!fgets(buff, BUFF_SIZE, flux_source)) , "read_graph" , failure);
	failure = fill_arr(buff, graph_dest->col_index, size_elems_arr);
	def_err_handler(failure, "read_graph" , failure);

	def_err_handler((!fgets(buff, BUFF_SIZE, flux_source)) , "read_graph" , failure);
	failure = fill_arr(buff, graph_dest->row_index, size_row_arr);
	def_err_handler(failure, "read_graph" , failure);

	return ERR_OK;
} // tested seems ok; changed; need to retest
